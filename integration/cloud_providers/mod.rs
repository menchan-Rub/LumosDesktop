use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::integration::{
    IntegrationPlugin, IntegrationContext, IntegrationError, 
    IntegrationResult, IntegrationState
};
use crate::core::system::security::permissions::Permission;

// クラウドプロバイダーの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloudProviderType {
    /// AetherクラウドプロバイダーAPI
    AetherCloud,
    /// Googleクラウドプロバイダー
    Google,
    /// Microsoftクラウドプロバイダー
    Microsoft,
    /// Amazonクラウドプロバイダー
    Amazon,
    /// Dropboxクラウドプロバイダー
    Dropbox,
    /// Apple iCloudプロバイダー
    Apple,
    /// その他のカスタムプロバイダー
    Custom,
}

impl std::fmt::Display for CloudProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloudProviderType::AetherCloud => write!(f, "Aether Cloud"),
            CloudProviderType::Google => write!(f, "Google Drive"),
            CloudProviderType::Microsoft => write!(f, "Microsoft OneDrive"),
            CloudProviderType::Amazon => write!(f, "Amazon Drive"),
            CloudProviderType::Dropbox => write!(f, "Dropbox"),
            CloudProviderType::Apple => write!(f, "Apple iCloud"),
            CloudProviderType::Custom => write!(f, "Custom Provider"),
        }
    }
}

// ファイルの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// 通常のファイル
    File,
    /// ディレクトリ
    Directory,
    /// シンボリックリンク
    SymbolicLink,
    /// 不明なタイプ
    Unknown,
}

// クラウドファイルの情報
#[derive(Debug, Clone)]
pub struct CloudFile {
    /// ファイルID
    pub id: String,
    /// ファイル名
    pub name: String,
    /// パス（相対または絶対）
    pub path: String,
    /// ファイルの種類
    pub file_type: FileType,
    /// ファイルサイズ（バイト単位）
    pub size: u64,
    /// MIMEタイプ
    pub mime_type: Option<String>,
    /// 作成日時
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 最終更新日時
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    /// カスタムメタデータ
    pub metadata: HashMap<String, String>,
}

impl CloudFile {
    /// 新しいCloudFileインスタンスを作成
    pub fn new(id: &str, name: &str, path: &str, file_type: FileType, size: u64) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            path: path.to_string(),
            file_type,
            size,
            mime_type: None,
            created_at: None,
            modified_at: None,
            metadata: HashMap::new(),
        }
    }
    
    /// MIMEタイプを設定
    pub fn with_mime_type(mut self, mime_type: &str) -> Self {
        self.mime_type = Some(mime_type.to_string());
        self
    }
    
    /// 作成日時を設定
    pub fn with_created_at(mut self, created_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }
    
    /// 最終更新日時を設定
    pub fn with_modified_at(mut self, modified_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.modified_at = Some(modified_at);
        self
    }
    
    /// メタデータを追加
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

// クラウドプロバイダー認証情報
#[derive(Debug, Clone)]
pub struct CloudCredentials {
    /// 認証タイプ
    pub auth_type: String,
    /// クライアントID
    pub client_id: Option<String>,
    /// クライアントシークレット
    pub client_secret: Option<String>,
    /// アクセストークン
    pub access_token: Option<String>,
    /// リフレッシュトークン
    pub refresh_token: Option<String>,
    /// トークン有効期限
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 追加パラメータ
    pub additional_params: HashMap<String, String>,
}

impl CloudCredentials {
    /// 新しいCloudCredentialsインスタンスを作成
    pub fn new(auth_type: &str) -> Self {
        Self {
            auth_type: auth_type.to_string(),
            client_id: None,
            client_secret: None,
            access_token: None,
            refresh_token: None,
            expires_at: None,
            additional_params: HashMap::new(),
        }
    }
    
    /// クライアントIDを設定
    pub fn with_client_id(mut self, client_id: &str) -> Self {
        self.client_id = Some(client_id.to_string());
        self
    }
    
    /// クライアントシークレットを設定
    pub fn with_client_secret(mut self, client_secret: &str) -> Self {
        self.client_secret = Some(client_secret.to_string());
        self
    }
    
    /// アクセストークンを設定
    pub fn with_access_token(mut self, token: &str) -> Self {
        self.access_token = Some(token.to_string());
        self
    }
    
    /// リフレッシュトークンを設定
    pub fn with_refresh_token(mut self, token: &str) -> Self {
        self.refresh_token = Some(token.to_string());
        self
    }
    
    /// トークン有効期限を設定
    pub fn with_expires_at(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// 追加パラメータを設定
    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.additional_params.insert(key.to_string(), value.to_string());
        self
    }
}

// クラウドプロバイダー設定
#[derive(Debug, Clone)]
pub struct CloudProviderConfig {
    /// プロバイダーの種類
    pub provider_type: CloudProviderType,
    /// プロバイダー名
    pub name: String,
    /// ベースURL
    pub base_url: Option<String>,
    /// 認証情報
    pub credentials: Option<CloudCredentials>,
    /// API呼び出しタイムアウト（秒）
    pub timeout_sec: u64,
    /// 自動同期を有効にするかどうか
    pub auto_sync_enabled: bool,
    /// 自動同期間隔（分）
    pub auto_sync_interval_min: u64,
    /// 同期対象のディレクトリ
    pub sync_directories: Vec<String>,
    /// 無視するファイルパターン（glob）
    pub ignore_patterns: Vec<String>,
}

impl CloudProviderConfig {
    /// 新しいCloudProviderConfigインスタンスを作成
    pub fn new(provider_type: CloudProviderType, name: &str) -> Self {
        Self {
            provider_type,
            name: name.to_string(),
            base_url: None,
            credentials: None,
            timeout_sec: 60,
            auto_sync_enabled: false,
            auto_sync_interval_min: 60,
            sync_directories: Vec::new(),
            ignore_patterns: Vec::new(),
        }
    }
    
    /// 認証情報を設定
    pub fn with_credentials(mut self, credentials: CloudCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }
    
    /// ベースURLを設定
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_string());
        self
    }
    
    /// タイムアウトを設定
    pub fn with_timeout(mut self, timeout_sec: u64) -> Self {
        self.timeout_sec = timeout_sec;
        self
    }
    
    /// 自動同期設定を構成
    pub fn with_auto_sync(mut self, enabled: bool, interval_min: u64) -> Self {
        self.auto_sync_enabled = enabled;
        self.auto_sync_interval_min = interval_min;
        self
    }
    
    /// 同期ディレクトリを追加
    pub fn add_sync_directory(mut self, directory: &str) -> Self {
        self.sync_directories.push(directory.to_string());
        self
    }
    
    /// 無視するパターンを追加
    pub fn add_ignore_pattern(mut self, pattern: &str) -> Self {
        self.ignore_patterns.push(pattern.to_string());
        self
    }
}

// クラウドプロバイダートレイト
pub trait CloudProvider: Send + Sync {
    /// プロバイダーの種類を取得
    fn provider_type(&self) -> CloudProviderType;
    
    /// プロバイダー名を取得
    fn name(&self) -> &str;
    
    /// 認証を実行
    fn authenticate(&self) -> IntegrationResult<()>;
    
    /// 認証をリフレッシュ
    fn refresh_auth(&self) -> IntegrationResult<()>;
    
    /// 認証のチェック
    fn is_authenticated(&self) -> bool;
    
    /// ファイルをリスト
    fn list_files(&self, path: &str) -> IntegrationResult<Vec<CloudFile>>;
    
    /// ファイル情報を取得
    fn get_file_info(&self, file_id: &str) -> IntegrationResult<CloudFile>;
    
    /// ファイルをダウンロード
    fn download_file(&self, file_id: &str, destination: &str) -> IntegrationResult<()>;
    
    /// ファイルをアップロード
    fn upload_file(&self, local_path: &str, remote_path: &str) -> IntegrationResult<CloudFile>;
    
    /// ファイルを削除
    fn delete_file(&self, file_id: &str) -> IntegrationResult<()>;
    
    /// ディレクトリを作成
    fn create_directory(&self, path: &str) -> IntegrationResult<CloudFile>;
    
    /// ファイルを共有
    fn share_file(&self, file_id: &str, email: &str) -> IntegrationResult<String>;
    
    /// ファイル共有を停止
    fn unshare_file(&self, file_id: &str, email: &str) -> IntegrationResult<()>;
    
    /// クラウドストレージの使用状況を取得
    fn get_storage_usage(&self) -> IntegrationResult<(u64, u64)>; // 使用量, 合計容量
}

// クラウドプロバイダープラグイン
pub struct CloudProviderPlugin {
    /// プラグインID
    id: String,
    /// プラグイン名
    name: String,
    /// プラグインの説明
    description: String,
    /// プラグインのバージョン
    version: String,
    /// 状態
    state: RwLock<IntegrationState>,
    /// クラウドプロバイダーの実装
    provider: Box<dyn CloudProvider>,
    /// 同期タスクが実行中かどうか
    is_syncing: RwLock<bool>,
    /// 最後の同期時刻
    last_synced: RwLock<Option<chrono::DateTime<chrono::Utc>>>,
}

impl CloudProviderPlugin {
    /// 新しいCloudProviderPluginインスタンスを作成
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        version: &str,
        provider: Box<dyn CloudProvider>,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            state: RwLock::new(IntegrationState::Uninitialized),
            provider,
            is_syncing: RwLock::new(false),
            last_synced: RwLock::new(None),
        }
    }
    
    /// クラウドプロバイダーの実装を取得
    pub fn provider(&self) -> &dyn CloudProvider {
        self.provider.as_ref()
    }
    
    /// 同期ステータスをチェック
    pub fn is_syncing(&self) -> IntegrationResult<bool> {
        let is_syncing = self.is_syncing.read().map_err(|e| {
            IntegrationError::InternalError(format!("同期状態の読み取り中にエラーが発生しました: {}", e))
        })?;
        
        Ok(*is_syncing)
    }
    
    /// 最後の同期時刻を取得
    pub fn last_synced(&self) -> IntegrationResult<Option<chrono::DateTime<chrono::Utc>>> {
        let last_synced = self.last_synced.read().map_err(|e| {
            IntegrationError::InternalError(format!("最終同期時刻の読み取り中にエラーが発生しました: {}", e))
        })?;
        
        Ok(*last_synced)
    }
    
    /// 同期を実行
    fn perform_sync(&self, context: &IntegrationContext) -> IntegrationResult<()> {
        // 同期フラグをチェック
        {
            let is_syncing = self.is_syncing.read().map_err(|e| {
                IntegrationError::InternalError(format!("同期状態の読み取り中にエラーが発生しました: {}", e))
            })?;
            
            if *is_syncing {
                return Err(IntegrationError::ServiceError("同期はすでに実行中です".to_string()));
            }
        }
        
        // 同期フラグを設定
        {
            let mut is_syncing = self.is_syncing.write().map_err(|e| {
                IntegrationError::InternalError(format!("同期状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *is_syncing = true;
        }
        
        // 同期処理を実行
        let result = self.sync_files(context);
        
        // 同期フラグをリセット
        {
            let mut is_syncing = self.is_syncing.write().map_err(|e| {
                IntegrationError::InternalError(format!("同期状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *is_syncing = false;
        }
        
        // 最終同期時刻を更新
        if result.is_ok() {
            let mut last_synced = self.last_synced.write().map_err(|e| {
                IntegrationError::InternalError(format!("最終同期時刻の更新中にエラーが発生しました: {}", e))
            })?;
            
            *last_synced = Some(chrono::Utc::now());
        }
        
        result
    }
    
    /// ファイルの同期処理
    fn sync_files(&self, _context: &IntegrationContext) -> IntegrationResult<()> {
        // TODO: 実際の同期処理を実装
        
        // 同期処理の擬似的な実装（実際の実装では、リモートとローカルのファイルを比較して同期する）
        
        Ok(())
    }
}

impl IntegrationPlugin for CloudProviderPlugin {
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
        vec![
            Permission::from("files.read"),
            Permission::from("files.write"),
            Permission::from("network.connect"),
        ]
    }
    
    fn initialize(&self, context: &IntegrationContext) -> IntegrationResult<()> {
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Initializing;
        }
        
        // プラグインの初期化処理
        // コンテキストにプラグインデータを保存
        context.set_data(self.id(), "initialized", "true")?;
        
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Initialized;
        }
        
        Ok(())
    }
    
    fn shutdown(&self) -> IntegrationResult<()> {
        {
            let state = self.state.read().map_err(|e| {
                IntegrationError::InternalError(format!("状態の読み取り中にエラーが発生しました: {}", e))
            })?;
            
            if *state == IntegrationState::Connected {
                // 接続されている場合は切断
                drop(state);
                self.disconnect()?;
            }
        }
        
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Uninitialized;
        }
        
        Ok(())
    }
    
    fn state(&self) -> IntegrationState {
        *self.state.read().unwrap_or_else(|_| panic!("状態の読み取りに失敗しました"))
    }
    
    fn connect(&self) -> IntegrationResult<()> {
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Connecting;
        }
        
        // 認証処理
        if !self.provider.is_authenticated() {
            self.provider.authenticate()?;
        }
        
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Connected;
        }
        
        Ok(())
    }
    
    fn disconnect(&self) -> IntegrationResult<()> {
        {
            let state = self.state.read().map_err(|e| {
                IntegrationError::InternalError(format!("状態の読み取り中にエラーが発生しました: {}", e))
            })?;
            
            if *state != IntegrationState::Connected && *state != IntegrationState::Paused {
                return Err(IntegrationError::ConnectionError(
                    "プロバイダーは接続されていないため、切断できません".to_string()
                ));
            }
        }
        
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Disconnected;
        }
        
        Ok(())
    }
    
    fn pause(&self) -> IntegrationResult<()> {
        {
            let state = self.state.read().map_err(|e| {
                IntegrationError::InternalError(format!("状態の読み取り中にエラーが発生しました: {}", e))
            })?;
            
            if *state != IntegrationState::Connected {
                return Err(IntegrationError::ConnectionError(
                    "プロバイダーは接続されていないため、一時停止できません".to_string()
                ));
            }
        }
        
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Paused;
        }
        
        Ok(())
    }
    
    fn resume(&self) -> IntegrationResult<()> {
        {
            let state = self.state.read().map_err(|e| {
                IntegrationError::InternalError(format!("状態の読み取り中にエラーが発生しました: {}", e))
            })?;
            
            if *state != IntegrationState::Paused {
                return Err(IntegrationError::ConnectionError(
                    "プロバイダーは一時停止されていないため、再開できません".to_string()
                ));
            }
        }
        
        // 認証の更新
        if self.provider.is_authenticated() {
            self.provider.refresh_auth()?;
        } else {
            self.provider.authenticate()?;
        }
        
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Connected;
        }
        
        Ok(())
    }
    
    fn synchronize(&self) -> IntegrationResult<()> {
        {
            let state = self.state.read().map_err(|e| {
                IntegrationError::InternalError(format!("状態の読み取り中にエラーが発生しました: {}", e))
            })?;
            
            if *state != IntegrationState::Connected {
                return Err(IntegrationError::ConnectionError(
                    "プロバイダーは接続されていないため、同期できません".to_string()
                ));
            }
        }
        
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Synchronizing;
        }
        
        // 統合コンテキストの取得（モックのため省略）
        let context = &IntegrationContext::new(
            Arc::new(crate::core::system::security::SecurityManager::new()),
            Arc::new(crate::core::system::notification_service::NotificationService::new()),
            Arc::new(crate::core::system::power_interface::PowerInterface::new()),
        );
        
        // 同期処理を実行
        let result = self.perform_sync(context);
        
        {
            let mut state = self.state.write().map_err(|e| {
                IntegrationError::InternalError(format!("状態の更新中にエラーが発生しました: {}", e))
            })?;
            
            *state = IntegrationState::Connected;
        }
        
        result
    }
}

// クラウドプロバイダーファクトリトレイト
pub trait CloudProviderFactory: Send + Sync {
    /// プロバイダーの種類を取得
    fn provider_type(&self) -> CloudProviderType;
    
    /// プロバイダー名を取得
    fn provider_name(&self) -> &str;
    
    /// プロバイダーの説明を取得
    fn provider_description(&self) -> &str;
    
    /// プロバイダーインスタンスを作成
    fn create_provider(&self, config: CloudProviderConfig) -> IntegrationResult<Box<dyn CloudProvider>>;
    
    /// プラグインインスタンスを作成
    fn create_plugin(&self, config: CloudProviderConfig) -> IntegrationResult<Box<dyn IntegrationPlugin>>;
}

// クラウドプロバイダーレジストリ
pub struct CloudProviderRegistry {
    factories: RwLock<HashMap<CloudProviderType, Box<dyn CloudProviderFactory>>>,
}

impl CloudProviderRegistry {
    /// 新しいCloudProviderRegistryインスタンスを作成
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
        }
    }
    
    /// プロバイダーファクトリを登録
    pub fn register_factory(&self, factory: Box<dyn CloudProviderFactory>) -> IntegrationResult<()> {
        let provider_type = factory.provider_type();
        
        let mut factories = self.factories.write().map_err(|e| {
            IntegrationError::InternalError(format!("ファクトリマップの更新中にエラーが発生しました: {}", e))
        })?;
        
        if factories.contains_key(&provider_type) {
            return Err(IntegrationError::ConfigurationError(
                format!("プロバイダータイプ {:?} のファクトリはすでに登録されています", provider_type)
            ));
        }
        
        factories.insert(provider_type, factory);
        
        Ok(())
    }
    
    /// プロバイダーファクトリを取得
    pub fn get_factory(&self, provider_type: CloudProviderType) -> IntegrationResult<Option<&dyn CloudProviderFactory>> {
        let factories = self.factories.read().map_err(|e| {
            IntegrationError::InternalError(format!("ファクトリマップの読み取り中にエラーが発生しました: {}", e))
        })?;
        
        Ok(factories.get(&provider_type).map(|f| f.as_ref()))
    }
    
    /// 利用可能なプロバイダータイプを取得
    pub fn get_available_provider_types(&self) -> IntegrationResult<Vec<CloudProviderType>> {
        let factories = self.factories.read().map_err(|e| {
            IntegrationError::InternalError(format!("ファクトリマップの読み取り中にエラーが発生しました: {}", e))
        })?;
        
        Ok(factories.keys().cloned().collect())
    }
    
    /// プロバイダーを作成
    pub fn create_provider(&self, config: CloudProviderConfig) -> IntegrationResult<Box<dyn CloudProvider>> {
        let factory = self.get_factory(config.provider_type)?
            .ok_or_else(|| IntegrationError::ConfigurationError(
                format!("プロバイダータイプ {:?} のファクトリが見つかりません", config.provider_type)
            ))?;
        
        factory.create_provider(config)
    }
    
    /// プラグインを作成
    pub fn create_plugin(&self, config: CloudProviderConfig) -> IntegrationResult<Box<dyn IntegrationPlugin>> {
        let factory = self.get_factory(config.provider_type)?
            .ok_or_else(|| IntegrationError::ConfigurationError(
                format!("プロバイダータイプ {:?} のファクトリが見つかりません", config.provider_type)
            ))?;
        
        factory.create_plugin(config)
    }
}

impl Default for CloudProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// グローバルクラウドプロバイダーレジストリのインスタンス
static mut CLOUD_PROVIDER_REGISTRY: Option<Arc<CloudProviderRegistry>> = None;
static CLOUD_PROVIDER_REGISTRY_INIT: std::sync::Once = std::sync::Once::new();

/// グローバルクラウドプロバイダーレジストリを初期化
pub fn initialize_cloud_provider_registry() -> IntegrationResult<Arc<CloudProviderRegistry>> {
    CLOUD_PROVIDER_REGISTRY_INIT.call_once(|| {
        let registry = Arc::new(CloudProviderRegistry::new());
        
        unsafe {
            CLOUD_PROVIDER_REGISTRY = Some(registry);
        }
    });
    
    get_cloud_provider_registry()
}

/// グローバルクラウドプロバイダーレジストリを取得
pub fn get_cloud_provider_registry() -> IntegrationResult<Arc<CloudProviderRegistry>> {
    unsafe {
        CLOUD_PROVIDER_REGISTRY.clone().ok_or_else(|| {
            IntegrationError::InternalError("クラウドプロバイダーレジストリが初期化されていません".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cloud_provider_type_display() {
        assert_eq!(format!("{}", CloudProviderType::AetherCloud), "Aether Cloud");
        assert_eq!(format!("{}", CloudProviderType::Google), "Google Drive");
        assert_eq!(format!("{}", CloudProviderType::Microsoft), "Microsoft OneDrive");
        assert_eq!(format!("{}", CloudProviderType::Amazon), "Amazon Drive");
        assert_eq!(format!("{}", CloudProviderType::Dropbox), "Dropbox");
        assert_eq!(format!("{}", CloudProviderType::Apple), "Apple iCloud");
        assert_eq!(format!("{}", CloudProviderType::Custom), "Custom Provider");
    }
    
    #[test]
    fn test_cloud_file_with_methods() {
        let file = CloudFile::new("123", "test.txt", "/path/to/test.txt", FileType::File, 1024)
            .with_mime_type("text/plain")
            .with_created_at(chrono::Utc::now())
            .with_metadata("owner", "user1");
        
        assert_eq!(file.id, "123");
        assert_eq!(file.name, "test.txt");
        assert_eq!(file.path, "/path/to/test.txt");
        assert_eq!(file.file_type, FileType::File);
        assert_eq!(file.size, 1024);
        assert_eq!(file.mime_type, Some("text/plain".to_string()));
        assert!(file.created_at.is_some());
        assert_eq!(file.metadata.get("owner"), Some(&"user1".to_string()));
    }
    
    #[test]
    fn test_cloud_credentials_with_methods() {
        let creds = CloudCredentials::new("oauth2")
            .with_client_id("client123")
            .with_client_secret("secret456")
            .with_access_token("token789")
            .with_refresh_token("refresh012")
            .with_param("scope", "read write");
        
        assert_eq!(creds.auth_type, "oauth2");
        assert_eq!(creds.client_id, Some("client123".to_string()));
        assert_eq!(creds.client_secret, Some("secret456".to_string()));
        assert_eq!(creds.access_token, Some("token789".to_string()));
        assert_eq!(creds.refresh_token, Some("refresh012".to_string()));
        assert_eq!(creds.additional_params.get("scope"), Some(&"read write".to_string()));
    }
    
    #[test]
    fn test_cloud_provider_config_with_methods() {
        let config = CloudProviderConfig::new(CloudProviderType::Google, "My Google Drive")
            .with_base_url("https://drive.google.com/api")
            .with_timeout(120)
            .with_auto_sync(true, 30)
            .add_sync_directory("/home/user/Documents")
            .add_ignore_pattern("*.tmp");
        
        assert_eq!(config.provider_type, CloudProviderType::Google);
        assert_eq!(config.name, "My Google Drive");
        assert_eq!(config.base_url, Some("https://drive.google.com/api".to_string()));
        assert_eq!(config.timeout_sec, 120);
        assert!(config.auto_sync_enabled);
        assert_eq!(config.auto_sync_interval_min, 30);
        assert_eq!(config.sync_directories, vec!["/home/user/Documents"]);
        assert_eq!(config.ignore_patterns, vec!["*.tmp"]);
    }
} 