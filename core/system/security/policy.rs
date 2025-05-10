use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use super::error::{SecurityError, SecurityResult};
use super::context::{Permission, SecurityLevel};

/// セキュリティポリシーの種類
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyType {
    /// システム全体のポリシー
    System,
    /// ユーザー固有のポリシー
    User,
    /// アプリケーション固有のポリシー
    Application,
    /// デバイス固有のポリシー
    Device,
    /// ネットワーク固有のポリシー
    Network,
    /// データ保護ポリシー
    DataProtection,
    /// カスタムポリシー
    Custom(String),
}

/// セキュリティポリシールール
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// ルールの一意識別子
    pub id: String,
    /// ルールの名前
    pub name: String,
    /// ルールの説明
    pub description: String,
    /// ルールの種類
    pub policy_type: PolicyType,
    /// ルールの優先度（高いほど優先）
    pub priority: u8,
    /// このルールが適用される権限のセット
    pub affected_permissions: HashSet<Permission>,
    /// 必要なセキュリティレベル
    pub required_security_level: SecurityLevel,
    /// ルールが有効かどうか
    pub enabled: bool,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 最終更新日時
    pub updated_at: DateTime<Utc>,
    /// ルールのカスタム属性
    pub attributes: HashMap<String, String>,
}

impl PolicyRule {
    /// 新しいポリシールールを作成する
    pub fn new(
        id: String,
        name: String,
        description: String,
        policy_type: PolicyType,
        priority: u8,
        affected_permissions: HashSet<Permission>,
        required_security_level: SecurityLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            policy_type,
            priority,
            affected_permissions,
            required_security_level,
            enabled: true,
            created_at: now,
            updated_at: now,
            attributes: HashMap::new(),
        }
    }

    /// ルールが特定の権限に影響するかどうかを確認する
    pub fn affects_permission(&self, permission: &Permission) -> bool {
        self.enabled && self.affected_permissions.contains(permission)
    }

    /// ルールを有効または無効にする
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.updated_at = Utc::now();
    }

    /// ルールにカスタム属性を追加する
    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// ルールからカスタム属性を削除する
    pub fn remove_attribute(&mut self, key: &str) -> Option<String> {
        let result = self.attributes.remove(key);
        if result.is_some() {
            self.updated_at = Utc::now();
        }
        result
    }
}

/// セキュリティポリシーのセット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// ポリシーの一意識別子
    pub id: String,
    /// ポリシーの名前
    pub name: String,
    /// ポリシーの説明
    pub description: String,
    /// ポリシーのバージョン
    pub version: String,
    /// ポリシールールのセット
    pub rules: Vec<PolicyRule>,
    /// ポリシーが有効かどうか
    pub enabled: bool,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 最終更新日時
    pub updated_at: DateTime<Utc>,
}

impl SecurityPolicy {
    /// 新しいセキュリティポリシーを作成する
    pub fn new(
        id: String,
        name: String,
        description: String,
        version: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            version,
            rules: Vec::new(),
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// ポリシーにルールを追加する
    pub fn add_rule(&mut self, rule: PolicyRule) -> SecurityResult<()> {
        // 既存のルールIDと重複していないか確認
        if self.rules.iter().any(|r| r.id == rule.id) {
            return Err(SecurityError::AlreadyExistsError(
                format!("ルールID '{}' は既に存在します", rule.id)
            ));
        }
        
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority)); // 優先度で降順ソート
        self.updated_at = Utc::now();
        Ok(())
    }

    /// ポリシーからルールを削除する
    pub fn remove_rule(&mut self, rule_id: &str) -> SecurityResult<PolicyRule> {
        let pos = self.rules.iter().position(|r| r.id == rule_id)
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("ルールID '{}' が見つかりません", rule_id)
            ))?;
        
        let rule = self.rules.remove(pos);
        self.updated_at = Utc::now();
        Ok(rule)
    }

    /// 指定された権限に影響するルールを取得する
    pub fn get_rules_for_permission(&self, permission: &Permission) -> Vec<&PolicyRule> {
        self.rules.iter()
            .filter(|r| r.enabled && r.affects_permission(permission))
            .collect()
    }

    /// ポリシーを有効または無効にする
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.updated_at = Utc::now();
    }

    /// 指定されたルールを更新する
    pub fn update_rule(&mut self, rule_id: &str, updater: impl FnOnce(&mut PolicyRule)) -> SecurityResult<()> {
        let rule = self.rules.iter_mut()
            .find(|r| r.id == rule_id)
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("ルールID '{}' が見つかりません", rule_id)
            ))?;
        
        updater(rule);
        rule.updated_at = Utc::now();
        self.updated_at = Utc::now();
        
        // 優先度が変更された可能性があるため再ソート
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }

    /// 指定された権限が許可されるかどうかを決定する
    pub fn evaluate_permission(
        &self,
        permission: &Permission,
        security_level: SecurityLevel,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        // この権限に影響するルールがあるか確認
        let affecting_rules = self.get_rules_for_permission(permission);
        if affecting_rules.is_empty() {
            // ルールがない場合はデフォルトで許可
            return true;
        }

        // 最も優先度の高いルールから評価（すでにソート済み）
        for rule in affecting_rules {
            // 現在のセキュリティレベルが必要なレベル以上であるか確認
            if security_level >= rule.required_security_level {
                return true;
            }
        }

        // すべてのルールが拒否した場合
        false
    }

    /// ポリシー全体をシリアライズする
    pub fn serialize(&self) -> SecurityResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| SecurityError::SerializationError(
                format!("ポリシーのシリアライズに失敗しました: {}", e)
            ))
    }

    /// シリアライズされたポリシーからデシリアライズする
    pub fn deserialize(data: &str) -> SecurityResult<Self> {
        serde_json::from_str(data)
            .map_err(|e| SecurityError::SerializationError(
                format!("ポリシーのデシリアライズに失敗しました: {}", e)
            ))
    }
}

/// デフォルトのシステムポリシーを作成する
pub fn create_default_system_policy() -> SecurityPolicy {
    let mut policy = SecurityPolicy::new(
        "system_default".to_string(),
        "システムデフォルトポリシー".to_string(),
        "基本的なシステムセキュリティルールを定義します".to_string(),
        "1.0.0".to_string(),
    );

    // ファイルアクセスルール
    let file_access_rule = PolicyRule::new(
        "file_access".to_string(),
        "ファイルアクセス".to_string(),
        "ファイルシステムへのアクセス権限を制御します".to_string(),
        PolicyType::System,
        100, // 高優先度
        [
            Permission::FileRead,
            Permission::FileWrite,
            Permission::FileExecute,
        ].iter().cloned().collect(),
        SecurityLevel::Standard,
    );

    // ネットワークアクセスルール
    let network_access_rule = PolicyRule::new(
        "network_access".to_string(),
        "ネットワークアクセス".to_string(),
        "ネットワークリソースへのアクセス権限を制御します".to_string(),
        PolicyType::Network,
        90,
        [
            Permission::NetworkConnect,
            Permission::NetworkListen,
        ].iter().cloned().collect(),
        SecurityLevel::Standard,
    );

    // システム設定ルール
    let system_settings_rule = PolicyRule::new(
        "system_settings".to_string(),
        "システム設定".to_string(),
        "システム設定の変更権限を制御します".to_string(),
        PolicyType::System,
        80,
        [
            Permission::SettingsRead,
            Permission::SettingsWrite,
        ].iter().cloned().collect(),
        SecurityLevel::High,
    );

    // ハードウェアアクセスルール
    let hardware_access_rule = PolicyRule::new(
        "hardware_access".to_string(),
        "ハードウェアアクセス".to_string(),
        "ハードウェアデバイスへのアクセス権限を制御します".to_string(),
        PolicyType::Device,
        70,
        [
            Permission::HardwareAccess,
            Permission::USBAccess,
            Permission::AudioRecord,
            Permission::VideoRecord,
        ].iter().cloned().collect(),
        SecurityLevel::High,
    );

    // プライバシーデータアクセスルール
    let privacy_data_rule = PolicyRule::new(
        "privacy_data".to_string(),
        "プライバシーデータ".to_string(),
        "ユーザーのプライバシーデータへのアクセス権限を制御します".to_string(),
        PolicyType::DataProtection,
        110, // 最高優先度
        [
            Permission::ContactsAccess,
            Permission::LocationAccess,
            Permission::CalendarAccess,
            Permission::HealthDataAccess,
        ].iter().cloned().collect(),
        SecurityLevel::Highest,
    );

    // 管理者権限ルール
    let admin_rule = PolicyRule::new(
        "admin_access".to_string(),
        "管理者アクセス".to_string(),
        "管理者レベルの操作権限を制御します".to_string(),
        PolicyType::System,
        120, // 最高優先度
        [
            Permission::SystemAdmin,
            Permission::InstallSoftware,
            Permission::ManageUsers,
        ].iter().cloned().collect(),
        SecurityLevel::Highest,
    );

    // ルールを追加
    let _ = policy.add_rule(file_access_rule);
    let _ = policy.add_rule(network_access_rule);
    let _ = policy.add_rule(system_settings_rule);
    let _ = policy.add_rule(hardware_access_rule);
    let _ = policy.add_rule(privacy_data_rule);
    let _ = policy.add_rule(admin_rule);

    policy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_rule_creation() {
        let permissions = [Permission::FileRead, Permission::FileWrite]
            .iter().cloned().collect();
        
        let rule = PolicyRule::new(
            "test_rule".to_string(),
            "テストルール".to_string(),
            "テスト用のルールです".to_string(),
            PolicyType::System,
            50,
            permissions,
            SecurityLevel::Standard,
        );

        assert_eq!(rule.id, "test_rule");
        assert_eq!(rule.name, "テストルール");
        assert!(rule.enabled);
        assert_eq!(rule.priority, 50);
        assert_eq!(rule.required_security_level, SecurityLevel::Standard);
        assert!(rule.affects_permission(&Permission::FileRead));
        assert!(rule.affects_permission(&Permission::FileWrite));
        assert!(!rule.affects_permission(&Permission::FileExecute));
    }

    #[test]
    fn test_policy_creation_and_rule_management() {
        let mut policy = SecurityPolicy::new(
            "test_policy".to_string(),
            "テストポリシー".to_string(),
            "テスト用のポリシーです".to_string(),
            "0.1.0".to_string(),
        );

        assert_eq!(policy.id, "test_policy");
        assert_eq!(policy.name, "テストポリシー");
        assert!(policy.enabled);
        assert!(policy.rules.is_empty());

        // ルール1を追加
        let rule1 = PolicyRule::new(
            "rule1".to_string(),
            "ルール1".to_string(),
            "テスト用ルール1".to_string(),
            PolicyType::System,
            50,
            [Permission::FileRead].iter().cloned().collect(),
            SecurityLevel::Standard,
        );
        let result = policy.add_rule(rule1);
        assert!(result.is_ok());
        assert_eq!(policy.rules.len(), 1);

        // ルール2を追加（より高い優先度）
        let rule2 = PolicyRule::new(
            "rule2".to_string(),
            "ルール2".to_string(),
            "テスト用ルール2".to_string(),
            PolicyType::System,
            100,
            [Permission::FileWrite].iter().cloned().collect(),
            SecurityLevel::High,
        );
        let result = policy.add_rule(rule2);
        assert!(result.is_ok());
        assert_eq!(policy.rules.len(), 2);
        
        // 優先度に基づいてソートされていることを確認
        assert_eq!(policy.rules[0].id, "rule2"); // 優先度が高いルールが最初
        
        // ルールを更新
        let result = policy.update_rule("rule1", |rule| {
            rule.priority = 150; // 優先度を更新
            rule.add_attribute("key1".to_string(), "value1".to_string());
        });
        assert!(result.is_ok());
        
        // 再ソートされていることを確認
        assert_eq!(policy.rules[0].id, "rule1"); // 優先度が更新されたので最初
        
        // 属性が追加されたか確認
        assert_eq!(
            policy.rules[0].attributes.get("key1").unwrap(),
            "value1"
        );
        
        // 存在しないルールの更新を試みる
        let result = policy.update_rule("nonexistent", |_| {});
        assert!(result.is_err());
        
        // ルールを削除
        let result = policy.remove_rule("rule1");
        assert!(result.is_ok());
        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.rules[0].id, "rule2");
        
        // 存在しないルールの削除を試みる
        let result = policy.remove_rule("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_permission_evaluation() {
        let mut policy = SecurityPolicy::new(
            "test_policy".to_string(),
            "テストポリシー".to_string(),
            "テスト用のポリシーです".to_string(),
            "0.1.0".to_string(),
        );

        // ファイルアクセスルール（標準セキュリティレベルが必要）
        let file_rule = PolicyRule::new(
            "file_access".to_string(),
            "ファイルアクセス".to_string(),
            "ファイルアクセス権限".to_string(),
            PolicyType::System,
            100,
            [Permission::FileRead, Permission::FileWrite].iter().cloned().collect(),
            SecurityLevel::Standard,
        );
        let _ = policy.add_rule(file_rule);

        // 管理者ルール（最高セキュリティレベルが必要）
        let admin_rule = PolicyRule::new(
            "admin_access".to_string(),
            "管理者アクセス".to_string(),
            "管理者権限".to_string(),
            PolicyType::System,
            200,
            [Permission::SystemAdmin].iter().cloned().collect(),
            SecurityLevel::Highest,
        );
        let _ = policy.add_rule(admin_rule);

        // 標準レベルでの評価
        assert!(policy.evaluate_permission(&Permission::FileRead, SecurityLevel::Standard));
        assert!(policy.evaluate_permission(&Permission::FileWrite, SecurityLevel::Standard));
        assert!(!policy.evaluate_permission(&Permission::SystemAdmin, SecurityLevel::Standard));

        // 高レベルでの評価
        assert!(policy.evaluate_permission(&Permission::FileRead, SecurityLevel::High));
        assert!(policy.evaluate_permission(&Permission::FileWrite, SecurityLevel::High));
        assert!(!policy.evaluate_permission(&Permission::SystemAdmin, SecurityLevel::High));

        // 最高レベルでの評価
        assert!(policy.evaluate_permission(&Permission::FileRead, SecurityLevel::Highest));
        assert!(policy.evaluate_permission(&Permission::FileWrite, SecurityLevel::Highest));
        assert!(policy.evaluate_permission(&Permission::SystemAdmin, SecurityLevel::Highest));

        // ポリシーを無効化
        policy.set_enabled(false);
        assert!(!policy.evaluate_permission(&Permission::FileRead, SecurityLevel::Highest));
        assert!(!policy.evaluate_permission(&Permission::SystemAdmin, SecurityLevel::Highest));
    }

    #[test]
    fn test_default_system_policy() {
        let policy = create_default_system_policy();
        
        assert_eq!(policy.id, "system_default");
        assert!(policy.enabled);
        assert!(!policy.rules.is_empty());
        
        // 標準ユーザー権限の確認
        assert!(policy.evaluate_permission(&Permission::FileRead, SecurityLevel::Standard));
        assert!(policy.evaluate_permission(&Permission::NetworkConnect, SecurityLevel::Standard));
        assert!(!policy.evaluate_permission(&Permission::SystemAdmin, SecurityLevel::Standard));
        assert!(!policy.evaluate_permission(&Permission::SettingsWrite, SecurityLevel::Standard));
        
        // 管理者権限の確認
        assert!(policy.evaluate_permission(&Permission::SystemAdmin, SecurityLevel::Highest));
        assert!(policy.evaluate_permission(&Permission::SettingsWrite, SecurityLevel::High));
    }

    #[test]
    fn test_serialization() {
        let policy = create_default_system_policy();
        
        // シリアライズ
        let serialized = policy.serialize();
        assert!(serialized.is_ok());
        let json = serialized.unwrap();
        
        // デシリアライズ
        let deserialized = SecurityPolicy::deserialize(&json);
        assert!(deserialized.is_ok());
        let restored_policy = deserialized.unwrap();
        
        // 同じ内容か確認
        assert_eq!(policy.id, restored_policy.id);
        assert_eq!(policy.name, restored_policy.name);
        assert_eq!(policy.rules.len(), restored_policy.rules.len());
    }
} 