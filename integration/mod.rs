use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{debug, info, warn, error, trace};
use thiserror::Error;
use uuid::Uuid;

// 内部モジュールをインポート
pub mod aether_services;
pub mod cloud_providers;
pub mod compat;
pub mod device_portal;
pub mod nexus_bridge;

// コアシステムコンポーネントをインポート
use crate::core::system::{
    security::{
        SecurityManager, SecurityContext, SecurityLevel, SecurityToken, SecurityError, SecurityResult,
        permissions::{Permission, PermissionManager},
        security_context::{Credentials, PolicyManager}
    },
    notification_service::{
        NotificationService, Notification, NotificationCategory, 
        NotificationPriority, NotificationAction, NotificationId
    },
    power_interface::{PowerInterface, PowerSource, BatteryInfo, PowerPlan}
};

/// 統合モジュールのエラー型
#[derive(Error, Debug, Clone)]
pub enum IntegrationError {
    #[error("認証エラー: {0}")]
    AuthenticationError(String),
    
    #[error("権限エラー: {0}")]
    PermissionError(String),
    
    #[error("接続エラー: {0}")]
    ConnectionError(String),
    
    #[error("設定エラー: {0}")]
    ConfigurationError(String),
    
    #[error("サービスエラー: {0}")]
    ServiceError(String),
    
    #[error("互換性エラー: {0}")]
    CompatibilityError(String),
    
    #[error("内部エラー: {0}")]
    InternalError(String),
    
    #[error("セキュリティエラー: {0}")]
    SecurityError(#[from] SecurityError),

    #[error("タイムアウトエラー: {operation}が{duration:?}後にタイムアウトしました")]
    TimeoutError { operation: String, duration: Duration },
    
    #[error("リソース制限エラー: {0}")]
    ResourceLimitError(String),
    
    #[error("同期エラー: {0}")]
    SynchronizationError(String),
    
    #[error("プラグインエラー: {plugin_id} - {message}")]
    PluginError { plugin_id: String, message: String },
    
    #[error("依存関係エラー: {0}")]
    DependencyError(String),
}

impl IntegrationError {
    /// エラーの重大度を取得
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::AuthenticationError(_) | 
            Self::PermissionError(_) | 
            Self::SecurityError(_) => ErrorSeverity::Critical,
            
            Self::ConnectionError(_) | 
            Self::TimeoutError { .. } | 
            Self::SynchronizationError(_) => ErrorSeverity::High,
            
            Self::ConfigurationError(_) | 
            Self::ServiceError(_) | 
            Self::CompatibilityError(_) |
            Self::ResourceLimitError(_) |
            Self::PluginError { .. } => ErrorSeverity::Medium,
            
            Self::InternalError(_) |
            Self::DependencyError(_) => ErrorSeverity::Low,
        }
    }
    
    /// エラーをリカバリー可能かどうかを判定
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::AuthenticationError(_) | 
            Self::ConnectionError(_) | 
            Self::TimeoutError { .. } |
            Self::ServiceError(_) => true,
            
            Self::PermissionError(_) | 
            Self::SecurityError(_) |
            Self::InternalError(_) => false,
            
            Self::ConfigurationError(_) | 
            Self::CompatibilityError(_) |
            Self::ResourceLimitError(_) |
            Self::SynchronizationError(_) |
            Self::PluginError { .. } |
            Self::DependencyError(_) => true,
        }
    }
    
    /// エラーIDを生成（トラッキング用）
    pub fn generate_error_id() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// エラーをログに記録
    pub fn log(&self, context: Option<&str>) {
        let error_id = Self::generate_error_id();
        let severity = self.severity();
        let context_str = context.unwrap_or("未指定のコンテキスト");
        
        match severity {
            ErrorSeverity::Critical => {
                error!("[ERROR-{:?}] 重大なエラー (ID: {}): {} - コンテキスト: {}", 
                      severity, error_id, self, context_str);
            },
            ErrorSeverity::High => {
                error!("[ERROR-{:?}] エラー (ID: {}): {} - コンテキスト: {}", 
                      severity, error_id, self, context_str);
            },
            ErrorSeverity::Medium => {
                warn!("[ERROR-{:?}] 警告 (ID: {}): {} - コンテキスト: {}", 
                     severity, error_id, self, context_str);
            },
            ErrorSeverity::Low => {
                debug!("[ERROR-{:?}] 軽微なエラー (ID: {}): {} - コンテキスト: {}", 
                      severity, error_id, self, context_str);
            },
        }
    }
}

/// エラーの重大度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 重大なエラー - システム全体に影響
    Critical,
    /// 高いエラー - 特定の機能が使用不可
    High,
    /// 中程度のエラー - 機能は低下するが使用可能
    Medium,
    /// 低いエラー - ほとんど影響なし
    Low,
}

/// 統合結果型
pub type IntegrationResult<T> = Result<T, IntegrationError>;

/// 統合モジュールの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationState {
    /// 未初期化
    Uninitialized,
    /// 初期化中
    Initializing,
    /// 接続中
    Connecting,
    /// 接続済み
    Connected,
    /// 同期中
    Synchronizing,
    /// エラー発生
    Error,
    /// 一時停止
    Paused,
    /// 切断済み
    Disconnected,
    /// 終了処理中
    ShuttingDown,
    /// 強制終了
    Terminated,
}

impl std::fmt::Display for IntegrationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized => write!(f, "未初期化"),
            Self::Initializing => write!(f, "初期化中"),
            Self::Connecting => write!(f, "接続中"),
            Self::Connected => write!(f, "接続済み"),
            Self::Synchronizing => write!(f, "同期中"),
            Self::Error => write!(f, "エラー発生"),
            Self::Paused => write!(f, "一時停止"),
            Self::Disconnected => write!(f, "切断済み"),
            Self::ShuttingDown => write!(f, "終了処理中"),
            Self::Terminated => write!(f, "強制終了"),
        }
    }
}

impl IntegrationState {
    /// 状態が活動中かどうかを確認
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Connected | Self::Synchronizing)
    }
    
    /// 状態が移行中かどうかを確認
    pub fn is_transitioning(&self) -> bool {
        matches!(self, Self::Initializing | Self::Connecting | Self::ShuttingDown)
    }
    
    /// 状態がエラーまたは終了状態かどうかを確認
    pub fn is_error_or_terminated(&self) -> bool {
        matches!(self, Self::Error | Self::Terminated)
    }
}

/// 統合プラグインのトレイト
/// 外部サービスや機能との統合を提供するプラグインインターフェース
#[allow(unused_variables)]
pub trait IntegrationPlugin: Send + Sync {
    /// プラグインの一意識別子を取得
    fn id(&self) -> &str;
    
    /// プラグインの名前を取得
    fn name(&self) -> &str;
    
    /// プラグインの説明を取得
    fn description(&self) -> &str;
    
    /// プラグインのバージョンを取得
    fn version(&self) -> &str;
    
    /// プラグインが必要とする権限のリストを取得
    fn required_permissions(&self) -> Vec<Permission>;
    
    /// プラグインの初期化
    fn initialize(&self, context: &IntegrationContext) -> IntegrationResult<()>;
    
    /// プラグインの終了処理
    fn shutdown(&self) -> IntegrationResult<()>;
    
    /// プラグインの現在の状態を取得
    fn state(&self) -> IntegrationState;
    
    /// プラグインを接続
    fn connect(&self) -> IntegrationResult<()>;
    
    /// プラグインを切断
    fn disconnect(&self) -> IntegrationResult<()>;
    
    /// プラグインを一時停止
    fn pause(&self) -> IntegrationResult<()>;
    
    /// プラグインを再開
    fn resume(&self) -> IntegrationResult<()>;
    
    /// プラグインの同期
    fn synchronize(&self) -> IntegrationResult<()>;
    
    /// プラグインが指定された機能をサポートしているかどうかを確認
    fn supports_feature(&self, feature_name: &str) -> bool {
        false
    }
    
    /// プラグインの状態メトリクスを取得
    fn get_metrics(&self) -> IntegrationResult<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
    }
    
    /// プラグインのヘルスチェックを実行
    fn health_check(&self) -> IntegrationResult<IntegrationHealth> {
        Ok(IntegrationHealth::Healthy)
    }
    
    /// プラグインのヘルプテキストを取得
    fn get_help(&self) -> String {
        format!("{}:\n{}\nバージョン: {}", self.name(), self.description(), self.version())
    }
}

/// 統合モジュールのヘルス状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationHealth {
    /// 正常
    Healthy,
    /// 部分的に正常
    PartiallyHealthy,
    /// 異常あり
    Unhealthy,
    /// 致命的問題
    Critical,
}

impl std::fmt::Display for IntegrationHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "正常"),
            Self::PartiallyHealthy => write!(f, "部分的に正常"),
            Self::Unhealthy => write!(f, "異常あり"),
            Self::Critical => write!(f, "致命的問題"),
        }
    }
}

/// 統合コンテキスト
/// 統合プラグインに提供されるコンテキスト情報と共有サービス
pub struct IntegrationContext {
    /// セキュリティマネージャー
    security_manager: Arc<SecurityManager>,
    
    /// 通知サービス
    notification_service: Arc<NotificationService>,
    
    /// 電源インターフェース
    power_interface: Arc<PowerInterface>,
    
    /// プラグインのクレデンシャル
    credentials: Arc<RwLock<HashMap<String, Credentials>>>,
    
    /// プラグインのセキュリティトークン
    tokens: Arc<RwLock<HashMap<String, SecurityToken>>>,
    
    /// カスタムデータストア
    data_store: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    
    /// 統合メトリクス収集
    metrics: Arc<RwLock<HashMap<String, HashMap<String, serde_json::Value>>>>,
    
    /// エラー履歴（プラグインID -> エラーリスト）
    error_history: Arc<RwLock<HashMap<String, Vec<(Instant, IntegrationError)>>>>,
    
    /// プラグイン間の共有状態
    shared_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl IntegrationContext {
    /// 新しい統合コンテキストを作成
    pub fn new(
        security_manager: Arc<SecurityManager>,
        notification_service: Arc<NotificationService>,
        power_interface: Arc<PowerInterface>,
    ) -> Self {
        Self {
            security_manager,
            notification_service,
            power_interface,
            credentials: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(RwLock::new(HashMap::new())),
            data_store: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            error_history: Arc::new(RwLock::new(HashMap::new())),
            shared_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// セキュリティマネージャーを取得
    pub fn security_manager(&self) -> Arc<SecurityManager> {
        self.security_manager.clone()
    }
    
    /// 通知サービスを取得
    pub fn notification_service(&self) -> Arc<NotificationService> {
        self.notification_service.clone()
    }
    
    /// 電源インターフェースを取得
    pub fn power_interface(&self) -> Arc<PowerInterface> {
        self.power_interface.clone()
    }
    
    /// プラグインのクレデンシャルを設定
    pub fn set_credentials(&self, plugin_id: &str, credentials: Credentials) -> IntegrationResult<()> {
        let mut creds = self.credentials.write().map_err(|e| 
            IntegrationError::InternalError(format!("クレデンシャルロックの取得に失敗: {}", e))
        )?;
        
        creds.insert(plugin_id.to_string(), credentials);
        Ok(())
    }
    
    /// プラグインのクレデンシャルを取得
    pub fn get_credentials(&self, plugin_id: &str) -> IntegrationResult<Option<Credentials>> {
        let creds = self.credentials.read().map_err(|e| 
            IntegrationError::InternalError(format!("クレデンシャルロックの取得に失敗: {}", e))
        )?;
        
        Ok(creds.get(plugin_id).cloned())
    }
    
    /// プラグインのセキュリティトークンを設定
    pub fn set_token(&self, plugin_id: &str, token: SecurityToken) -> IntegrationResult<()> {
        let mut tokens = self.tokens.write().map_err(|e| 
            IntegrationError::InternalError(format!("トークンロックの取得に失敗: {}", e))
        )?;
        
        tokens.insert(plugin_id.to_string(), token);
        Ok(())
    }
    
    /// プラグインのセキュリティトークンを取得
    pub fn get_token(&self, plugin_id: &str) -> IntegrationResult<Option<SecurityToken>> {
        let tokens = self.tokens.read().map_err(|e| 
            IntegrationError::InternalError(format!("トークンロックの取得に失敗: {}", e))
        )?;
        
        Ok(tokens.get(plugin_id).cloned())
    }
    
    /// プラグインのデータを設定
    pub fn set_data(&self, plugin_id: &str, key: &str, value: &str) -> IntegrationResult<()> {
        let mut store = self.data_store.write().map_err(|e| 
            IntegrationError::InternalError(format!("データストアロックの取得に失敗: {}", e))
        )?;
        
        let plugin_store = store.entry(plugin_id.to_string()).or_insert_with(HashMap::new);
        plugin_store.insert(key.to_string(), value.to_string());
        
        Ok(())
    }
    
    /// プラグインのデータを取得
    pub fn get_data(&self, plugin_id: &str, key: &str) -> IntegrationResult<Option<String>> {
        let store = self.data_store.read().map_err(|e| 
            IntegrationError::InternalError(format!("データストアロックの取得に失敗: {}", e))
        )?;
        
        Ok(store
            .get(plugin_id)
            .and_then(|plugin_store| plugin_store.get(key).cloned()))
    }
    
    /// 通知を送信
    pub fn send_notification(&self, notification: Notification) -> IntegrationResult<NotificationId> {
        self.notification_service.send_notification(notification)
            .map_err(|e| IntegrationError::ServiceError(format!("通知の送信に失敗: {}", e)))
    }
    
    /// 権限をチェック
    pub fn check_permission(&self, plugin_id: &str, permission: &Permission) -> IntegrationResult<bool> {
        self.security_manager.check_permission(plugin_id, permission)
            .map_err(|e| IntegrationError::SecurityError(e))
    }
    
    /// 現在の電源ソースを取得
    pub fn get_power_source(&self) -> PowerSource {
        self.power_interface.get_power_source()
    }
    
    /// バッテリー情報を取得
    pub fn get_battery_info(&self) -> HashMap<String, BatteryInfo> {
        self.power_interface.get_battery_info()
    }
    
    /// プラグインのメトリクスを記録
    pub fn record_metrics(&self, plugin_id: &str, metrics: HashMap<String, serde_json::Value>) -> IntegrationResult<()> {
        let mut metrics_store = self.metrics.write().map_err(|e| 
            IntegrationError::InternalError(format!("メトリクスロックの取得に失敗: {}", e))
        )?;
        
        metrics_store.insert(plugin_id.to_string(), metrics);
        Ok(())
    }
    
    /// プラグインのメトリクスを取得
    pub fn get_metrics(&self, plugin_id: &str) -> IntegrationResult<Option<HashMap<String, serde_json::Value>>> {
        let metrics_store = self.metrics.read().map_err(|e| 
            IntegrationError::InternalError(format!("メトリクスロックの取得に失敗: {}", e))
        )?;
        
        Ok(metrics_store.get(plugin_id).cloned())
    }
    
    /// エラーを記録
    pub fn record_error(&self, plugin_id: &str, error: IntegrationError) -> IntegrationResult<()> {
        let mut error_store = self.error_history.write().map_err(|e| 
            IntegrationError::InternalError(format!("エラー履歴ロックの取得に失敗: {}", e))
        )?;
        
        let plugin_errors = error_store.entry(plugin_id.to_string()).or_insert_with(Vec::new);
        plugin_errors.push((Instant::now(), error.clone()));
        
        // エラー履歴が大きくなりすぎないように古いエラーを削除
        const MAX_ERROR_HISTORY: usize = 100;
        if plugin_errors.len() > MAX_ERROR_HISTORY {
            plugin_errors.sort_by_key(|(time, _)| *time);
            plugin_errors.truncate(MAX_ERROR_HISTORY);
        }
        
        // エラーをログに記録
        error.log(Some(&format!("プラグイン: {}", plugin_id)));
        
        Ok(())
    }
    
    /// 共有状態を設定
    pub fn set_shared_state(&self, key: &str, value: serde_json::Value) -> IntegrationResult<()> {
        let mut state = self.shared_state.write().map_err(|e| 
            IntegrationError::InternalError(format!("共有状態ロックの取得に失敗: {}", e))
        )?;
        
        state.insert(key.to_string(), value);
        Ok(())
    }
    
    /// 共有状態を取得
    pub fn get_shared_state(&self, key: &str) -> IntegrationResult<Option<serde_json::Value>> {
        let state = self.shared_state.read().map_err(|e| 
            IntegrationError::InternalError(format!("共有状態ロックの取得に失敗: {}", e))
        )?;
        
        Ok(state.get(key).cloned())
    }
}

/// 統合マネージャー
pub struct IntegrationManager {
    /// 統合コンテキスト
    context: Arc<IntegrationContext>,
    
    /// 登録されたプラグイン
    plugins: RwLock<HashMap<String, Box<dyn IntegrationPlugin>>>,
    
    /// プラグインの状態
    plugin_states: RwLock<HashMap<String, IntegrationState>>,
    
    /// 初期化状態
    initialized: RwLock<bool>,
}

impl IntegrationManager {
    /// 新しい統合マネージャーを作成
    pub fn new(
        security_manager: Arc<SecurityManager>,
        notification_service: Arc<NotificationService>,
        power_interface: Arc<PowerInterface>,
    ) -> Self {
        let context = Arc::new(IntegrationContext::new(
            security_manager,
            notification_service,
            power_interface,
        ));
        
        Self {
            context,
            plugins: RwLock::new(HashMap::new()),
            plugin_states: RwLock::new(HashMap::new()),
            initialized: RwLock::new(false),
        }
    }
    
    /// 統合マネージャーを初期化
    pub fn initialize(&self) -> IntegrationResult<()> {
        let mut initialized = self.initialized.write().map_err(|e| {
            IntegrationError::InternalError(format!("初期化状態の更新中にエラーが発生しました: {}", e))
        })?;
        
        if *initialized {
            return Ok(());
        }
        
        // 初期化されていない場合は初期化を実施
        
        // システムプラグインを登録
        self.register_system_plugins()?;
        
        *initialized = true;
        Ok(())
    }
    
    /// システムプラグインを登録
    fn register_system_plugins(&self) -> IntegrationResult<()> {
        // Aetherサービスプラグインを登録
        self.register_aether_services_plugins()?;
        
        // クラウドプロバイダープラグインを登録
        self.register_cloud_provider_plugins()?;
        
        // 互換性プラグインを登録
        self.register_compat_plugins()?;
        
        // デバイスポータルプラグインを登録
        self.register_device_portal_plugins()?;
        
        // Nexusブリッジプラグインを登録
        self.register_nexus_bridge_plugins()?;
        
        Ok(())
    }
    
    /// Aetherサービスプラグインを登録
    fn register_aether_services_plugins(&self) -> IntegrationResult<()> {
        // TODO: Aetherサービスプラグインの実装を追加
        Ok(())
    }
    
    /// クラウドプロバイダープラグインを登録
    fn register_cloud_provider_plugins(&self) -> IntegrationResult<()> {
        // TODO: クラウドプロバイダープラグインの実装を追加
        Ok(())
    }
    
    /// 互換性プラグインを登録
    fn register_compat_plugins(&self) -> IntegrationResult<()> {
        // TODO: 互換性プラグインの実装を追加
        Ok(())
    }
    
    /// デバイスポータルプラグインを登録
    fn register_device_portal_plugins(&self) -> IntegrationResult<()> {
        // TODO: デバイスポータルプラグインの実装を追加
        Ok(())
    }
    
    /// Nexusブリッジプラグインを登録
    fn register_nexus_bridge_plugins(&self) -> IntegrationResult<()> {
        // TODO: Nexusブリッジプラグインの実装を追加
        Ok(())
    }
    
    /// プラグインを登録
    pub fn register_plugin(&self, plugin: Box<dyn IntegrationPlugin>) -> IntegrationResult<()> {
        let plugin_id = plugin.id().to_string();
        
        let mut plugins = self.plugins.write().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの登録中にエラーが発生しました: {}", e))
        })?;
        
        if plugins.contains_key(&plugin_id) {
            return Err(IntegrationError::ConfigurationError(
                format!("プラグイン '{}' はすでに登録されています", plugin_id)
            ));
        }
        
        // プラグインの状態を初期化
        let mut states = self.plugin_states.write().map_err(|e| {
            IntegrationError::InternalError(format!("プラグイン状態の更新中にエラーが発生しました: {}", e))
        })?;
        
        states.insert(plugin_id.clone(), IntegrationState::Uninitialized);
        
        // プラグインを登録
        plugins.insert(plugin_id, plugin);
        
        Ok(())
    }
    
    /// プラグインを登録解除
    pub fn unregister_plugin(&self, plugin_id: &str) -> IntegrationResult<()> {
        let mut plugins = self.plugins.write().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの登録解除中にエラーが発生しました: {}", e))
        })?;
        
        if let Some(plugin) = plugins.remove(plugin_id) {
            // プラグインの終了処理を実行
            plugin.shutdown()?;
            
            // プラグインの状態を削除
            let mut states = self.plugin_states.write().map_err(|e| {
                IntegrationError::InternalError(format!("プラグイン状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            states.remove(plugin_id);
            
            Ok(())
        } else {
            Err(IntegrationError::ConfigurationError(
                format!("プラグイン '{}' は登録されていません", plugin_id)
            ))
        }
    }
    
    /// プラグインを取得
    pub fn get_plugin(&self, plugin_id: &str) -> IntegrationResult<Option<Box<dyn IntegrationPlugin + '_>>> {
        let plugins = self.plugins.read().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの取得中にエラーが発生しました: {}", e))
        })?;
        
        Ok(plugins.get(plugin_id).map(|plugin| Box::new(plugin.as_ref())))
    }
    
    /// 登録されたすべてのプラグインを取得
    pub fn get_all_plugins(&self) -> IntegrationResult<Vec<String>> {
        let plugins = self.plugins.read().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの取得中にエラーが発生しました: {}", e))
        })?;
        
        Ok(plugins.keys().cloned().collect())
    }
    
    /// プラグインの状態を取得
    pub fn get_plugin_state(&self, plugin_id: &str) -> IntegrationResult<Option<IntegrationState>> {
        let states = self.plugin_states.read().map_err(|e| {
            IntegrationError::InternalError(format!("プラグイン状態の取得中にエラーが発生しました: {}", e))
        })?;
        
        Ok(states.get(plugin_id).copied())
    }
    
    /// プラグインの状態を設定
    pub fn set_plugin_state(&self, plugin_id: &str, state: IntegrationState) -> IntegrationResult<()> {
        let mut states = self.plugin_states.write().map_err(|e| {
            IntegrationError::InternalError(format!("プラグイン状態の更新中にエラーが発生しました: {}", e))
        })?;
        
        if !states.contains_key(plugin_id) {
            return Err(IntegrationError::ConfigurationError(
                format!("プラグイン '{}' は登録されていません", plugin_id)
            ));
        }
        
        states.insert(plugin_id.to_string(), state);
        
        Ok(())
    }
    
    /// プラグインを初期化
    pub fn initialize_plugin(&self, plugin_id: &str) -> IntegrationResult<()> {
        let plugins = self.plugins.read().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの取得中にエラーが発生しました: {}", e))
        })?;
        
        let plugin = plugins.get(plugin_id).ok_or_else(|| {
            IntegrationError::ConfigurationError(format!("プラグイン '{}' は登録されていません", plugin_id))
        })?;
        
        // プラグインの状態を更新
        self.set_plugin_state(plugin_id, IntegrationState::Initializing)?;
        
        // プラグインを初期化
        match plugin.initialize(&self.context) {
            Ok(_) => {
                // 初期化成功
                self.set_plugin_state(plugin_id, plugin.state())?;
                Ok(())
            }
            Err(e) => {
                // 初期化失敗
                self.set_plugin_state(plugin_id, IntegrationState::Error)?;
                Err(e)
            }
        }
    }
    
    /// プラグインを接続
    pub fn connect_plugin(&self, plugin_id: &str) -> IntegrationResult<()> {
        let plugins = self.plugins.read().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの取得中にエラーが発生しました: {}", e))
        })?;
        
        let plugin = plugins.get(plugin_id).ok_or_else(|| {
            IntegrationError::ConfigurationError(format!("プラグイン '{}' は登録されていません", plugin_id))
        })?;
        
        // プラグインの状態を更新
        self.set_plugin_state(plugin_id, IntegrationState::Connecting)?;
        
        // プラグインを接続
        match plugin.connect() {
            Ok(_) => {
                // 接続成功
                self.set_plugin_state(plugin_id, plugin.state())?;
                Ok(())
            }
            Err(e) => {
                // 接続失敗
                self.set_plugin_state(plugin_id, IntegrationState::Error)?;
                Err(e)
            }
        }
    }
    
    /// プラグインを切断
    pub fn disconnect_plugin(&self, plugin_id: &str) -> IntegrationResult<()> {
        let plugins = self.plugins.read().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの取得中にエラーが発生しました: {}", e))
        })?;
        
        let plugin = plugins.get(plugin_id).ok_or_else(|| {
            IntegrationError::ConfigurationError(format!("プラグイン '{}' は登録されていません", plugin_id))
        })?;
        
        // プラグインを切断
        match plugin.disconnect() {
            Ok(_) => {
                // 切断成功
                self.set_plugin_state(plugin_id, IntegrationState::Disconnected)?;
                Ok(())
            }
            Err(e) => {
                // 切断失敗
                self.set_plugin_state(plugin_id, IntegrationState::Error)?;
                Err(e)
            }
        }
    }
    
    /// プラグインを一時停止
    pub fn pause_plugin(&self, plugin_id: &str) -> IntegrationResult<()> {
        let plugins = self.plugins.read().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの取得中にエラーが発生しました: {}", e))
        })?;
        
        let plugin = plugins.get(plugin_id).ok_or_else(|| {
            IntegrationError::ConfigurationError(format!("プラグイン '{}' は登録されていません", plugin_id))
        })?;
        
        // プラグインを一時停止
        match plugin.pause() {
            Ok(_) => {
                // 一時停止成功
                self.set_plugin_state(plugin_id, IntegrationState::Paused)?;
                Ok(())
            }
            Err(e) => {
                // 一時停止失敗
                self.set_plugin_state(plugin_id, IntegrationState::Error)?;
                Err(e)
            }
        }
    }
    
    /// プラグインを再開
    pub fn resume_plugin(&self, plugin_id: &str) -> IntegrationResult<()> {
        let plugins = self.plugins.read().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの取得中にエラーが発生しました: {}", e))
        })?;
        
        let plugin = plugins.get(plugin_id).ok_or_else(|| {
            IntegrationError::ConfigurationError(format!("プラグイン '{}' は登録されていません", plugin_id))
        })?;
        
        // プラグインを再開
        match plugin.resume() {
            Ok(_) => {
                // 再開成功
                self.set_plugin_state(plugin_id, plugin.state())?;
                Ok(())
            }
            Err(e) => {
                // 再開失敗
                self.set_plugin_state(plugin_id, IntegrationState::Error)?;
                Err(e)
            }
        }
    }
    
    /// プラグインを同期
    pub fn synchronize_plugin(&self, plugin_id: &str) -> IntegrationResult<()> {
        let plugins = self.plugins.read().map_err(|e| {
            IntegrationError::InternalError(format!("プラグインの取得中にエラーが発生しました: {}", e))
        })?;
        
        let plugin = plugins.get(plugin_id).ok_or_else(|| {
            IntegrationError::ConfigurationError(format!("プラグイン '{}' は登録されていません", plugin_id))
        })?;
        
        // プラグインの状態を更新
        self.set_plugin_state(plugin_id, IntegrationState::Synchronizing)?;
        
        // プラグインを同期
        match plugin.synchronize() {
            Ok(_) => {
                // 同期成功
                self.set_plugin_state(plugin_id, plugin.state())?;
                Ok(())
            }
            Err(e) => {
                // 同期失敗
                self.set_plugin_state(plugin_id, IntegrationState::Error)?;
                Err(e)
            }
        }
    }
    
    /// すべてのプラグインを初期化
    pub fn initialize_all_plugins(&self) -> IntegrationResult<Vec<(String, IntegrationResult<()>)>> {
        let plugin_ids = self.get_all_plugins()?;
        let mut results = Vec::new();
        
        for plugin_id in plugin_ids {
            let result = self.initialize_plugin(&plugin_id);
            results.push((plugin_id, result));
        }
        
        Ok(results)
    }
    
    /// すべてのプラグインを接続
    pub fn connect_all_plugins(&self) -> IntegrationResult<Vec<(String, IntegrationResult<()>)>> {
        let plugin_ids = self.get_all_plugins()?;
        let mut results = Vec::new();
        
        for plugin_id in plugin_ids {
            let result = self.connect_plugin(&plugin_id);
            results.push((plugin_id, result));
        }
        
        Ok(results)
    }
    
    /// すべてのプラグインを切断
    pub fn disconnect_all_plugins(&self) -> IntegrationResult<Vec<(String, IntegrationResult<()>)>> {
        let plugin_ids = self.get_all_plugins()?;
        let mut results = Vec::new();
        
        for plugin_id in plugin_ids {
            let result = self.disconnect_plugin(&plugin_id);
            results.push((plugin_id, result));
        }
        
        Ok(results)
    }
    
    /// 統合コンテキストを取得
    pub fn get_context(&self) -> Arc<IntegrationContext> {
        self.context.clone()
    }
}

// グローバル統合マネージャーのインスタンス
static mut INTEGRATION_MANAGER: Option<Arc<IntegrationManager>> = None;
static INTEGRATION_MANAGER_INIT: std::sync::Once = std::sync::Once::new();

/// グローバル統合マネージャーを初期化
pub fn initialize_integration_manager(
    security_manager: Arc<SecurityManager>,
    notification_service: Arc<NotificationService>,
    power_interface: Arc<PowerInterface>,
) -> IntegrationResult<()> {
    INTEGRATION_MANAGER_INIT.call_once(|| {
        let manager = Arc::new(IntegrationManager::new(
            security_manager,
            notification_service,
            power_interface,
        ));
        
        unsafe {
            INTEGRATION_MANAGER = Some(manager);
        }
    });
    
    get_integration_manager()?.initialize()
}

/// グローバル統合マネージャーを取得
pub fn get_integration_manager() -> IntegrationResult<Arc<IntegrationManager>> {
    unsafe {
        INTEGRATION_MANAGER.clone().ok_or_else(|| {
            IntegrationError::InternalError("統合マネージャーが初期化されていません".to_string())
        })
    }
}

/// 統合マネージャーのシャットダウン
pub fn shutdown_integration_manager() -> IntegrationResult<()> {
    let manager = get_integration_manager()?;
    
    // すべてのプラグインを切断
    manager.disconnect_all_plugins()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // モックプラグインの実装
    struct MockPlugin {
        id: String,
        name: String,
        description: String,
        version: String,
        state: std::sync::RwLock<IntegrationState>,
    }
    
    impl MockPlugin {
        fn new(id: &str, name: &str, description: &str, version: &str) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                version: version.to_string(),
                state: std::sync::RwLock::new(IntegrationState::Uninitialized),
            }
        }
    }
    
    impl IntegrationPlugin for MockPlugin {
        fn id(&self) -> &str {
            &self.id
        }
        
        fn name(&self) -> &str {
            &self.name
        }
        
        fn description(&self) -> &str {
            &self.description
        }
        
        fn version(&self) -> &str {
            &self.version
        }
        
        fn required_permissions(&self) -> Vec<Permission> {
            Vec::new() // モックのためパーミッションは空
        }
        
        fn initialize(&self, _context: &IntegrationContext) -> IntegrationResult<()> {
            let mut state = self.state.write().unwrap();
            *state = IntegrationState::Initialized;
            Ok(())
        }
        
        fn shutdown(&self) -> IntegrationResult<()> {
            let mut state = self.state.write().unwrap();
            *state = IntegrationState::Disconnected;
            Ok(())
        }
        
        fn state(&self) -> IntegrationState {
            *self.state.read().unwrap()
        }
        
        fn connect(&self) -> IntegrationResult<()> {
            let mut state = self.state.write().unwrap();
            *state = IntegrationState::Connected;
            Ok(())
        }
        
        fn disconnect(&self) -> IntegrationResult<()> {
            let mut state = self.state.write().unwrap();
            *state = IntegrationState::Disconnected;
            Ok(())
        }
        
        fn pause(&self) -> IntegrationResult<()> {
            let mut state = self.state.write().unwrap();
            *state = IntegrationState::Paused;
            Ok(())
        }
        
        fn resume(&self) -> IntegrationResult<()> {
            let mut state = self.state.write().unwrap();
            *state = IntegrationState::Connected;
            Ok(())
        }
        
        fn synchronize(&self) -> IntegrationResult<()> {
            let mut state = self.state.write().unwrap();
            *state = IntegrationState::Synchronizing;
            // 同期完了後は接続状態に戻す
            *state = IntegrationState::Connected;
            Ok(())
        }
    }
    
    // テスト用のヘルパー関数
    fn create_test_manager() -> IntegrationManager {
        // TODO: モックの実装
        // 実際のテストでは、モックのセキュリティマネージャー、通知サービス、電源インターフェースを作成する
        unimplemented!()
    }
    
    #[test]
    fn test_integration_plugin_lifecycle() {
        // TODO: プラグインのライフサイクル（初期化、接続、一時停止、再開、切断）をテスト
        unimplemented!()
    }
    
    #[test]
    fn test_integration_manager_register_plugin() {
        // TODO: プラグインの登録と登録解除をテスト
        unimplemented!()
    }
    
    #[test]
    fn test_integration_context_data_store() {
        // TODO: 統合コンテキストのデータストア機能をテスト
        unimplemented!()
    }
    
    #[test]
    fn test_integration_state_transitions() {
        // TODO: プラグインの状態遷移をテスト
        unimplemented!()
    }
}

// 再エクスポート
pub use nexus_bridge::NexusBridge;
pub use compat::CompatManager; 