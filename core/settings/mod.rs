// LumosDesktop 設定モジュール
// アプリケーション設定の管理と永続化を担当

//! # 設定モジュール
//!
//! このモジュールはLumosDesktopの設定管理システムを提供します。
//! ユーザー設定、アプリケーション設定、システム設定を管理し、
//! 永続化と同期を行います。
//!
//! 主な機能：
//! - 設定の読み書きと型安全なアクセス
//! - スキーマベースの設定検証
//! - 設定の変更監視と通知
//! - プロファイル管理（複数ユーザー、コンテキスト対応設定）
//! - 設定の同期（デバイス間、クラウド）
//! - デフォルト値と継承メカニズム

pub mod registry;
pub mod profile_manager;
pub mod schema;
pub mod sync_agent;

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::error::Error;
use std::fmt;
use std::io;
use std::fs;

// 主要なモジュールの公開型をre-export
pub use registry::{
    SettingsRegistry,
    SettingsKey,
    SettingsValue,
    SettingsPath,
    SettingsNode,
    SettingsTransaction
};

pub use profile_manager::{
    ProfileManager,
    UserProfile,
    ProfileId,
    ProfileType,
    ProfileMetadata
};

pub use schema::{
    SchemaManager,
    SettingsSchema,
    SchemaType,
    SchemaConstraint,
    ValidationResult
};

pub use sync_agent::{
    SyncAgent,
    SyncStatus,
    SyncDirection,
    SyncProvider,
    SyncConflict,
    SyncResult
};

/// 設定モジュールのエラー型
#[derive(Debug)]
pub enum SettingsError {
    /// I/Oエラー
    Io(io::Error),
    /// キーが見つからない
    KeyNotFound(String),
    /// 型エラー
    TypeError(String),
    /// 検証エラー
    ValidationError(String),
    /// プロファイルエラー
    ProfileError(String),
    /// 同期エラー
    SyncError(String),
    /// スキーマエラー
    SchemaError(String),
    /// パーミッションエラー
    PermissionDenied(String),
    /// その他のエラー
    Other(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::Io(err) => write!(f, "I/Oエラー: {}", err),
            SettingsError::KeyNotFound(key) => write!(f, "設定キーが見つかりません: {}", key),
            SettingsError::TypeError(msg) => write!(f, "型エラー: {}", msg),
            SettingsError::ValidationError(msg) => write!(f, "検証エラー: {}", msg),
            SettingsError::ProfileError(msg) => write!(f, "プロファイルエラー: {}", msg),
            SettingsError::SyncError(msg) => write!(f, "同期エラー: {}", msg),
            SettingsError::SchemaError(msg) => write!(f, "スキーマエラー: {}", msg),
            SettingsError::PermissionDenied(msg) => write!(f, "権限エラー: {}", msg),
            SettingsError::Other(msg) => write!(f, "設定エラー: {}", msg),
        }
    }
}

impl Error for SettingsError {}

impl From<io::Error> for SettingsError {
    fn from(error: io::Error) -> Self {
        SettingsError::Io(error)
    }
}

/// 設定変更イベント
#[derive(Debug, Clone)]
pub struct SettingsChangeEvent {
    /// 変更された設定のパス
    pub path: String,
    /// 古い値
    pub old_value: Option<SettingsValue>,
    /// 新しい値
    pub new_value: Option<SettingsValue>,
    /// 変更のタイムスタンプ
    pub timestamp: std::time::SystemTime,
    /// 変更の発生源
    pub source: ChangeSource,
}

/// 設定変更の発生源
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeSource {
    /// ユーザーによる変更
    User,
    /// アプリケーションによる変更
    Application,
    /// システムによる変更
    System,
    /// 同期による変更
    Sync,
    /// リセットによる変更
    Reset,
}

/// 設定変更リスナーのコールバック型
pub type SettingsChangeListener = Box<dyn Fn(&SettingsChangeEvent) + Send + Sync>;

/// 設定権限レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingsPermission {
    /// 読み取りのみ
    Read,
    /// 読み書き
    ReadWrite,
    /// 管理（スキーマ変更、削除など）
    Admin,
}

/// 設定マネージャーの設定
#[derive(Debug, Clone)]
pub struct SettingsManagerConfig {
    /// 設定ファイルのベースディレクトリ
    pub base_directory: PathBuf,
    /// デフォルト設定ファイル
    pub default_settings_file: PathBuf,
    /// スキーマディレクトリ
    pub schema_directory: PathBuf,
    /// プロファイルディレクトリ
    pub profiles_directory: PathBuf,
    /// 自動保存の間隔（None の場合は即時保存）
    pub auto_save_interval: Option<Duration>,
    /// 変更通知を有効化
    pub enable_notifications: bool,
    /// 同期を有効化
    pub enable_sync: bool,
    /// バックアップを有効化
    pub enable_backups: bool,
    /// バックアップの保持数
    pub backup_count: usize,
}

impl Default for SettingsManagerConfig {
    fn default() -> Self {
        Self {
            base_directory: PathBuf::from("./config"),
            default_settings_file: PathBuf::from("./config/defaults.json"),
            schema_directory: PathBuf::from("./config/schema"),
            profiles_directory: PathBuf::from("./config/profiles"),
            auto_save_interval: Some(Duration::from_secs(5)),
            enable_notifications: true,
            enable_sync: false,
            enable_backups: true,
            backup_count: 5,
        }
    }
}

/// 設定マネージャー
///
/// アプリケーション全体の設定を管理し、
/// レジストリ、プロファイル、スキーマ、同期の各コンポーネントを調整します。
pub struct SettingsManager {
    /// 設定
    config: SettingsManagerConfig,
    /// 設定レジストリ
    registry: Arc<RwLock<registry::SettingsRegistry>>,
    /// プロファイルマネージャー
    profile_manager: Arc<RwLock<profile_manager::ProfileManager>>,
    /// スキーママネージャー
    schema_manager: Arc<RwLock<schema::SchemaManager>>,
    /// 同期エージェント
    sync_agent: Option<Arc<RwLock<sync_agent::SyncAgent>>>,
    /// 変更リスナー
    change_listeners: Arc<RwLock<HashMap<String, Vec<SettingsChangeListener>>>>,
    /// 自動保存タイマー
    last_save_time: Arc<Mutex<Instant>>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl SettingsManager {
    /// 新しい設定マネージャーを作成
    pub fn new() -> Self {
        let config = SettingsManagerConfig::default();
        Self {
            config: config.clone(),
            registry: Arc::new(RwLock::new(registry::SettingsRegistry::new())),
            profile_manager: Arc::new(RwLock::new(profile_manager::ProfileManager::new(&config.profiles_directory))),
            schema_manager: Arc::new(RwLock::new(schema::SchemaManager::new(&config.schema_directory))),
            sync_agent: None,
            change_listeners: Arc::new(RwLock::new(HashMap::new())),
            last_save_time: Arc::new(Mutex::new(Instant::now())),
            initialized: false,
        }
    }

    /// 設定を指定して新しい設定マネージャーを作成
    pub fn with_config(config: SettingsManagerConfig) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry::SettingsRegistry::new())),
            profile_manager: Arc::new(RwLock::new(profile_manager::ProfileManager::new(&config.profiles_directory))),
            schema_manager: Arc::new(RwLock::new(schema::SchemaManager::new(&config.schema_directory))),
            sync_agent: None,
            change_listeners: Arc::new(RwLock::new(HashMap::new())),
            last_save_time: Arc::new(Mutex::new(Instant::now())),
            config,
            initialized: false,
        }
    }

    /// 設定マネージャーを初期化
    pub fn initialize(&mut self) -> Result<(), SettingsError> {
        if self.initialized {
            return Ok(());
        }

        // ディレクトリを準備
        fs::create_dir_all(&self.config.base_directory)
            .map_err(|e| SettingsError::Io(e))?;
        fs::create_dir_all(&self.config.schema_directory)
            .map_err(|e| SettingsError::Io(e))?;
        fs::create_dir_all(&self.config.profiles_directory)
            .map_err(|e| SettingsError::Io(e))?;

        // レジストリを初期化
        {
            let mut registry = self.registry.write().unwrap();
            registry.initialize()?;
            if self.config.default_settings_file.exists() {
                registry.load_defaults(&self.config.default_settings_file)?;
            }
        }

        // スキーママネージャーを初期化
        {
            let mut schema_manager = self.schema_manager.write().unwrap();
            schema_manager.initialize()?;
        }

        // プロファイルマネージャーを初期化
        {
            let mut profile_manager = self.profile_manager.write().unwrap();
            profile_manager.initialize()?;
        }

        // 同期エージェントを初期化（有効な場合）
        if self.config.enable_sync {
            let sync_agent = Arc::new(RwLock::new(sync_agent::SyncAgent::new()));
            {
                let mut agent = sync_agent.write().unwrap();
                agent.initialize()?;
            }
            self.sync_agent = Some(sync_agent);
        }

        // 自動保存タイマーを開始（設定されている場合）
        if let Some(interval) = self.config.auto_save_interval {
            let registry_clone = Arc::clone(&self.registry);
            let last_save_time_clone = Arc::clone(&self.last_save_time);
            
            // 自動保存スレッドの起動
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_millis(500));
                    
                    let now = Instant::now();
                    let should_save = {
                        let last_save = last_save_time_clone.lock().unwrap();
                        now.duration_since(*last_save) >= interval
                    };
                    
                    if should_save {
                        let registry = registry_clone.read().unwrap();
                        if registry.is_dirty() {
                            match registry.save() {
                                Ok(_) => {
                                    let mut last_save = last_save_time_clone.lock().unwrap();
                                    *last_save = now;
                                }
                                Err(e) => {
                                    eprintln!("設定の自動保存に失敗しました: {}", e);
                                }
                            }
                        }
                    }
                }
            });
        }

        self.initialized = true;
        Ok(())
    }

    /// 設定値を取得
    pub fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, SettingsError> {
        // アクティブなプロファイルから取得を試みる
        let profile_manager = self.profile_manager.read().unwrap();
        if let Some(active_profile_id) = profile_manager.get_active_profile_id() {
            if let Ok(value) = profile_manager.get_profile_setting::<T>(active_profile_id, path) {
                return Ok(value);
            }
        }
        
        // プロファイルになければレジストリから取得
        let registry = self.registry.read().unwrap();
        registry.get(path)
    }

    /// 設定値を設定
    pub fn set<T: serde::Serialize>(&self, path: &str, value: T) -> Result<(), SettingsError> {
        if !self.initialized {
            return Err(SettingsError::Other("設定マネージャーが初期化されていません".to_string()));
        }

        // 値を検証
        {
            let schema_manager = self.schema_manager.read().unwrap();
            schema_manager.validate_value(path, &value)?;
        }

        // 古い値を保存
        let old_value = {
            let registry = self.registry.read().unwrap();
            registry.get_raw(path).ok()
        };

        // アクティブなプロファイルがあれば、そこに設定
        let profile_updated = {
            let profile_manager = self.profile_manager.read().unwrap();
            if let Some(active_profile_id) = profile_manager.get_active_profile_id() {
                profile_manager.set_profile_setting(active_profile_id, path, &value).is_ok()
            } else {
                false
            }
        };

        // プロファイルがなければレジストリに直接設定
        if !profile_updated {
            let mut registry = self.registry.write().unwrap();
            registry.set(path, value)?;
        }

        // 変更通知を送信
        if self.config.enable_notifications {
            let new_value = {
                let registry = self.registry.read().unwrap();
                registry.get_raw(path).ok()
            };

            let event = SettingsChangeEvent {
                path: path.to_string(),
                old_value,
                new_value,
                timestamp: std::time::SystemTime::now(),
                source: ChangeSource::User,
            };

            self.notify_change(&event);
        }

        // 即時保存（自動保存が無効の場合）
        if self.config.auto_save_interval.is_none() {
            self.save()?;
        } else {
            // 最終保存時間を更新しない（自動保存タイマーに任せる）
        }

        Ok(())
    }

    /// 設定を保存
    pub fn save(&self) -> Result<(), SettingsError> {
        if !self.initialized {
            return Err(SettingsError::Other("設定マネージャーが初期化されていません".to_string()));
        }

        // レジストリを保存
        {
            let registry = self.registry.read().unwrap();
            registry.save()?;
        }

        // プロファイルを保存
        {
            let profile_manager = self.profile_manager.read().unwrap();
            profile_manager.save_all()?;
        }

        // 最終保存時間を更新
        {
            let mut last_save = self.last_save_time.lock().unwrap();
            *last_save = Instant::now();
        }

        Ok(())
    }

    /// 設定値をリセット
    pub fn reset(&self, path: &str) -> Result<(), SettingsError> {
        if !self.initialized {
            return Err(SettingsError::Other("設定マネージャーが初期化されていません".to_string()));
        }

        // 古い値を保存
        let old_value = {
            let registry = self.registry.read().unwrap();
            registry.get_raw(path).ok()
        };

        // アクティブなプロファイルの設定をリセット
        let profile_reset = {
            let profile_manager = self.profile_manager.read().unwrap();
            if let Some(active_profile_id) = profile_manager.get_active_profile_id() {
                profile_manager.reset_profile_setting(active_profile_id, path).is_ok()
            } else {
                false
            }
        };

        // プロファイルがなければレジストリの値をリセット
        if !profile_reset {
            let mut registry = self.registry.write().unwrap();
            registry.reset(path)?;
        }

        // 変更通知を送信
        if self.config.enable_notifications {
            let new_value = {
                let registry = self.registry.read().unwrap();
                registry.get_raw(path).ok()
            };

            let event = SettingsChangeEvent {
                path: path.to_string(),
                old_value,
                new_value,
                timestamp: std::time::SystemTime::now(),
                source: ChangeSource::Reset,
            };

            self.notify_change(&event);
        }

        Ok(())
    }

    /// 設定変更リスナーを追加
    pub fn add_change_listener(&self, path: &str, listener: SettingsChangeListener) -> String {
        let listener_id = format!("{}", uuid::Uuid::new_v4());
        let mut listeners = self.change_listeners.write().unwrap();
        
        let path_listeners = listeners.entry(path.to_string()).or_insert_with(Vec::new);
        path_listeners.push(listener);
        
        listener_id
    }

    /// 設定変更リスナーを削除
    pub fn remove_change_listener(&self, _listener_id: &str) -> Result<(), SettingsError> {
        // 実際の実装では、リスナーIDからリスナーを特定して削除する
        // 現在の実装では簡略化のためダミー実装
        Err(SettingsError::Other("未実装".to_string()))
    }

    /// 変更通知を送信
    fn notify_change(&self, event: &SettingsChangeEvent) {
        let listeners = self.change_listeners.read().unwrap();
        
        // 完全一致するパスのリスナーに通知
        if let Some(path_listeners) = listeners.get(&event.path) {
            for listener in path_listeners {
                listener(event);
            }
        }
        
        // ワイルドカードリスナーにも通知
        if let Some(wild_listeners) = listeners.get("*") {
            for listener in wild_listeners {
                listener(event);
            }
        }
    }

    /// スキーママネージャーを取得
    pub fn get_schema_manager(&self) -> Arc<RwLock<schema::SchemaManager>> {
        Arc::clone(&self.schema_manager)
    }

    /// プロファイルマネージャーを取得
    pub fn get_profile_manager(&self) -> Arc<RwLock<profile_manager::ProfileManager>> {
        Arc::clone(&self.profile_manager)
    }

    /// 同期エージェントを取得
    pub fn get_sync_agent(&self) -> Option<Arc<RwLock<sync_agent::SyncAgent>>> {
        self.sync_agent.clone()
    }

    /// レジストリを取得
    pub fn get_registry(&self) -> Arc<RwLock<registry::SettingsRegistry>> {
        Arc::clone(&self.registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_settings_manager_creation() {
        let manager = SettingsManager::new();
        assert!(!manager.initialized);
    }
    
    #[test]
    fn test_settings_error_display() {
        let err = SettingsError::KeyNotFound("test.key".to_string());
        assert_eq!(format!("{}", err), "設定キーが見つかりません: test.key");
        
        let err = SettingsError::TypeError("int -> string".to_string());
        assert_eq!(format!("{}", err), "型エラー: int -> string");
    }
    
    #[test]
    fn test_settings_permission() {
        assert!(SettingsPermission::Admin > SettingsPermission::ReadWrite);
        assert!(SettingsPermission::ReadWrite > SettingsPermission::Read);
    }
    
    #[test]
    fn test_settings_manager_config_default() {
        let config = SettingsManagerConfig::default();
        assert_eq!(config.base_directory, PathBuf::from("./config"));
        assert_eq!(config.backup_count, 5);
        assert!(config.enable_notifications);
        assert!(!config.enable_sync);
    }
} 