use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};

use crate::core::system::{
    security::{SecurityContext, SecurityLevel, SecurityToken, SecurityError},
    notification_service::{Notification, NotificationCategory, NotificationPriority},
};

use crate::integration::{
    IntegrationPlugin, IntegrationContext, IntegrationState, 
    IntegrationError, IntegrationResult
};

/// NexusBridgeの接続状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NexusConnectionState {
    /// 未接続
    Disconnected,
    /// 接続中
    Connecting,
    /// 接続済み
    Connected,
    /// 認証中
    Authenticating,
    /// 認証済み
    Authenticated,
    /// 同期中
    Synchronizing,
    /// 接続エラー
    Error,
}

impl std::fmt::Display for NexusConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NexusConnectionState::Disconnected => write!(f, "未接続"),
            NexusConnectionState::Connecting => write!(f, "接続中"),
            NexusConnectionState::Connected => write!(f, "接続済み"),
            NexusConnectionState::Authenticating => write!(f, "認証中"),
            NexusConnectionState::Authenticated => write!(f, "認証済み"),
            NexusConnectionState::Synchronizing => write!(f, "同期中"),
            NexusConnectionState::Error => write!(f, "エラー"),
        }
    }
}

/// Nexusシステムからのイベント種別
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NexusEventType {
    /// システム状態更新
    SystemStateUpdate,
    /// システム設定変更
    SystemConfigChange,
    /// アプリケーションイベント
    ApplicationEvent,
    /// セキュリティイベント
    SecurityEvent,
    /// ユーザーイベント
    UserEvent,
    /// デバイスイベント
    DeviceEvent,
    /// ネットワークイベント
    NetworkEvent,
    /// 同期イベント
    SynchronizationEvent,
    /// カスタムイベント
    Custom(String),
}

/// Nexusシステムからのイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusEvent {
    /// イベントID
    pub id: String,
    /// イベント種別
    pub event_type: NexusEventType,
    /// イベント発生時間（エポックからのミリ秒）
    pub timestamp: u64,
    /// イベントソース
    pub source: String,
    /// イベントデータ（JSON形式）
    pub data: serde_json::Value,
    /// イベントの優先度
    pub priority: u8,
}

/// Nexusへ送信するコマンド種別
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NexusCommandType {
    /// 接続要求
    Connect,
    /// 切断要求
    Disconnect,
    /// 認証要求
    Authenticate,
    /// 同期要求
    Synchronize,
    /// システム設定更新
    UpdateSystemConfig,
    /// アプリケーション操作
    ApplicationControl,
    /// ユーザー操作
    UserControl,
    /// デバイス操作
    DeviceControl,
    /// カスタムコマンド
    Custom(String),
}

/// Nexusへ送信するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusCommand {
    /// コマンドID
    pub id: String,
    /// コマンド種別
    pub command_type: NexusCommandType,
    /// コマンドデータ（JSON形式）
    pub data: serde_json::Value,
    /// タイムアウト（ミリ秒）
    pub timeout_ms: Option<u64>,
}

/// Nexusコマンドの応答
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusCommandResponse {
    /// コマンドID
    pub command_id: String,
    /// 成功したかどうか
    pub success: bool,
    /// 応答データ（JSON形式）
    pub data: Option<serde_json::Value>,
    /// エラーメッセージ
    pub error_message: Option<String>,
}

/// NexusBridgeの設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusBridgeConfig {
    /// 接続先のNexusシステムのアドレス
    pub nexus_address: String,
    /// 接続ポート
    pub nexus_port: u16,
    /// 接続タイムアウト（ミリ秒）
    pub connection_timeout_ms: u64,
    /// 再接続の試行回数
    pub reconnect_attempts: u32,
    /// 再接続の間隔（ミリ秒）
    pub reconnect_interval_ms: u64,
    /// セキュアモードを使用するかどうか
    pub use_secure_mode: bool,
    /// 自動再接続を有効にするかどうか
    pub enable_auto_reconnect: bool,
    /// イベントバッファサイズ
    pub event_buffer_size: usize,
}

impl Default for NexusBridgeConfig {
    fn default() -> Self {
        Self {
            nexus_address: "localhost".to_string(),
            nexus_port: 9876,
            connection_timeout_ms: 5000,
            reconnect_attempts: 5,
            reconnect_interval_ms: 2000,
            use_secure_mode: true,
            enable_auto_reconnect: true,
            event_buffer_size: 100,
        }
    }
}

/// NexusBridgeのイベントハンドラ型
pub type NexusEventHandler = Box<dyn Fn(&NexusEvent) -> IntegrationResult<()> + Send + Sync>;

/// Nexus Bridge プラグイン
pub struct NexusBridgePlugin {
    /// プラグインID
    id: String,
    /// プラグイン名
    name: String,
    /// 説明
    description: String,
    /// バージョン
    version: String,
    /// 接続状態
    connection_state: RwLock<NexusConnectionState>,
    /// 統合状態
    integration_state: RwLock<IntegrationState>,
    /// 設定
    config: RwLock<NexusBridgeConfig>,
    /// イベントハンドラ
    event_handlers: RwLock<HashMap<String, NexusEventHandler>>,
    /// 最終接続時間
    last_connected: RwLock<Option<Instant>>,
    /// 最終同期時間
    last_synced: RwLock<Option<Instant>>,
    /// イベントバッファ
    event_buffer: Mutex<Vec<NexusEvent>>,
    /// コンテキスト
    context: Mutex<Option<Arc<IntegrationContext>>>,
}

impl NexusBridgePlugin {
    /// 新しいNexusBridgeプラグインを作成
    pub fn new() -> Self {
        Self {
            id: "nexus_bridge".to_string(),
            name: "Nexus Bridge".to_string(),
            description: "AetherOSのNexusシステムとLumosDesktopを接続するブリッジ".to_string(),
            version: "1.0.0".to_string(),
            connection_state: RwLock::new(NexusConnectionState::Disconnected),
            integration_state: RwLock::new(IntegrationState::Uninitialized),
            config: RwLock::new(NexusBridgeConfig::default()),
            event_handlers: RwLock::new(HashMap::new()),
            last_connected: RwLock::new(None),
            last_synced: RwLock::new(None),
            event_buffer: Mutex::new(Vec::new()),
            context: Mutex::new(None),
        }
    }
    
    /// 設定を更新
    pub fn update_config(&self, config: NexusBridgeConfig) -> IntegrationResult<()> {
        let mut current_config = self.config.write().map_err(|e| {
            IntegrationError::InternalError(format!("設定の更新中にエラーが発生しました: {}", e))
        })?;
        
        *current_config = config;
        Ok(())
    }
    
    /// 現在の設定を取得
    pub fn get_config(&self) -> IntegrationResult<NexusBridgeConfig> {
        let config = self.config.read().map_err(|e| {
            IntegrationError::InternalError(format!("設定の取得中にエラーが発生しました: {}", e))
        })?;
        
        Ok(config.clone())
    }
    
    /// 接続状態を設定
    fn set_connection_state(&self, state: NexusConnectionState) -> IntegrationResult<()> {
        let mut current_state = self.connection_state.write().map_err(|e| {
            IntegrationError::InternalError(format!("接続状態の設定中にエラーが発生しました: {}", e))
        })?;
        
        *current_state = state;
        
        // 接続状態に応じて統合状態も更新
        let integration_state = match state {
            NexusConnectionState::Disconnected => IntegrationState::Disconnected,
            NexusConnectionState::Connecting => IntegrationState::Connecting,
            NexusConnectionState::Connected | NexusConnectionState::Authenticated => IntegrationState::Connected,
            NexusConnectionState::Authenticating => IntegrationState::Initializing,
            NexusConnectionState::Synchronizing => IntegrationState::Synchronizing,
            NexusConnectionState::Error => IntegrationState::Error,
        };
        
        let mut current_integration_state = self.integration_state.write().map_err(|e| {
            IntegrationError::InternalError(format!("統合状態の設定中にエラーが発生しました: {}", e))
        })?;
        
        *current_integration_state = integration_state;
        
        Ok(())
    }
    
    /// 接続状態を取得
    pub fn get_connection_state(&self) -> IntegrationResult<NexusConnectionState> {
        let state = self.connection_state.read().map_err(|e| {
            IntegrationError::InternalError(format!("接続状態の取得中にエラーが発生しました: {}", e))
        })?;
        
        Ok(*state)
    }
    
    /// イベントハンドラを登録
    pub fn register_event_handler<F>(&self, handler_id: &str, handler: F) -> IntegrationResult<()> 
    where
        F: Fn(&NexusEvent) -> IntegrationResult<()> + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.write().map_err(|e| {
            IntegrationError::InternalError(format!("イベントハンドラの登録中にエラーが発生しました: {}", e))
        })?;
        
        handlers.insert(handler_id.to_string(), Box::new(handler));
        Ok(())
    }
    
    /// イベントハンドラを解除
    pub fn unregister_event_handler(&self, handler_id: &str) -> IntegrationResult<()> {
        let mut handlers = self.event_handlers.write().map_err(|e| {
            IntegrationError::InternalError(format!("イベントハンドラの解除中にエラーが発生しました: {}", e))
        })?;
        
        handlers.remove(handler_id);
        Ok(())
    }
    
    /// イベントを処理
    fn process_event(&self, event: &NexusEvent) -> IntegrationResult<()> {
        let handlers = self.event_handlers.read().map_err(|e| {
            IntegrationError::InternalError(format!("イベントハンドラの取得中にエラーが発生しました: {}", e))
        })?;
        
        for handler in handlers.values() {
            if let Err(e) = handler(event) {
                error!("イベントハンドラでエラーが発生しました: {}", e);
                // エラーが発生しても他のハンドラは実行する
            }
        }
        
        Ok(())
    }
    
    /// イベントバッファにイベントを追加
    fn add_event_to_buffer(&self, event: NexusEvent) -> IntegrationResult<()> {
        let mut buffer = self.event_buffer.lock().map_err(|e| {
            IntegrationError::InternalError(format!("イベントバッファへのアクセス中にエラーが発生しました: {}", e))
        })?;
        
        let config = self.config.read().map_err(|e| {
            IntegrationError::InternalError(format!("設定の取得中にエラーが発生しました: {}", e))
        })?;
        
        // バッファがいっぱいの場合は古いイベントを削除
        if buffer.len() >= config.event_buffer_size {
            buffer.remove(0);
        }
        
        buffer.push(event);
        Ok(())
    }
    
    /// イベントバッファからすべてのイベントを取得
    pub fn get_all_events(&self) -> IntegrationResult<Vec<NexusEvent>> {
        let buffer = self.event_buffer.lock().map_err(|e| {
            IntegrationError::InternalError(format!("イベントバッファへのアクセス中にエラーが発生しました: {}", e))
        })?;
        
        Ok(buffer.clone())
    }
    
    /// イベントバッファをクリア
    pub fn clear_event_buffer(&self) -> IntegrationResult<()> {
        let mut buffer = self.event_buffer.lock().map_err(|e| {
            IntegrationError::InternalError(format!("イベントバッファへのアクセス中にエラーが発生しました: {}", e))
        })?;
        
        buffer.clear();
        Ok(())
    }
    
    /// コマンドを送信
    pub fn send_command(&self, command: NexusCommand) -> IntegrationResult<NexusCommandResponse> {
        let connection_state = self.get_connection_state()?;
        
        if connection_state != NexusConnectionState::Connected && 
           connection_state != NexusConnectionState::Authenticated {
            return Err(IntegrationError::ConnectionError(
                "Nexusに接続されていないためコマンドを送信できません".to_string()
            ));
        }
        
        // TODO: 実際のNexusシステムへのコマンド送信処理を実装
        // 現在はモックレスポンスを返す
        let response = NexusCommandResponse {
            command_id: command.id.clone(),
            success: true,
            data: Some(serde_json::json!({
                "status": "success",
                "message": "コマンドが正常に処理されました",
                "timestamp": chrono::Utc::now().timestamp_millis(),
            })),
            error_message: None,
        };
        
        Ok(response)
    }
    
    /// Nexusシステムに接続
    fn connect_to_nexus(&self) -> IntegrationResult<()> {
        let config = self.get_config()?;
        
        self.set_connection_state(NexusConnectionState::Connecting)?;
        
        // TODO: 実際のNexusシステムへの接続処理を実装
        // 現在はモック接続処理
        
        // 接続成功を模擬
        self.set_connection_state(NexusConnectionState::Connected)?;
        
        // 最終接続時間を更新
        let mut last_connected = self.last_connected.write().map_err(|e| {
            IntegrationError::InternalError(format!("最終接続時間の更新中にエラーが発生しました: {}", e))
        })?;
        
        *last_connected = Some(Instant::now());
        
        // 接続通知を送信
        if let Some(context) = self.context.lock().map_err(|e| {
            IntegrationError::InternalError(format!("コンテキストの取得中にエラーが発生しました: {}", e))
        })?.as_ref() {
            let notification = Notification::new(
                "nexus_connected".to_string(),
                "Nexus接続成功".to_string(),
                format!("AetherOSのNexusシステムに接続しました: {}", config.nexus_address),
                NotificationCategory::System,
                NotificationPriority::Normal,
            );
            
            if let Err(e) = context.notification_service().add_notification(notification) {
                warn!("接続通知の送信に失敗しました: {}", e);
            }
        }
        
        info!("Nexusシステムに接続しました: {}:{}", config.nexus_address, config.nexus_port);
        Ok(())
    }
    
    /// Nexusシステム認証
    fn authenticate_with_nexus(&self) -> IntegrationResult<()> {
        self.set_connection_state(NexusConnectionState::Authenticating)?;
        
        let context_guard = self.context.lock().map_err(|e| {
            IntegrationError::InternalError(format!("コンテキストの取得中にエラーが発生しました: {}", e))
        })?;
        
        let context = context_guard.as_ref().ok_or_else(|| {
            IntegrationError::InternalError("コンテキストが初期化されていません".to_string())
        })?;
        
        // クレデンシャルを取得
        let credentials = context.get_credentials(&self.id)?;
        
        if credentials.is_none() {
            return Err(IntegrationError::AuthenticationError(
                "Nexus認証用のクレデンシャルが設定されていません".to_string()
            ));
        }
        
        // TODO: 実際のNexusシステムでの認証処理を実装
        // 現在はモック認証処理
        
        // 認証成功を模擬
        self.set_connection_state(NexusConnectionState::Authenticated)?;
        
        info!("Nexusシステムで認証が完了しました");
        Ok(())
    }
    
    /// Nexusとの同期処理
    fn synchronize_with_nexus(&self) -> IntegrationResult<()> {
        let connection_state = self.get_connection_state()?;
        
        if connection_state != NexusConnectionState::Authenticated {
            return Err(IntegrationError::ConnectionError(
                "Nexusで認証が完了していないため同期できません".to_string()
            ));
        }
        
        self.set_connection_state(NexusConnectionState::Synchronizing)?;
        
        // TODO: 実際のNexusシステムとの同期処理を実装
        // 現在はモック同期処理
        
        // 同期完了を模擬
        self.set_connection_state(NexusConnectionState::Authenticated)?;
        
        // 最終同期時間を更新
        let mut last_synced = self.last_synced.write().map_err(|e| {
            IntegrationError::InternalError(format!("最終同期時間の更新中にエラーが発生しました: {}", e))
        })?;
        
        *last_synced = Some(Instant::now());
        
        info!("Nexusシステムとの同期が完了しました");
        Ok(())
    }
    
    /// Nexusシステムから切断
    fn disconnect_from_nexus(&self) -> IntegrationResult<()> {
        let connection_state = self.get_connection_state()?;
        
        if connection_state == NexusConnectionState::Disconnected {
            return Ok(());
        }
        
        // TODO: 実際のNexusシステムからの切断処理を実装
        // 現在はモック切断処理
        
        self.set_connection_state(NexusConnectionState::Disconnected)?;
        
        // 切断通知を送信
        if let Some(context) = self.context.lock().map_err(|e| {
            IntegrationError::InternalError(format!("コンテキストの取得中にエラーが発生しました: {}", e))
        })?.as_ref() {
            let notification = Notification::new(
                "nexus_disconnected".to_string(),
                "Nexus切断".to_string(),
                "AetherOSのNexusシステムから切断しました".to_string(),
                NotificationCategory::System,
                NotificationPriority::Normal,
            );
            
            if let Err(e) = context.notification_service().add_notification(notification) {
                warn!("切断通知の送信に失敗しました: {}", e);
            }
        }
        
        info!("Nexusシステムから切断しました");
        Ok(())
    }
    
    /// デバッグ用のモックイベントを生成
    #[cfg(debug_assertions)]
    pub fn generate_mock_event(&self, event_type: NexusEventType) -> IntegrationResult<()> {
        let event = NexusEvent {
            id: format!("mock_event_{}", uuid::Uuid::new_v4()),
            event_type,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            source: "mock_generator".to_string(),
            data: serde_json::json!({
                "mock": true,
                "generated_at": chrono::Utc::now().to_rfc3339(),
            }),
            priority: 1,
        };
        
        self.add_event_to_buffer(event.clone())?;
        self.process_event(&event)?;
        
        Ok(())
    }
}

impl IntegrationPlugin for NexusBridgePlugin {
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
    
    fn required_permissions(&self) -> Vec<crate::core::system::security::permissions::Permission> {
        use crate::core::system::security::permissions::Permission;
        
        vec![
            Permission::SystemCommunication,
            Permission::SecurityTokenManagement,
            Permission::NotificationCreation,
        ]
    }
    
    fn initialize(&self, context: &IntegrationContext) -> IntegrationResult<()> {
        // すでに初期化済みの場合は何もしない
        if *self.integration_state.read().map_err(|e| {
            IntegrationError::InternalError(format!("統合状態の取得中にエラーが発生しました: {}", e))
        })? != IntegrationState::Uninitialized {
            return Ok(());
        }
        
        // コンテキストを保存
        let mut ctx = self.context.lock().map_err(|e| {
            IntegrationError::InternalError(format!("コンテキストの設定中にエラーが発生しました: {}", e))
        })?;
        
        *ctx = Some(Arc::new(context.clone()));
        
        // 統合状態を初期化中に設定
        let mut state = self.integration_state.write().map_err(|e| {
            IntegrationError::InternalError(format!("統合状態の設定中にエラーが発生しました: {}", e))
        })?;
        
        *state = IntegrationState::Initializing;
        
        // デフォルトのイベントハンドラを登録
        self.register_event_handler("default_handler", |event| {
            debug!("デフォルトハンドラでイベントを受信: {:?}", event);
            Ok(())
        })?;
        
        // 統合状態を初期化済みに設定
        *state = IntegrationState::Initialized;
        
        info!("NexusBridgeプラグインが初期化されました");
        Ok(())
    }
    
    fn shutdown(&self) -> IntegrationResult<()> {
        // 接続中の場合は切断
        let connection_state = self.get_connection_state()?;
        
        if connection_state != NexusConnectionState::Disconnected {
            self.disconnect_from_nexus()?;
        }
        
        // イベントバッファをクリア
        self.clear_event_buffer()?;
        
        // 統合状態を未初期化に設定
        let mut state = self.integration_state.write().map_err(|e| {
            IntegrationError::InternalError(format!("統合状態の設定中にエラーが発生しました: {}", e))
        })?;
        
        *state = IntegrationState::Uninitialized;
        
        info!("NexusBridgeプラグインがシャットダウンしました");
        Ok(())
    }
    
    fn state(&self) -> IntegrationState {
        self.integration_state.read().map(|state| *state).unwrap_or(IntegrationState::Error)
    }
    
    fn connect(&self) -> IntegrationResult<()> {
        // 接続状態を確認
        let connection_state = self.get_connection_state()?;
        
        if connection_state != NexusConnectionState::Disconnected {
            return Ok(());
        }
        
        // Nexusに接続
        self.connect_to_nexus()?;
        
        // 認証
        self.authenticate_with_nexus()?;
        
        Ok(())
    }
    
    fn disconnect(&self) -> IntegrationResult<()> {
        self.disconnect_from_nexus()
    }
    
    fn pause(&self) -> IntegrationResult<()> {
        // 現在の接続状態を保存
        let connection_state = self.get_connection_state()?;
        
        // すでに切断済みの場合は何もしない
        if connection_state == NexusConnectionState::Disconnected {
            return Ok(());
        }
        
        // 接続状態を保存
        if let Some(context) = self.context.lock().map_err(|e| {
            IntegrationError::InternalError(format!("コンテキストの取得中にエラーが発生しました: {}", e))
        })?.as_ref() {
            context.set_data(&self.id, "previous_connection_state", &format!("{:?}", connection_state))?;
        }
        
        // 切断
        self.disconnect_from_nexus()?;
        
        // 統合状態を一時停止に設定
        let mut state = self.integration_state.write().map_err(|e| {
            IntegrationError::InternalError(format!("統合状態の設定中にエラーが発生しました: {}", e))
        })?;
        
        *state = IntegrationState::Paused;
        
        info!("NexusBridgeプラグインが一時停止されました");
        Ok(())
    }
    
    fn resume(&self) -> IntegrationResult<()> {
        // 統合状態が一時停止でない場合は何もしない
        if *self.integration_state.read().map_err(|e| {
            IntegrationError::InternalError(format!("統合状態の取得中にエラーが発生しました: {}", e))
        })? != IntegrationState::Paused {
            return Ok(());
        }
        
        // 前回の接続状態を取得
        let previous_state_str = if let Some(context) = self.context.lock().map_err(|e| {
            IntegrationError::InternalError(format!("コンテキストの取得中にエラーが発生しました: {}", e))
        })?.as_ref() {
            context.get_data(&self.id, "previous_connection_state")?
        } else {
            None
        };
        
        // 前回の接続状態に基づいて再接続
        if let Some(state_str) = previous_state_str {
            if state_str.contains("Connected") || state_str.contains("Authenticated") {
                self.connect()?;
            }
        }
        
        info!("NexusBridgeプラグインが再開されました");
        Ok(())
    }
    
    fn synchronize(&self) -> IntegrationResult<()> {
        self.synchronize_with_nexus()
    }
}

/// NexusBridgeプラグインのインスタンスを作成
pub fn create_nexus_bridge_plugin() -> Box<dyn IntegrationPlugin> {
    Box::new(NexusBridgePlugin::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::system::security::permissions::Permission;
    
    // モックのIntegrationContextを作成
    fn create_mock_context() -> IntegrationContext {
        use crate::core::system::security::{SecurityManager, security_context::Credentials};
        use crate::core::system::notification_service::NotificationService;
        use crate::core::system::power_interface::PowerInterface;
        
        let security_manager = Arc::new(SecurityManager::new());
        let notification_service = Arc::new(NotificationService::new());
        let power_interface = Arc::new(PowerInterface::new());
        
        let context = IntegrationContext::new(
            security_manager,
            notification_service,
            power_interface,
        );
        
        // テスト用のクレデンシャルを設定
        let credentials = Credentials::new(
            "test_user".to_string(),
            "test_token".to_string(),
        );
        
        context.set_credentials("nexus_bridge", credentials).unwrap();
        
        context
    }
    
    #[test]
    fn test_nexus_bridge_plugin_lifecycle() {
        let plugin = NexusBridgePlugin::new();
        let context = create_mock_context();
        
        // 初期状態の確認
        assert_eq!(plugin.state(), IntegrationState::Uninitialized);
        
        // 初期化
        plugin.initialize(&context).unwrap();
        assert_eq!(plugin.state(), IntegrationState::Initialized);
        
        // 接続
        plugin.connect().unwrap();
        assert_eq!(plugin.get_connection_state().unwrap(), NexusConnectionState::Authenticated);
        assert_eq!(plugin.state(), IntegrationState::Connected);
        
        // 同期
        plugin.synchronize().unwrap();
        
        // 一時停止
        plugin.pause().unwrap();
        assert_eq!(plugin.state(), IntegrationState::Paused);
        
        // 再開
        plugin.resume().unwrap();
        assert_eq!(plugin.state(), IntegrationState::Connected);
        
        // 切断
        plugin.disconnect().unwrap();
        assert_eq!(plugin.get_connection_state().unwrap(), NexusConnectionState::Disconnected);
        
        // シャットダウン
        plugin.shutdown().unwrap();
        assert_eq!(plugin.state(), IntegrationState::Uninitialized);
    }
    
    #[test]
    fn test_nexus_bridge_event_handling() {
        let plugin = NexusBridgePlugin::new();
        let context = create_mock_context();
        
        // 初期化
        plugin.initialize(&context).unwrap();
        
        // イベントハンドラの登録
        let events_received = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events_received.clone();
        
        plugin.register_event_handler("test_handler", move |event| {
            let mut events = events_clone.lock().unwrap();
            events.push(event.clone());
            Ok(())
        }).unwrap();
        
        // モックイベントの生成（デバッグビルドでのみ利用可能）
        #[cfg(debug_assertions)]
        {
            plugin.generate_mock_event(NexusEventType::SystemStateUpdate).unwrap();
            plugin.generate_mock_event(NexusEventType::UserEvent).unwrap();
            
            // イベントが処理されたことを確認
            let events = events_received.lock().unwrap();
            assert_eq!(events.len(), 2);
            assert_eq!(events[0].event_type, NexusEventType::SystemStateUpdate);
            assert_eq!(events[1].event_type, NexusEventType::UserEvent);
        }
        
        // イベントハンドラの解除
        plugin.unregister_event_handler("test_handler").unwrap();
    }
    
    #[test]
    fn test_nexus_bridge_config() {
        let plugin = NexusBridgePlugin::new();
        
        // デフォルト設定の確認
        let default_config = plugin.get_config().unwrap();
        assert_eq!(default_config.nexus_address, "localhost");
        assert_eq!(default_config.nexus_port, 9876);
        
        // 設定の更新
        let mut new_config = NexusBridgeConfig::default();
        new_config.nexus_address = "192.168.1.100".to_string();
        new_config.nexus_port = 8080;
        new_config.use_secure_mode = false;
        
        plugin.update_config(new_config.clone()).unwrap();
        
        // 更新された設定の確認
        let updated_config = plugin.get_config().unwrap();
        assert_eq!(updated_config.nexus_address, "192.168.1.100");
        assert_eq!(updated_config.nexus_port, 8080);
        assert_eq!(updated_config.use_secure_mode, false);
    }
} 