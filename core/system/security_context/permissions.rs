//! 権限管理モジュール
//!
//! このモジュールは、アプリケーションの権限を管理します。
//! 権限の定義、権限セットの管理、権限チェックの機能を提供します。

use std::collections::HashSet;
use std::fmt;

/// 権限の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// インターネットアクセス
    Internet,
    /// ストレージへのアクセス
    Storage,
    /// 位置情報へのアクセス
    Location,
    /// Bluetooth機能へのアクセス
    Bluetooth,
    /// Wi-Fi機能へのアクセス
    WiFi,
    /// カメラへのアクセス
    Camera,
    /// マイクへのアクセス
    Microphone,
    /// 連絡先へのアクセス
    Contacts,
    /// カレンダーへのアクセス
    Calendar,
    /// 通知の送信
    Notifications,
    /// バックグラウンド実行
    BackgroundExecution,
    /// 電話機能へのアクセス
    Phone,
    /// SMS機能へのアクセス
    SMS,
    /// センサーへのアクセス
    Sensors,
    /// 生体認証へのアクセス
    Biometrics,
    /// システム設定の変更
    SystemSettings,
    /// アクセシビリティサービス
    Accessibility,
    /// 管理者権限
    Administrator,
    /// パッケージのインストール
    InstallPackages,
    /// デバイス管理
    DeviceManagement,
    /// 特権システム操作
    SystemPrivileged,
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Permission::Internet => "インターネット",
            Permission::Storage => "ストレージ",
            Permission::Location => "位置情報",
            Permission::Bluetooth => "Bluetooth",
            Permission::WiFi => "Wi-Fi",
            Permission::Camera => "カメラ",
            Permission::Microphone => "マイク",
            Permission::Contacts => "連絡先",
            Permission::Calendar => "カレンダー",
            Permission::Notifications => "通知",
            Permission::BackgroundExecution => "バックグラウンド実行",
            Permission::Phone => "電話",
            Permission::SMS => "SMS",
            Permission::Sensors => "センサー",
            Permission::Biometrics => "生体認証",
            Permission::SystemSettings => "システム設定",
            Permission::Accessibility => "アクセシビリティ",
            Permission::Administrator => "管理者権限",
            Permission::InstallPackages => "パッケージインストール",
            Permission::DeviceManagement => "デバイス管理",
            Permission::SystemPrivileged => "特権システム操作",
        };
        write!(f, "{}", name)
    }
}

impl Permission {
    /// 権限が危険（特に注意が必要）かどうか
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            Permission::Location
                | Permission::Camera
                | Permission::Microphone
                | Permission::Contacts
                | Permission::Calendar
                | Permission::Phone
                | Permission::SMS
                | Permission::Biometrics
                | Permission::SystemSettings
                | Permission::Accessibility
                | Permission::Administrator
                | Permission::InstallPackages
                | Permission::DeviceManagement
                | Permission::SystemPrivileged
        )
    }

    /// 権限の説明を取得
    pub fn description(&self) -> &'static str {
        match self {
            Permission::Internet => "インターネットへの接続を許可します",
            Permission::Storage => "ファイルの読み書きを許可します",
            Permission::Location => "位置情報へのアクセスを許可します",
            Permission::Bluetooth => "Bluetoothデバイスの検出と接続を許可します",
            Permission::WiFi => "Wi-Fi接続の管理を許可します",
            Permission::Camera => "カメラへのアクセスを許可します",
            Permission::Microphone => "マイクへのアクセスを許可します",
            Permission::Contacts => "連絡先情報へのアクセスを許可します",
            Permission::Calendar => "カレンダーイベントへのアクセスを許可します",
            Permission::Notifications => "通知の送信を許可します",
            Permission::BackgroundExecution => "バックグラウンドでの実行を許可します",
            Permission::Phone => "電話をかける/受けることを許可します",
            Permission::SMS => "SMSメッセージの送受信を許可します",
            Permission::Sensors => "センサーデータへのアクセスを許可します",
            Permission::Biometrics => "生体認証へのアクセスを許可します",
            Permission::SystemSettings => "システム設定の変更を許可します",
            Permission::Accessibility => "アクセシビリティサービスの提供を許可します",
            Permission::Administrator => "管理者権限でのアクションを許可します",
            Permission::InstallPackages => "パッケージのインストールを許可します",
            Permission::DeviceManagement => "デバイス管理機能へのアクセスを許可します",
            Permission::SystemPrivileged => "特権システム操作を許可します",
        }
    }

    /// 権限に関連するシステムの機能/コンポーネント
    pub fn related_components(&self) -> Vec<&'static str> {
        match self {
            Permission::Internet => vec!["NetworkService", "Firewall"],
            Permission::Storage => vec!["FileSystem", "StorageService"],
            Permission::Location => vec!["LocationService", "GPSManager"],
            Permission::Bluetooth => vec!["BluetoothService"],
            Permission::WiFi => vec!["WiFiService", "NetworkManager"],
            Permission::Camera => vec!["CameraService", "MediaManager"],
            Permission::Microphone => vec!["AudioService", "MediaManager"],
            Permission::Contacts => vec!["ContactsProvider", "AccountManager"],
            Permission::Calendar => vec!["CalendarProvider", "TimeManager"],
            Permission::Notifications => vec!["NotificationService", "StatusBarManager"],
            Permission::BackgroundExecution => vec!["ProcessManager", "SchedulerService"],
            Permission::Phone => vec!["TelephonyService", "CallManager"],
            Permission::SMS => vec!["SmsService", "MessageManager"],
            Permission::Sensors => vec!["SensorService", "HardwareManager"],
            Permission::Biometrics => vec!["BiometricService", "SecurityManager"],
            Permission::SystemSettings => vec!["SettingsService", "ConfigManager"],
            Permission::Accessibility => vec!["AccessibilityService", "UIManager"],
            Permission::Administrator => vec!["AdminService", "PolicyManager"],
            Permission::InstallPackages => vec!["PackageManager", "InstallerService"],
            Permission::DeviceManagement => vec!["DeviceManager", "HardwareService"],
            Permission::SystemPrivileged => vec!["SystemService", "SecurityManager", "KernelInterface"],
        }
    }
}

/// 権限の付与スコープ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionScope {
    /// 一度だけ（このセッションのみ）
    OneTime,
    /// アプリが使用中の間
    WhileInUse,
    /// 常に許可
    Always,
    /// 指定された期間
    TimeLimited(u64), // 期間（秒）
}

impl fmt::Display for PermissionScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionScope::OneTime => write!(f, "一度だけ"),
            PermissionScope::WhileInUse => write!(f, "使用中のみ"),
            PermissionScope::Always => write!(f, "常に許可"),
            PermissionScope::TimeLimited(secs) => {
                let hours = secs / 3600;
                let mins = (secs % 3600) / 60;
                if hours > 0 {
                    write!(f, "{}時間{}分間", hours, mins)
                } else {
                    write!(f, "{}分間", mins)
                }
            }
        }
    }
}

/// 権限セット
///
/// アプリケーションが持つ権限の集合を管理します。
#[derive(Debug, Clone, Default)]
pub struct PermissionSet {
    /// 許可された権限
    permissions: HashSet<Permission>,
    /// 特定のスコープで許可された権限
    scoped_permissions: Vec<(Permission, PermissionScope)>,
}

impl PermissionSet {
    /// 新しい権限セットを作成
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
            scoped_permissions: Vec::new(),
        }
    }

    /// 基本的な権限のみを持つ権限セットを作成
    pub fn with_basic_permissions() -> Self {
        let mut set = Self::new();
        set.add_permission(Permission::Internet);
        set.add_permission(Permission::Storage);
        set.add_permission(Permission::Notifications);
        set
    }

    /// 指定された権限を追加
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// 指定された権限を削除
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
        // スコープ付き権限も削除
        self.scoped_permissions.retain(|(p, _)| p != permission);
    }

    /// 指定されたスコープで権限を追加
    pub fn add_scoped_permission(&mut self, permission: Permission, scope: PermissionScope) {
        // すでに同じ権限がある場合は削除
        self.scoped_permissions.retain(|(p, _)| p != &permission);
        self.scoped_permissions.push((permission, scope));
    }

    /// 指定された権限があるかどうかを確認
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission) || 
        self.scoped_permissions.iter().any(|(p, _)| p == permission)
    }

    /// 指定されたスコープで権限があるかどうかを確認
    pub fn has_permission_with_scope(&self, permission: &Permission, required_scope: &PermissionScope) -> bool {
        if self.permissions.contains(permission) {
            return true; // 無制限の権限がある
        }

        self.scoped_permissions.iter().any(|(p, scope)| {
            p == permission && match (scope, required_scope) {
                // OneTimeはすべてのスコープで有効
                (PermissionScope::OneTime, _) => true,
                // WhileInUseはOneTimeとWhileInUseで有効
                (PermissionScope::WhileInUse, PermissionScope::OneTime) => true,
                (PermissionScope::WhileInUse, PermissionScope::WhileInUse) => true,
                // Alwaysはすべてのスコープで有効
                (PermissionScope::Always, _) => true,
                // TimeLimitedは期間による（ここでは常に一致すると仮定）
                (PermissionScope::TimeLimited(_), _) => true,
                // その他の組み合わせは無効
                _ => false,
            }
        })
    }

    /// すべての権限を取得
    pub fn get_all_permissions(&self) -> HashSet<Permission> {
        let mut all = self.permissions.clone();
        for (p, _) in &self.scoped_permissions {
            all.insert(*p);
        }
        all
    }

    /// 特定のスコープでの権限を取得
    pub fn get_scoped_permissions(&self, scope: PermissionScope) -> Vec<Permission> {
        self.scoped_permissions
            .iter()
            .filter_map(|(p, s)| if *s == scope { Some(*p) } else { None })
            .collect()
    }

    /// 危険な権限のみを取得
    pub fn get_dangerous_permissions(&self) -> HashSet<Permission> {
        self.get_all_permissions()
            .into_iter()
            .filter(|p| p.is_dangerous())
            .collect()
    }

    /// 権限セットをマージ
    pub fn merge(&mut self, other: &PermissionSet) {
        // 無制限の権限をマージ
        for p in &other.permissions {
            self.permissions.insert(*p);
        }

        // スコープ付き権限をマージ（同じ権限は上書き）
        for (p, s) in &other.scoped_permissions {
            if !self.permissions.contains(p) {
                self.add_scoped_permission(*p, *s);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set_basic() {
        let mut perms = PermissionSet::new();

        // 権限がない状態
        assert!(!perms.has_permission(&Permission::Internet));

        // 権限を追加
        perms.add_permission(Permission::Internet);
        assert!(perms.has_permission(&Permission::Internet));
        assert!(!perms.has_permission(&Permission::Camera));

        // 権限を削除
        perms.remove_permission(&Permission::Internet);
        assert!(!perms.has_permission(&Permission::Internet));
    }

    #[test]
    fn test_permission_with_scope() {
        let mut perms = PermissionSet::new();

        // スコープ付き権限を追加
        perms.add_scoped_permission(Permission::Location, PermissionScope::WhileInUse);
        assert!(perms.has_permission(&Permission::Location));
        assert!(perms.has_permission_with_scope(
            &Permission::Location,
            &PermissionScope::WhileInUse
        ));
        assert!(perms.has_permission_with_scope(
            &Permission::Location,
            &PermissionScope::OneTime
        ));
        assert!(!perms.has_permission_with_scope(
            &Permission::Location,
            &PermissionScope::Always
        ));

        // Always権限を追加
        perms.add_scoped_permission(Permission::Camera, PermissionScope::Always);
        assert!(perms.has_permission_with_scope(
            &Permission::Camera,
            &PermissionScope::OneTime
        ));
        assert!(perms.has_permission_with_scope(
            &Permission::Camera,
            &PermissionScope::WhileInUse
        ));
        assert!(perms.has_permission_with_scope(
            &Permission::Camera,
            &PermissionScope::Always
        ));

        // 無制限の権限を追加
        perms.add_permission(Permission::Storage);
        assert!(perms.has_permission(&Permission::Storage));
        assert!(perms.has_permission_with_scope(
            &Permission::Storage,
            &PermissionScope::Always
        ));
    }

    #[test]
    fn test_dangerous_permissions() {
        let mut perms = PermissionSet::new();
        perms.add_permission(Permission::Internet); // 危険でない
        perms.add_permission(Permission::Storage); // 危険でない
        perms.add_permission(Permission::Location); // 危険
        perms.add_permission(Permission::Camera); // 危険

        let dangerous = perms.get_dangerous_permissions();
        assert_eq!(dangerous.len(), 2);
        assert!(dangerous.contains(&Permission::Location));
        assert!(dangerous.contains(&Permission::Camera));
        assert!(!dangerous.contains(&Permission::Internet));
        assert!(!dangerous.contains(&Permission::Storage));
    }

    #[test]
    fn test_merge_permission_sets() {
        let mut set1 = PermissionSet::new();
        set1.add_permission(Permission::Internet);
        set1.add_scoped_permission(Permission::Location, PermissionScope::WhileInUse);

        let mut set2 = PermissionSet::new();
        set2.add_permission(Permission::Storage);
        set2.add_scoped_permission(Permission::Camera, PermissionScope::OneTime);

        set1.merge(&set2);

        assert!(set1.has_permission(&Permission::Internet));
        assert!(set1.has_permission(&Permission::Storage));
        assert!(set1.has_permission(&Permission::Location));
        assert!(set1.has_permission(&Permission::Camera));

        assert!(set1.has_permission_with_scope(
            &Permission::Location,
            &PermissionScope::WhileInUse
        ));
        assert!(set1.has_permission_with_scope(
            &Permission::Camera,
            &PermissionScope::OneTime
        ));
    }

    #[test]
    fn test_basic_permissions() {
        let perms = PermissionSet::with_basic_permissions();
        assert!(perms.has_permission(&Permission::Internet));
        assert!(perms.has_permission(&Permission::Storage));
        assert!(perms.has_permission(&Permission::Notifications));
        assert!(!perms.has_permission(&Permission::Camera));
    }

    #[test]
    fn test_permission_descriptions() {
        for p in [
            Permission::Internet,
            Permission::Storage,
            Permission::Location,
            Permission::Camera,
        ] {
            assert!(!p.description().is_empty());
            assert!(!p.related_components().is_empty());
        }
    }
} 