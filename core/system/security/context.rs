use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::error::{SecurityError, SecurityResult};
use super::policy::{SecurityPolicy, PolicyType};

/// セキュリティレベルを表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// 最低限のセキュリティレベル
    Minimal,
    /// 低いセキュリティレベル
    Low,
    /// 標準のセキュリティレベル
    Standard,
    /// 高いセキュリティレベル
    High,
    /// 最高のセキュリティレベル
    Highest,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        SecurityLevel::Standard
    }
}

/// システム内で定義される権限
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // ファイルシステム関連
    /// ファイル読み取り権限
    FileRead,
    /// ファイル書き込み権限
    FileWrite,
    /// ファイル実行権限
    FileExecute,
    
    // ネットワーク関連
    /// ネットワーク接続権限
    NetworkConnect,
    /// ネットワーク待ち受け権限
    NetworkListen,
    
    // システム設定関連
    /// 設定読み取り権限
    SettingsRead,
    /// 設定書き込み権限
    SettingsWrite,
    
    // ハードウェア関連
    /// ハードウェアアクセス権限
    HardwareAccess,
    /// USB機器アクセス権限
    USBAccess,
    /// オーディオ録音権限
    AudioRecord,
    /// ビデオ録画権限
    VideoRecord,
    
    // プライバシーデータ関連
    /// 連絡先アクセス権限
    ContactsAccess,
    /// 位置情報アクセス権限
    LocationAccess,
    /// カレンダーアクセス権限
    CalendarAccess,
    /// ヘルスデータアクセス権限
    HealthDataAccess,
    
    // 管理者関連
    /// システム管理者権限
    SystemAdmin,
    /// ソフトウェアインストール権限
    InstallSoftware,
    /// ユーザー管理権限
    ManageUsers,
    
    // カスタム権限
    /// カスタム権限（名前付き）
    Custom(String),
}

/// 認証の種類
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthenticationType {
    /// パスワード認証
    Password,
    /// 生体認証
    Biometric,
    /// 二要素認証
    TwoFactor,
    /// シングルサインオン
    SingleSignOn,
    /// カスタム認証
    Custom(String),
}

/// セキュリティエンティティの種類
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// ユーザー
    User,
    /// アプリケーション
    Application,
    /// システムサービス
    SystemService,
    /// プラグイン
    Plugin,
    /// デバイス
    Device,
    /// カスタムエンティティ
    Custom(String),
}

/// セキュリティコンテキストが持つセッション情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySession {
    /// セッションID
    pub id: String,
    /// セッション作成時刻
    pub created_at: DateTime<Utc>,
    /// 最終アクティブ時刻
    pub last_active: DateTime<Utc>,
    /// セッション有効期限
    pub expires_at: Option<DateTime<Utc>>,
    /// 認証の種類
    pub auth_type: AuthenticationType,
    /// セッションメタデータ
    pub metadata: HashMap<String, String>,
}

impl SecuritySession {
    /// 新しいセキュリティセッションを作成する
    pub fn new(auth_type: AuthenticationType, expires_in_secs: Option<u64>) -> Self {
        let now = Utc::now();
        let expires_at = expires_in_secs.map(|secs| now + chrono::Duration::seconds(secs as i64));
        
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: now,
            last_active: now,
            expires_at,
            auth_type,
            metadata: HashMap::new(),
        }
    }

    /// セッションがまだ有効かどうかを確認する
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expiry) => Utc::now() < expiry,
            None => true,
        }
    }

    /// セッションの最終アクティブ時刻を更新する
    pub fn update_activity(&mut self) {
        self.last_active = Utc::now();
    }

    /// セッションにメタデータを設定する
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// セッションからメタデータを取得する
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// セキュリティエンティティ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEntity {
    /// エンティティID
    pub id: String,
    /// エンティティ名
    pub name: String,
    /// エンティティの種類
    pub entity_type: EntityType,
    /// セキュリティレベル
    pub security_level: SecurityLevel,
    /// 付与された権限のセット
    pub granted_permissions: HashSet<Permission>,
    /// 明示的に拒否された権限のセット
    pub denied_permissions: HashSet<Permission>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 最終更新日時
    pub updated_at: DateTime<Utc>,
    /// エンティティが有効かどうか
    pub enabled: bool,
    /// エンティティメタデータ
    pub metadata: HashMap<String, String>,
}

impl SecurityEntity {
    /// 新しいセキュリティエンティティを作成する
    pub fn new(
        id: String,
        name: String,
        entity_type: EntityType,
        security_level: SecurityLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            entity_type,
            security_level,
            granted_permissions: HashSet::new(),
            denied_permissions: HashSet::new(),
            created_at: now,
            updated_at: now,
            enabled: true,
            metadata: HashMap::new(),
        }
    }

    /// 権限を付与する
    pub fn grant_permission(&mut self, permission: Permission) {
        self.denied_permissions.remove(&permission);
        self.granted_permissions.insert(permission);
        self.updated_at = Utc::now();
    }

    /// 権限を拒否する
    pub fn deny_permission(&mut self, permission: Permission) {
        self.granted_permissions.remove(&permission);
        self.denied_permissions.insert(permission);
        self.updated_at = Utc::now();
    }

    /// 権限の付与・拒否をリセットする
    pub fn reset_permission(&mut self, permission: &Permission) {
        self.granted_permissions.remove(permission);
        self.denied_permissions.remove(permission);
        self.updated_at = Utc::now();
    }

    /// 権限が付与されているかどうかを確認する
    pub fn has_permission(&self, permission: &Permission) -> bool {
        if !self.enabled {
            return false;
        }
        
        self.granted_permissions.contains(permission) && !self.denied_permissions.contains(permission)
    }

    /// セキュリティレベルを設定する
    pub fn set_security_level(&mut self, level: SecurityLevel) {
        self.security_level = level;
        self.updated_at = Utc::now();
    }

    /// エンティティを有効または無効にする
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.updated_at = Utc::now();
    }

    /// メタデータを設定する
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// メタデータを取得する
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// セキュリティコンテキスト
/// システムのセキュリティ状態と権限管理を担当
#[derive(Debug)]
pub struct SecurityContext {
    /// セキュリティポリシー
    policies: Arc<RwLock<HashMap<String, SecurityPolicy>>>,
    /// セキュリティエンティティ
    entities: Arc<RwLock<HashMap<String, SecurityEntity>>>,
    /// アクティブセッション
    sessions: Arc<RwLock<HashMap<String, SecuritySession>>>,
    /// 現在のデフォルトポリシーID
    default_policy_id: Arc<RwLock<String>>,
}

impl SecurityContext {
    /// 新しいセキュリティコンテキストを作成する
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            entities: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            default_policy_id: Arc::new(RwLock::new(String::new())),
        }
    }

    /// コンテキストを初期化する
    pub fn initialize(&self) -> SecurityResult<()> {
        // デフォルトポリシーを作成
        let default_policy = super::policy::create_default_system_policy();
        let policy_id = default_policy.id.clone();
        
        // ポリシーを登録
        self.register_policy(default_policy)?;
        
        // デフォルトポリシーを設定
        *self.default_policy_id.write().map_err(|_| {
            SecurityError::InternalError("デフォルトポリシーIDの書き込みロックに失敗しました".to_string())
        })? = policy_id;
        
        // システムサービスエンティティを作成
        let system_entity = SecurityEntity::new(
            "system".to_string(),
            "システムサービス".to_string(),
            EntityType::SystemService,
            SecurityLevel::Highest,
        );
        
        // エンティティを登録
        self.register_entity(system_entity)?;
        
        Ok(())
    }

    /// ポリシーを登録する
    pub fn register_policy(&self, policy: SecurityPolicy) -> SecurityResult<()> {
        let mut policies = self.policies.write().map_err(|_| {
            SecurityError::InternalError("ポリシーの書き込みロックに失敗しました".to_string())
        })?;
        
        // 既に同じIDのポリシーが存在するか確認
        if policies.contains_key(&policy.id) {
            return Err(SecurityError::AlreadyExistsError(
                format!("ポリシーID '{}' は既に存在します", policy.id)
            ));
        }
        
        policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    /// ポリシーを取得する
    pub fn get_policy(&self, policy_id: &str) -> SecurityResult<SecurityPolicy> {
        let policies = self.policies.read().map_err(|_| {
            SecurityError::InternalError("ポリシーの読み取りロックに失敗しました".to_string())
        })?;
        
        policies.get(policy_id)
            .cloned()
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("ポリシーID '{}' が見つかりません", policy_id)
            ))
    }

    /// ポリシーを更新する
    pub fn update_policy<F>(&self, policy_id: &str, updater: F) -> SecurityResult<()>
    where
        F: FnOnce(&mut SecurityPolicy)
    {
        let mut policies = self.policies.write().map_err(|_| {
            SecurityError::InternalError("ポリシーの書き込みロックに失敗しました".to_string())
        })?;
        
        let policy = policies.get_mut(policy_id)
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("ポリシーID '{}' が見つかりません", policy_id)
            ))?;
        
        updater(policy);
        Ok(())
    }

    /// ポリシーを削除する
    pub fn remove_policy(&self, policy_id: &str) -> SecurityResult<SecurityPolicy> {
        // デフォルトポリシーは削除できない
        {
            let default_id = self.default_policy_id.read().map_err(|_| {
                SecurityError::InternalError("デフォルトポリシーIDの読み取りロックに失敗しました".to_string())
            })?;
            
            if &*default_id == policy_id {
                return Err(SecurityError::InvalidArgumentError(
                    "デフォルトポリシーは削除できません".to_string()
                ));
            }
        }
        
        let mut policies = self.policies.write().map_err(|_| {
            SecurityError::InternalError("ポリシーの書き込みロックに失敗しました".to_string())
        })?;
        
        policies.remove(policy_id)
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("ポリシーID '{}' が見つかりません", policy_id)
            ))
    }

    /// 指定した種類のポリシーをすべて取得する
    pub fn get_policies_by_type(&self, policy_type: &PolicyType) -> SecurityResult<Vec<SecurityPolicy>> {
        let policies = self.policies.read().map_err(|_| {
            SecurityError::InternalError("ポリシーの読み取りロックに失敗しました".to_string())
        })?;
        
        let filtered = policies.values()
            .filter(|p| p.rules.iter().any(|r| &r.policy_type == policy_type))
            .cloned()
            .collect();
        
        Ok(filtered)
    }

    /// デフォルトポリシーを設定する
    pub fn set_default_policy(&self, policy_id: &str) -> SecurityResult<()> {
        // 指定されたポリシーが存在するか確認
        {
            let policies = self.policies.read().map_err(|_| {
                SecurityError::InternalError("ポリシーの読み取りロックに失敗しました".to_string())
            })?;
            
            if !policies.contains_key(policy_id) {
                return Err(SecurityError::NotFoundError(
                    format!("ポリシーID '{}' が見つかりません", policy_id)
                ));
            }
        }
        
        // デフォルトポリシーを更新
        *self.default_policy_id.write().map_err(|_| {
            SecurityError::InternalError("デフォルトポリシーIDの書き込みロックに失敗しました".to_string())
        })? = policy_id.to_string();
        
        Ok(())
    }

    /// デフォルトポリシーを取得する
    pub fn get_default_policy(&self) -> SecurityResult<SecurityPolicy> {
        let default_id = self.default_policy_id.read().map_err(|_| {
            SecurityError::InternalError("デフォルトポリシーIDの読み取りロックに失敗しました".to_string())
        })?;
        
        if default_id.is_empty() {
            return Err(SecurityError::InvalidStateError(
                "デフォルトポリシーが設定されていません".to_string()
            ));
        }
        
        self.get_policy(&default_id)
    }

    /// エンティティを登録する
    pub fn register_entity(&self, entity: SecurityEntity) -> SecurityResult<()> {
        let mut entities = self.entities.write().map_err(|_| {
            SecurityError::InternalError("エンティティの書き込みロックに失敗しました".to_string())
        })?;
        
        // 既に同じIDのエンティティが存在するか確認
        if entities.contains_key(&entity.id) {
            return Err(SecurityError::AlreadyExistsError(
                format!("エンティティID '{}' は既に存在します", entity.id)
            ));
        }
        
        entities.insert(entity.id.clone(), entity);
        Ok(())
    }

    /// エンティティを取得する
    pub fn get_entity(&self, entity_id: &str) -> SecurityResult<SecurityEntity> {
        let entities = self.entities.read().map_err(|_| {
            SecurityError::InternalError("エンティティの読み取りロックに失敗しました".to_string())
        })?;
        
        entities.get(entity_id)
            .cloned()
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("エンティティID '{}' が見つかりません", entity_id)
            ))
    }

    /// エンティティを更新する
    pub fn update_entity<F>(&self, entity_id: &str, updater: F) -> SecurityResult<()>
    where
        F: FnOnce(&mut SecurityEntity)
    {
        let mut entities = self.entities.write().map_err(|_| {
            SecurityError::InternalError("エンティティの書き込みロックに失敗しました".to_string())
        })?;
        
        let entity = entities.get_mut(entity_id)
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("エンティティID '{}' が見つかりません", entity_id)
            ))?;
        
        updater(entity);
        Ok(())
    }

    /// エンティティを削除する
    pub fn remove_entity(&self, entity_id: &str) -> SecurityResult<SecurityEntity> {
        // システムエンティティは削除できない
        if entity_id == "system" {
            return Err(SecurityError::InvalidArgumentError(
                "システムエンティティは削除できません".to_string()
            ));
        }
        
        let mut entities = self.entities.write().map_err(|_| {
            SecurityError::InternalError("エンティティの書き込みロックに失敗しました".to_string())
        })?;
        
        entities.remove(entity_id)
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("エンティティID '{}' が見つかりません", entity_id)
            ))
    }

    /// 指定した種類のエンティティをすべて取得する
    pub fn get_entities_by_type(&self, entity_type: &EntityType) -> SecurityResult<Vec<SecurityEntity>> {
        let entities = self.entities.read().map_err(|_| {
            SecurityError::InternalError("エンティティの読み取りロックに失敗しました".to_string())
        })?;
        
        let filtered = entities.values()
            .filter(|e| &e.entity_type == entity_type)
            .cloned()
            .collect();
        
        Ok(filtered)
    }

    /// セッションを作成する
    pub fn create_session(
        &self,
        auth_type: AuthenticationType,
        expires_in_secs: Option<u64>,
    ) -> SecurityResult<SecuritySession> {
        let session = SecuritySession::new(auth_type, expires_in_secs);
        let session_id = session.id.clone();
        
        let mut sessions = self.sessions.write().map_err(|_| {
            SecurityError::InternalError("セッションの書き込みロックに失敗しました".to_string())
        })?;
        
        sessions.insert(session_id, session.clone());
        Ok(session)
    }

    /// セッションを取得する
    pub fn get_session(&self, session_id: &str) -> SecurityResult<SecuritySession> {
        let sessions = self.sessions.read().map_err(|_| {
            SecurityError::InternalError("セッションの読み取りロックに失敗しました".to_string())
        })?;
        
        sessions.get(session_id)
            .cloned()
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("セッションID '{}' が見つかりません", session_id)
            ))
    }

    /// セッションを更新する
    pub fn update_session<F>(&self, session_id: &str, updater: F) -> SecurityResult<()>
    where
        F: FnOnce(&mut SecuritySession)
    {
        let mut sessions = self.sessions.write().map_err(|_| {
            SecurityError::InternalError("セッションの書き込みロックに失敗しました".to_string())
        })?;
        
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("セッションID '{}' が見つかりません", session_id)
            ))?;
        
        updater(session);
        Ok(())
    }

    /// セッションを削除する
    pub fn remove_session(&self, session_id: &str) -> SecurityResult<SecuritySession> {
        let mut sessions = self.sessions.write().map_err(|_| {
            SecurityError::InternalError("セッションの書き込みロックに失敗しました".to_string())
        })?;
        
        sessions.remove(session_id)
            .ok_or_else(|| SecurityError::NotFoundError(
                format!("セッションID '{}' が見つかりません", session_id)
            ))
    }

    /// 期限切れのセッションをすべて削除する
    pub fn cleanup_expired_sessions(&self) -> SecurityResult<usize> {
        let mut sessions = self.sessions.write().map_err(|_| {
            SecurityError::InternalError("セッションの書き込みロックに失敗しました".to_string())
        })?;
        
        let now = Utc::now();
        let expired_sessions: Vec<String> = sessions.iter()
            .filter(|(_, s)| match s.expires_at {
                Some(expiry) => expiry < now,
                None => false,
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in &expired_sessions {
            sessions.remove(id);
        }
        
        Ok(expired_sessions.len())
    }

    /// 指定したエンティティが指定した権限を持っているかを確認する
    pub fn check_permission(
        &self,
        entity_id: &str,
        permission: &Permission,
    ) -> SecurityResult<bool> {
        // エンティティを取得
        let entity = self.get_entity(entity_id)?;
        
        // エンティティが無効の場合は権限なし
        if !entity.enabled {
            return Ok(false);
        }
        
        // エンティティに明示的に拒否されている権限はポリシーに関わらず拒否
        if entity.denied_permissions.contains(permission) {
            return Ok(false);
        }
        
        // エンティティに明示的に付与されている権限はポリシーに関わらず許可
        if entity.granted_permissions.contains(permission) {
            return Ok(true);
        }
        
        // デフォルトポリシーによる評価
        let default_policy = self.get_default_policy()?;
        let has_permission = default_policy.evaluate_permission(permission, entity.security_level);
        
        Ok(has_permission)
    }

    /// 権限を要求する（失敗時はエラー）
    pub fn require_permission(
        &self,
        entity_id: &str,
        permission: &Permission,
    ) -> SecurityResult<()> {
        let has_permission = self.check_permission(entity_id, permission)?;
        
        if !has_permission {
            return Err(SecurityError::PermissionError(
                format!("エンティティ '{}' は権限 '{:?}' を持っていません", entity_id, permission)
            ));
        }
        
        Ok(())
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_security_context() -> SecurityContext {
        let ctx = SecurityContext::new();
        let _ = ctx.initialize();
        ctx
    }

    #[test]
    fn test_security_context_initialization() {
        let ctx = SecurityContext::new();
        let result = ctx.initialize();
        assert!(result.is_ok());
        
        // デフォルトポリシーが設定されているか確認
        let policy = ctx.get_default_policy();
        assert!(policy.is_ok());
        
        // システムエンティティが作成されているか確認
        let entity = ctx.get_entity("system");
        assert!(entity.is_ok());
        assert_eq!(entity.unwrap().entity_type, EntityType::SystemService);
    }

    #[test]
    fn test_entity_permission_management() {
        let ctx = create_test_security_context();
        
        // テストユーザーを作成
        let user = SecurityEntity::new(
            "user1".to_string(),
            "テストユーザー".to_string(),
            EntityType::User,
            SecurityLevel::Standard,
        );
        let result = ctx.register_entity(user);
        assert!(result.is_ok());
        
        // 権限を付与
        ctx.update_entity("user1", |entity| {
            entity.grant_permission(Permission::FileRead);
            entity.grant_permission(Permission::NetworkConnect);
        }).unwrap();
        
        // 権限チェック
        assert!(ctx.check_permission("user1", &Permission::FileRead).unwrap());
        assert!(ctx.check_permission("user1", &Permission::NetworkConnect).unwrap());
        assert!(!ctx.check_permission("user1", &Permission::SystemAdmin).unwrap());
        
        // 権限を拒否
        ctx.update_entity("user1", |entity| {
            entity.deny_permission(Permission::NetworkConnect);
        }).unwrap();
        
        // 拒否された権限は無効になっていることを確認
        assert!(ctx.check_permission("user1", &Permission::FileRead).unwrap());
        assert!(!ctx.check_permission("user1", &Permission::NetworkConnect).unwrap());
        
        // require_permissionのテスト
        assert!(ctx.require_permission("user1", &Permission::FileRead).is_ok());
        assert!(ctx.require_permission("user1", &Permission::NetworkConnect).is_err());
    }

    #[test]
    fn test_policy_management() {
        let ctx = create_test_security_context();
        
        // カスタムポリシーを作成
        let mut custom_policy = SecurityPolicy::new(
            "custom_policy".to_string(),
            "カスタムポリシー".to_string(),
            "テスト用カスタムポリシー".to_string(),
            "1.0.0".to_string(),
        );
        
        // ポリシーにルールを追加
        let rule = super::super::policy::PolicyRule::new(
            "test_rule".to_string(),
            "テストルール".to_string(),
            "テスト用ルール".to_string(),
            PolicyType::Custom("test".to_string()),
            50,
            [Permission::FileRead].iter().cloned().collect(),
            SecurityLevel::Low,
        );
        let _ = custom_policy.add_rule(rule);
        
        // ポリシーを登録
        let result = ctx.register_policy(custom_policy);
        assert!(result.is_ok());
        
        // 登録したポリシーを取得
        let policy = ctx.get_policy("custom_policy");
        assert!(policy.is_ok());
        assert_eq!(policy.unwrap().name, "カスタムポリシー");
        
        // ポリシーを更新
        ctx.update_policy("custom_policy", |policy| {
            policy.set_enabled(false);
        }).unwrap();
        
        // 更新が反映されているか確認
        let policy = ctx.get_policy("custom_policy").unwrap();
        assert!(!policy.enabled);
        
        // カスタムポリシーをデフォルトに設定
        let result = ctx.set_default_policy("custom_policy");
        assert!(result.is_ok());
        
        // デフォルトポリシーが変更されているか確認
        let default = ctx.get_default_policy().unwrap();
        assert_eq!(default.id, "custom_policy");
    }

    #[test]
    fn test_session_management() {
        let ctx = create_test_security_context();
        
        // セッションを作成（30秒で期限切れ）
        let session = ctx.create_session(AuthenticationType::Password, Some(30)).unwrap();
        let session_id = session.id.clone();
        
        // セッションが有効か確認
        assert!(session.is_valid());
        
        // セッションを取得
        let session = ctx.get_session(&session_id).unwrap();
        assert_eq!(session.auth_type, AuthenticationType::Password);
        
        // セッションを更新
        ctx.update_session(&session_id, |s| {
            s.update_activity();
            s.set_metadata("test_key".to_string(), "test_value".to_string());
        }).unwrap();
        
        // 更新が反映されているか確認
        let session = ctx.get_session(&session_id).unwrap();
        assert_eq!(session.get_metadata("test_key").unwrap(), "test_value");
        
        // 期限切れセッションのクリーンアップ
        let removed = ctx.cleanup_expired_sessions().unwrap();
        assert_eq!(removed, 0); // まだ期限切れではない
        
        // セッションを削除
        let result = ctx.remove_session(&session_id);
        assert!(result.is_ok());
        
        // 削除されたセッションは取得できないことを確認
        let result = ctx.get_session(&session_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_security_levels() {
        assert!(SecurityLevel::Highest > SecurityLevel::High);
        assert!(SecurityLevel::High > SecurityLevel::Standard);
        assert!(SecurityLevel::Standard > SecurityLevel::Low);
        assert!(SecurityLevel::Low > SecurityLevel::Minimal);
        
        let default_level = SecurityLevel::default();
        assert_eq!(default_level, SecurityLevel::Standard);
    }

    #[test]
    fn test_entity_by_type() {
        let ctx = create_test_security_context();
        
        // アプリケーションエンティティを作成
        let app1 = SecurityEntity::new(
            "app1".to_string(),
            "アプリ1".to_string(),
            EntityType::Application,
            SecurityLevel::Standard,
        );
        let app2 = SecurityEntity::new(
            "app2".to_string(),
            "アプリ2".to_string(),
            EntityType::Application,
            SecurityLevel::Standard,
        );
        
        // ユーザーエンティティを作成
        let user1 = SecurityEntity::new(
            "test_user1".to_string(),
            "テストユーザー1".to_string(),
            EntityType::User,
            SecurityLevel::Standard,
        );
        
        // エンティティを登録
        let _ = ctx.register_entity(app1);
        let _ = ctx.register_entity(app2);
        let _ = ctx.register_entity(user1);
        
        // アプリケーション型のエンティティを取得
        let apps = ctx.get_entities_by_type(&EntityType::Application).unwrap();
        assert_eq!(apps.len(), 2);
        assert!(apps.iter().any(|e| e.id == "app1"));
        assert!(apps.iter().any(|e| e.id == "app2"));
        
        // ユーザー型のエンティティを取得
        let users = ctx.get_entities_by_type(&EntityType::User).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, "test_user1");
    }
} 