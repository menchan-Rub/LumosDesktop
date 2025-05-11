// LumosDesktop 設定同期エージェント
// 設定の同期と複数デバイス間の設定共有を担当

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant, SystemTime};
use serde::{Serialize, Deserialize};

use crate::core::settings::SettingsError;
use crate::core::settings::registry::{SettingsRegistry, SettingsValue};

/// 同期ステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// 同期中
    Syncing,
    /// アイドル状態
    Idle,
    /// 一時停止
    Paused,
    /// エラー状態
    Error,
    /// 競合解決待ち
    ConflictResolutionPending,
}

/// 同期方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    /// 双方向同期
    Bidirectional,
    /// アップロードのみ
    UploadOnly,
    /// ダウンロードのみ
    DownloadOnly,
}

/// 同期プロバイダ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncProvider {
    /// ファイルシステム
    FileSystem(PathBuf),
    /// クラウドサービス
    Cloud {
        /// サービス名
        service: String,
        /// 認証トークン
        auth_token: String,
        /// エンドポイントURL
        endpoint: String,
    },
    /// P2P同期
    P2P {
        /// ピア識別子
        peer_id: String,
        /// 接続情報
        connection_info: String,
    },
}

/// 同期設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 同期を有効化
    pub enabled: bool,
    /// 同期プロバイダ
    pub provider: SyncProvider,
    /// 同期方向
    pub direction: SyncDirection,
    /// 自動同期の有効化
    pub auto_sync: bool,
    /// 自動同期の間隔（秒）
    pub auto_sync_interval: u64,
    /// 同期する設定パスのリスト（空の場合はすべて）
    pub include_paths: Vec<String>,
    /// 同期から除外する設定パスのリスト
    pub exclude_paths: Vec<String>,
    /// 同期時に暗号化を使用
    pub use_encryption: bool,
    /// 暗号化キー（使用する場合）
    pub encryption_key: Option<String>,
    /// 競合時のデフォルト解決方法
    pub default_conflict_resolution: ConflictResolution,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: SyncProvider::FileSystem(PathBuf::from("./sync")),
            direction: SyncDirection::Bidirectional,
            auto_sync: false,
            auto_sync_interval: 3600, // 1時間
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
            use_encryption: false,
            encryption_key: None,
            default_conflict_resolution: ConflictResolution::AskUser,
        }
    }
}

/// 同期競合
#[derive(Debug, Clone)]
pub struct SyncConflict {
    /// 競合した設定パス
    pub path: String,
    /// ローカルの値
    pub local_value: SettingsValue,
    /// リモートの値
    pub remote_value: SettingsValue,
    /// ローカルの最終更新時間
    pub local_timestamp: SystemTime,
    /// リモートの最終更新時間
    pub remote_timestamp: SystemTime,
}

/// 競合解決方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// ローカル優先
    PreferLocal,
    /// リモート優先
    PreferRemote,
    /// 新しい方を優先
    PreferNewer,
    /// ユーザーに確認
    AskUser,
}

/// 同期メタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncMetadata {
    /// パス
    path: String,
    /// 最終同期時間
    last_sync: SystemTime,
    /// バージョン番号
    version: u64,
    /// ハッシュ値
    hash: String,
}

/// 同期結果
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// 成功フラグ
    pub success: bool,
    /// 同期した設定数
    pub items_synced: usize,
    /// 競合数
    pub conflicts: usize,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// 詳細ログ
    pub details: Vec<String>,
}

/// 同期エージェント
pub struct SyncAgent {
    /// 設定
    config: SyncConfig,
    /// 現在のステータス
    status: SyncStatus,
    /// 最終同期時間
    last_sync: Option<SystemTime>,
    /// 同期メタデータ
    metadata: HashMap<String, SyncMetadata>,
    /// 未解決の競合
    pending_conflicts: Vec<SyncConflict>,
    /// 自動同期タイマー
    auto_sync_timer: Option<Instant>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl SyncAgent {
    /// 新しい同期エージェントを作成
    pub fn new() -> Self {
        Self {
            config: SyncConfig::default(),
            status: SyncStatus::Idle,
            last_sync: None,
            metadata: HashMap::new(),
            pending_conflicts: Vec::new(),
            auto_sync_timer: None,
            initialized: false,
        }
    }
    
    /// 設定を指定して新しい同期エージェントを作成
    pub fn with_config(config: SyncConfig) -> Self {
        Self {
            config,
            status: SyncStatus::Idle,
            last_sync: None,
            metadata: HashMap::new(),
            pending_conflicts: Vec::new(),
            auto_sync_timer: None,
            initialized: false,
        }
    }
    
    /// 同期エージェントを初期化
    pub fn initialize(&mut self) -> Result<(), SettingsError> {
        if self.initialized {
            return Ok(());
        }
        
        // 同期ディレクトリが存在することを確認
        match &self.config.provider {
            SyncProvider::FileSystem(path) => {
                fs::create_dir_all(path)
                    .map_err(|e| SettingsError::Io(e))?;
                    
                // メタデータをロード
                self.load_metadata(path)?;
            },
            SyncProvider::Cloud { .. } => {
                // クラウドプロバイダの初期化（接続テストなど）
                // 実装は将来のリリースで追加予定
            },
            SyncProvider::P2P { .. } => {
                // P2P接続の初期化
                // 実装は将来のリリースで追加予定
            }
        }
        
        // 自動同期タイマーの設定
        if self.config.auto_sync && self.config.enabled {
            self.auto_sync_timer = Some(Instant::now());
        }
        
        self.initialized = true;
        self.status = SyncStatus::Idle;
        
        Ok(())
    }
    
    /// メタデータをロード
    fn load_metadata<P: AsRef<Path>>(&mut self, base_path: P) -> Result<(), SettingsError> {
        let metadata_path = base_path.as_ref().join("sync_metadata.json");
        
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path)
                .map_err(|e| SettingsError::Io(e))?;
                
            let metadata_list: Vec<SyncMetadata> = serde_json::from_str(&content)
                .map_err(|e| SettingsError::SyncError(format!("メタデータのパースエラー: {}", e)))?;
                
            for item in metadata_list {
                self.metadata.insert(item.path.clone(), item);
            }
        }
        
        Ok(())
    }
    
    /// メタデータを保存
    fn save_metadata<P: AsRef<Path>>(&self, base_path: P) -> Result<(), SettingsError> {
        let metadata_path = base_path.as_ref().join("sync_metadata.json");
        
        let metadata_list: Vec<SyncMetadata> = self.metadata.values().cloned().collect();
        let content = serde_json::to_string_pretty(&metadata_list)
            .map_err(|e| SettingsError::SyncError(format!("メタデータのシリアル化エラー: {}", e)))?;
            
        fs::write(&metadata_path, content)
            .map_err(|e| SettingsError::Io(e))?;
            
        Ok(())
    }
    
    /// 同期を実行
    pub fn sync(&mut self) -> Result<SyncResult, SettingsError> {
        if !self.initialized {
            return Err(SettingsError::Other("同期エージェントが初期化されていません".to_string()));
        }
        
        if !self.config.enabled {
            return Err(SettingsError::SyncError("同期が無効になっています".to_string()));
        }
        
        // 同期ステータスを更新
        self.status = SyncStatus::Syncing;
        
        // 結果オブジェクトを初期化
        let mut result = SyncResult {
            success: true,
            items_synced: 0,
            conflicts: 0,
            error_message: None,
            details: Vec::new(),
        };
        
        // プロバイダに応じた同期処理
        match &self.config.provider {
            SyncProvider::FileSystem(path) => {
                self.sync_with_filesystem(path, &mut result)?;
            },
            SyncProvider::Cloud { .. } => {
                result.error_message = Some("クラウド同期は現在実装されていません".to_string());
                result.success = false;
            },
            SyncProvider::P2P { .. } => {
                result.error_message = Some("P2P同期は現在実装されていません".to_string());
                result.success = false;
            }
        }
        
        // 同期完了後のステータス更新
        if result.conflicts > 0 {
            self.status = SyncStatus::ConflictResolutionPending;
        } else if !result.success {
            self.status = SyncStatus::Error;
        } else {
            self.status = SyncStatus::Idle;
            self.last_sync = Some(SystemTime::now());
            
            // 自動同期タイマーをリセット
            if self.config.auto_sync {
                self.auto_sync_timer = Some(Instant::now());
            }
        }
        
        Ok(result)
    }
    
    /// ファイルシステムとの同期
    fn sync_with_filesystem<P: AsRef<Path>>(&mut self, base_path: P, result: &mut SyncResult) -> Result<(), SettingsError> {
        // メタデータディレクトリを確保
        let sync_dir = base_path.as_ref().to_path_buf();
        let settings_dir = sync_dir.join("settings");
        
        fs::create_dir_all(&settings_dir)
            .map_err(|e| SettingsError::Io(e))?;
            
        // 同期方向に応じた処理
        match self.config.direction {
            SyncDirection::UploadOnly => {
                // アップロードのみ（ローカル→リモート）
                // result.details.push("アップロードのみモードで同期しています".to_string());
                // TODO: 実装
            },
            SyncDirection::DownloadOnly => {
                // ダウンロードのみ（リモート→ローカル）
                // result.details.push("ダウンロードのみモードで同期しています".to_string());
                // TODO: 実装
            },
            SyncDirection::Bidirectional => {
                // 双方向同期（マージ処理が必要）
                // result.details.push("双方向モードで同期しています".to_string());
                // TODO: 実装
            }
        }
        
        // 同期が成功したらメタデータを保存
        if result.success {
            self.save_metadata(&sync_dir)?;
        }
        
        Ok(())
    }
    
    /// 自動同期を実行（必要な場合）
    pub fn check_auto_sync(&mut self) -> Result<bool, SettingsError> {
        if !self.initialized || !self.config.enabled || !self.config.auto_sync {
            return Ok(false);
        }
        
        // 自動同期タイマーをチェック
        if let Some(last_time) = self.auto_sync_timer {
            let now = Instant::now();
            let elapsed = now.duration_since(last_time);
            let interval = Duration::from_secs(self.config.auto_sync_interval);
            
            if elapsed >= interval {
                // 自動同期を実行
                self.sync()?;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// 競合を解決
    pub fn resolve_conflict(&mut self, path: &str, resolution: ConflictResolution) -> Result<(), SettingsError> {
        if self.status != SyncStatus::ConflictResolutionPending {
            return Err(SettingsError::SyncError("解決すべき競合がありません".to_string()));
        }
        
        // 指定されたパスの競合を検索
        let conflict_index = self.pending_conflicts.iter().position(|c| c.path == path);
        
        if let Some(index) = conflict_index {
            let conflict = &self.pending_conflicts[index];
            
            // 解決方法に応じた処理
            match resolution {
                ConflictResolution::PreferLocal => {
                    // ローカルの値を使用（何もしない）
                },
                ConflictResolution::PreferRemote => {
                    // リモートの値を使用
                    // TODO: リモート値の適用処理
                },
                ConflictResolution::PreferNewer => {
                    // 新しい方を使用
                    if conflict.remote_timestamp > conflict.local_timestamp {
                        // リモートの方が新しい
                        // TODO: リモート値の適用処理
                    }
                },
                ConflictResolution::AskUser => {
                    // ユーザーに確認（既に選択されているはず）
                    return Err(SettingsError::SyncError("無効な解決方法: ユーザーに確認".to_string()));
                }
            }
            
            // 競合リストから削除
            self.pending_conflicts.remove(index);
            
            // すべての競合が解決された場合はステータスを更新
            if self.pending_conflicts.is_empty() {
                self.status = SyncStatus::Idle;
            }
            
            Ok(())
        } else {
            Err(SettingsError::SyncError(format!("指定されたパス'{}'の競合が見つかりません", path)))
        }
    }
    
    /// 同期を有効化
    pub fn enable(&mut self) {
        self.config.enabled = true;
        
        // 自動同期タイマーを設定
        if self.config.auto_sync {
            self.auto_sync_timer = Some(Instant::now());
        }
    }
    
    /// 同期を無効化
    pub fn disable(&mut self) {
        self.config.enabled = false;
        self.status = SyncStatus::Paused;
        self.auto_sync_timer = None;
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> &SyncConfig {
        &self.config
    }
    
    /// 設定を設定
    pub fn set_config(&mut self, config: SyncConfig) {
        let was_enabled = self.config.enabled && self.config.auto_sync;
        let will_be_enabled = config.enabled && config.auto_sync;
        
        self.config = config;
        
        // 自動同期のステータスが変わった場合はタイマーを更新
        if !was_enabled && will_be_enabled {
            self.auto_sync_timer = Some(Instant::now());
        } else if was_enabled && !will_be_enabled {
            self.auto_sync_timer = None;
        }
    }
    
    /// ステータスを取得
    pub fn get_status(&self) -> SyncStatus {
        self.status
    }
    
    /// 最終同期時間を取得
    pub fn get_last_sync_time(&self) -> Option<SystemTime> {
        self.last_sync
    }
    
    /// 未解決の競合を取得
    pub fn get_pending_conflicts(&self) -> &[SyncConflict] {
        &self.pending_conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_sync_agent_creation() {
        let agent = SyncAgent::new();
        assert!(!agent.config.enabled);
        assert_eq!(agent.status, SyncStatus::Idle);
        assert!(!agent.initialized);
    }
    
    #[test]
    fn test_sync_agent_with_config() {
        let config = SyncConfig {
            enabled: true,
            auto_sync: true,
            direction: SyncDirection::UploadOnly,
            ..Default::default()
        };
        
        let agent = SyncAgent::with_config(config);
        assert!(agent.config.enabled);
        assert!(agent.config.auto_sync);
        assert_eq!(agent.config.direction, SyncDirection::UploadOnly);
    }
    
    #[test]
    fn test_sync_agent_initialization() -> Result<(), SettingsError> {
        let dir = tempdir()?;
        let sync_path = dir.path().to_path_buf();
        
        let config = SyncConfig {
            enabled: true,
            provider: SyncProvider::FileSystem(sync_path),
            ..Default::default()
        };
        
        let mut agent = SyncAgent::with_config(config);
        agent.initialize()?;
        
        assert!(agent.initialized);
        assert_eq!(agent.status, SyncStatus::Idle);
        
        Ok(())
    }
    
    #[test]
    fn test_sync_agent_disable() -> Result<(), SettingsError> {
        let dir = tempdir()?;
        let sync_path = dir.path().to_path_buf();
        
        let config = SyncConfig {
            enabled: true,
            provider: SyncProvider::FileSystem(sync_path),
            ..Default::default()
        };
        
        let mut agent = SyncAgent::with_config(config);
        agent.initialize()?;
        
        assert!(agent.config.enabled);
        
        agent.disable();
        
        assert!(!agent.config.enabled);
        assert_eq!(agent.status, SyncStatus::Paused);
        
        Ok(())
    }
} 