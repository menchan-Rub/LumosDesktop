// LumosDesktop Security Permission Module
//
// このモジュールはアプリケーションの権限を管理します。
// 権限の定義、権限セット、権限マネージャーなどを提供します。

use std::collections::HashSet;
use std::sync::RwLock;
use std::hash::{Hash, Hasher};
use crate::core::system::logging;

use super::SecurityLevel;

/// システム権限を表す列挙型
#[derive(Debug, Clone, Eq)]
pub enum Permission {
    // ファイルシステム関連
    FileRead,
    FileWrite,
    FileDelete,
    FileExecute,
    
    // ネットワーク関連
    NetworkAccess,
    NetworkListen,
    NetworkConnect,
    
    // プロセス関連
    ProcessCreate,
    ProcessKill,
    ProcessInfo,
    
    // システム関連
    SystemInfo,
    SystemConfig,
    SystemShutdown,
    
    // デバイス関連
    DeviceAccess,
    DeviceControl,
    
    // ユーザーデータ関連
    UserDataRead,
    UserDataWrite,
    
    // 通知関連
    NotificationSend,
    NotificationReceive,
    
    // カスタム権限（プラグイン用）
    Custom(String),
}

impl PartialEq for Permission {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Permission::Custom(a), Permission::Custom(b)) => a == b,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

impl Hash for Permission {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Permission::Custom(s) => {
                // カスタム権限の場合は文字列をハッシュ化
                13u8.hash(state);
                s.hash(state);
            }
            _ => {
                // 通常権限の場合は識別子をハッシュ化
                std::mem::discriminant(self).hash(state);
            }
        }
    }
}

impl Permission {
    /// 権限の文字列表現を取得
    pub fn as_str(&self) -> &str {
        match self {
            Permission::FileRead => "file:read",
            Permission::FileWrite => "file:write",
            Permission::FileDelete => "file:delete",
            Permission::FileExecute => "file:execute",
            
            Permission::NetworkAccess => "network:access",
            Permission::NetworkListen => "network:listen",
            Permission::NetworkConnect => "network:connect",
            
            Permission::ProcessCreate => "process:create",
            Permission::ProcessKill => "process:kill",
            Permission::ProcessInfo => "process:info",
            
            Permission::SystemInfo => "system:info",
            Permission::SystemConfig => "system:config",
            Permission::SystemShutdown => "system:shutdown",
            
            Permission::DeviceAccess => "device:access",
            Permission::DeviceControl => "device:control",
            
            Permission::UserDataRead => "userdata:read",
            Permission::UserDataWrite => "userdata:write",
            
            Permission::NotificationSend => "notification:send",
            Permission::NotificationReceive => "notification:receive",
            
            Permission::Custom(s) => s,
        }
    }
    
    /// 文字列から権限を作成
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "file:read" => Some(Permission::FileRead),
            "file:write" => Some(Permission::FileWrite),
            "file:delete" => Some(Permission::FileDelete),
            "file:execute" => Some(Permission::FileExecute),
            
            "network:access" => Some(Permission::NetworkAccess),
            "network:listen" => Some(Permission::NetworkListen),
            "network:connect" => Some(Permission::NetworkConnect),
            
            "process:create" => Some(Permission::ProcessCreate),
            "process:kill" => Some(Permission::ProcessKill),
            "process:info" => Some(Permission::ProcessInfo),
            
            "system:info" => Some(Permission::SystemInfo),
            "system:config" => Some(Permission::SystemConfig),
            "system:shutdown" => Some(Permission::SystemShutdown),
            
            "device:access" => Some(Permission::DeviceAccess),
            "device:control" => Some(Permission::DeviceControl),
            
            "userdata:read" => Some(Permission::UserDataRead),
            "userdata:write" => Some(Permission::UserDataWrite),
            
            "notification:send" => Some(Permission::NotificationSend),
            "notification:receive" => Some(Permission::NotificationReceive),
            
            _ => {
                if s.starts_with("custom:") {
                    Some(Permission::Custom(s.to_string()))
                } else {
                    None
                }
            }
        }
    }
    
    /// 権限の説明を取得
    pub fn description(&self) -> &str {
        match self {
            Permission::FileRead => "ファイルの読み取り権限",
            Permission::FileWrite => "ファイルの書き込み権限",
            Permission::FileDelete => "ファイルの削除権限",
            Permission::FileExecute => "ファイルの実行権限",
            
            Permission::NetworkAccess => "ネットワークへのアクセス権限",
            Permission::NetworkListen => "ネットワークポートのリッスン権限",
            Permission::NetworkConnect => "ネットワーク接続の権限",
            
            Permission::ProcessCreate => "プロセス作成の権限",
            Permission::ProcessKill => "プロセス終了の権限",
            Permission::ProcessInfo => "プロセス情報取得の権限",
            
            Permission::SystemInfo => "システム情報取得の権限",
            Permission::SystemConfig => "システム設定の権限",
            Permission::SystemShutdown => "システムシャットダウンの権限",
            
            Permission::DeviceAccess => "デバイスアクセスの権限",
            Permission::DeviceControl => "デバイス制御の権限",
            
            Permission::UserDataRead => "ユーザーデータ読み取りの権限",
            Permission::UserDataWrite => "ユーザーデータ書き込みの権限",
            
            Permission::NotificationSend => "通知送信の権限",
            Permission::NotificationReceive => "通知受信の権限",
            
            Permission::Custom(_) => "カスタム権限",
        }
    }
    
    /// この権限が危険かどうかを判定
    pub fn is_dangerous(&self) -> bool {
        match self {
            Permission::FileWrite | 
            Permission::FileDelete | 
            Permission::FileExecute |
            Permission::ProcessCreate | 
            Permission::ProcessKill |
            Permission::SystemConfig | 
            Permission::SystemShutdown |
            Permission::DeviceControl |
            Permission::UserDataWrite => true,
            _ => false,
        }
    }
}

/// 権限セット
#[derive(Debug, Clone)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// 新しい権限セットを作成
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }
    
    /// 権限を追加
    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }
    
    /// 権限を削除
    pub fn remove(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }
    
    /// 権限を持っているか確認
    pub fn has(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
    
    /// すべての権限を取得
    pub fn all(&self) -> Vec<Permission> {
        self.permissions.iter().cloned().collect()
    }
    
    /// 権限セットが空かどうか
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }
    
    /// 権限の数を取得
    pub fn len(&self) -> usize {
        self.permissions.len()
    }
    
    /// 権限セットのマージ
    pub fn merge(&mut self, other: &PermissionSet) {
        for perm in &other.permissions {
            self.permissions.insert(perm.clone());
        }
    }
    
    /// 権限セットの差分
    pub fn difference(&self, other: &PermissionSet) -> PermissionSet {
        let mut result = PermissionSet::new();
        for perm in &self.permissions {
            if !other.has(perm) {
                result.add(perm.clone());
            }
        }
        result
    }
    
    /// 危険な権限のみを抽出
    pub fn dangerous_permissions(&self) -> PermissionSet {
        let mut result = PermissionSet::new();
        for perm in &self.permissions {
            if perm.is_dangerous() {
                result.add(perm.clone());
            }
        }
        result
    }
}

/// 権限マネージャー
pub struct PermissionManager {
    default_permissions: RwLock<Vec<(SecurityLevel, PermissionSet)>>,
}

impl PermissionManager {
    /// 新しい権限マネージャーを作成
    pub fn new() -> Self {
        let logger = logging::get_logger("permission_manager");
        logging::debug!(logger, "PermissionManagerを初期化中...");
        
        let mut manager = Self {
            default_permissions: RwLock::new(Vec::new()),
        };
        
        // デフォルト権限を初期化
        let _ = manager.initialize_default_permissions();
        
        manager
    }
    
    /// デフォルト権限を初期化
    fn initialize_default_permissions(&self) -> Result<(), String> {
        let logger = logging::get_logger("permission_manager");
        logging::info!(logger, "デフォルト権限を初期化中...");
        
        let mut defaults = self.default_permissions.write().map_err(|_| {
            "デフォルト権限へのアクセス中にエラーが発生しました".to_string()
        })?;
        
        // ルートレベルの権限（すべての権限）
        let mut root_perms = PermissionSet::new();
        root_perms.add(Permission::FileRead);
        root_perms.add(Permission::FileWrite);
        root_perms.add(Permission::FileDelete);
        root_perms.add(Permission::FileExecute);
        root_perms.add(Permission::NetworkAccess);
        root_perms.add(Permission::NetworkListen);
        root_perms.add(Permission::NetworkConnect);
        root_perms.add(Permission::ProcessCreate);
        root_perms.add(Permission::ProcessKill);
        root_perms.add(Permission::ProcessInfo);
        root_perms.add(Permission::SystemInfo);
        root_perms.add(Permission::SystemConfig);
        root_perms.add(Permission::SystemShutdown);
        root_perms.add(Permission::DeviceAccess);
        root_perms.add(Permission::DeviceControl);
        root_perms.add(Permission::UserDataRead);
        root_perms.add(Permission::UserDataWrite);
        root_perms.add(Permission::NotificationSend);
        root_perms.add(Permission::NotificationReceive);
        
        // 管理者レベルの権限
        let mut admin_perms = PermissionSet::new();
        admin_perms.add(Permission::FileRead);
        admin_perms.add(Permission::FileWrite);
        admin_perms.add(Permission::FileDelete);
        admin_perms.add(Permission::NetworkAccess);
        admin_perms.add(Permission::NetworkListen);
        admin_perms.add(Permission::ProcessCreate);
        admin_perms.add(Permission::ProcessInfo);
        admin_perms.add(Permission::SystemInfo);
        admin_perms.add(Permission::SystemConfig);
        admin_perms.add(Permission::DeviceAccess);
        admin_perms.add(Permission::UserDataRead);
        admin_perms.add(Permission::UserDataWrite);
        admin_perms.add(Permission::NotificationSend);
        admin_perms.add(Permission::NotificationReceive);
        
        // 通常レベルの権限
        let mut normal_perms = PermissionSet::new();
        normal_perms.add(Permission::FileRead);
        normal_perms.add(Permission::NetworkAccess);
        normal_perms.add(Permission::NetworkConnect);
        normal_perms.add(Permission::ProcessInfo);
        normal_perms.add(Permission::SystemInfo);
        normal_perms.add(Permission::DeviceAccess);
        normal_perms.add(Permission::UserDataRead);
        normal_perms.add(Permission::NotificationSend);
        normal_perms.add(Permission::NotificationReceive);
        
        // 制限レベルの権限
        let mut restricted_perms = PermissionSet::new();
        restricted_perms.add(Permission::FileRead);
        restricted_perms.add(Permission::NetworkConnect);
        restricted_perms.add(Permission::SystemInfo);
        restricted_perms.add(Permission::NotificationReceive);
        
        // セキュリティレベルに応じたデフォルト権限を設定
        defaults.push((SecurityLevel::Root, root_perms));
        defaults.push((SecurityLevel::Admin, admin_perms));
        defaults.push((SecurityLevel::Normal, normal_perms));
        defaults.push((SecurityLevel::Restricted, restricted_perms));
        
        logging::info!(logger, "デフォルト権限の初期化が完了しました");
        Ok(())
    }
    
    /// デフォルト権限を取得
    pub fn get_default_permissions(&self, level: SecurityLevel) -> PermissionSet {
        let defaults = self.default_permissions.read().unwrap_or_else(|_| {
            let logger = logging::get_logger("permission_manager");
            logging::error!(logger, "デフォルト権限へのアクセス中にエラーが発生しました");
            panic!("デフォルト権限へのアクセス中にエラーが発生しました");
        });
        
        for (sec_level, perms) in defaults.iter() {
            if *sec_level == level {
                return perms.clone();
            }
        }
        
        // 該当するレベルが見つからない場合は制限レベルの権限を返す
        for (sec_level, perms) in defaults.iter() {
            if *sec_level == SecurityLevel::Restricted {
                return perms.clone();
            }
        }
        
        // 制限レベルも見つからない場合は空の権限セットを返す
        PermissionSet::new()
    }
    
    /// カスタム権限セットを登録
    pub fn register_custom_permission_set(
        &self,
        level: SecurityLevel,
        permissions: PermissionSet
    ) -> Result<(), String> {
        let mut defaults = self.default_permissions.write().map_err(|_| {
            "デフォルト権限へのアクセス中にエラーが発生しました".to_string()
        })?;
        
        // 既存のレベルを上書き
        for i in 0..defaults.len() {
            if defaults[i].0 == level {
                defaults[i].1 = permissions;
                return Ok(());
            }
        }
        
        // 該当するレベルがない場合は追加
        defaults.push((level, permissions));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_equality() {
        assert_eq!(Permission::FileRead, Permission::FileRead);
        assert_ne!(Permission::FileRead, Permission::FileWrite);
        
        assert_eq!(
            Permission::Custom("custom:test".to_string()),
            Permission::Custom("custom:test".to_string())
        );
        assert_ne!(
            Permission::Custom("custom:test".to_string()),
            Permission::Custom("custom:other".to_string())
        );
    }
    
    #[test]
    fn test_permission_set_operations() {
        let mut set = PermissionSet::new();
        assert_eq!(set.len(), 0);
        
        set.add(Permission::FileRead);
        assert_eq!(set.len(), 1);
        assert!(set.has(&Permission::FileRead));
        assert!(!set.has(&Permission::FileWrite));
        
        set.add(Permission::FileWrite);
        assert_eq!(set.len(), 2);
        assert!(set.has(&Permission::FileWrite));
        
        set.remove(&Permission::FileRead);
        assert_eq!(set.len(), 1);
        assert!(!set.has(&Permission::FileRead));
    }
    
    #[test]
    fn test_permission_set_merge() {
        let mut set1 = PermissionSet::new();
        set1.add(Permission::FileRead);
        set1.add(Permission::FileWrite);
        
        let mut set2 = PermissionSet::new();
        set2.add(Permission::NetworkAccess);
        set2.add(Permission::FileRead);
        
        set1.merge(&set2);
        assert_eq!(set1.len(), 3);
        assert!(set1.has(&Permission::FileRead));
        assert!(set1.has(&Permission::FileWrite));
        assert!(set1.has(&Permission::NetworkAccess));
    }
    
    #[test]
    fn test_permission_manager() {
        let manager = PermissionManager::new();
        
        let root_perms = manager.get_default_permissions(SecurityLevel::Root);
        let normal_perms = manager.get_default_permissions(SecurityLevel::Normal);
        
        assert!(root_perms.has(&Permission::SystemShutdown));
        assert!(!normal_perms.has(&Permission::SystemShutdown));
        
        assert!(root_perms.has(&Permission::FileWrite));
        assert!(!normal_perms.has(&Permission::FileWrite));
    }
} 