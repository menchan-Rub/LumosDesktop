// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
//
// セキュリティコンテキスト
// Copyright (c) 2023-2024 AetherOS Team.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex, RwLock};
use log::{debug, info, warn, error};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;

use crate::system::security::{
    SecurityError, 
    SecurityResult,
    permissions::Permission,
};

/// セキュリティレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// 最低権限（サンドボックス化されたアプリケーション）
    Untrusted = 0,
    
    /// 標準権限（一般アプリケーション）
    Standard = 1,
    
    /// 高度な権限（システムコンポーネント）
    Elevated = 2,
    
    /// 最高権限（システム管理者）
    System = 3,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        SecurityLevel::Standard
    }
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityLevel::Untrusted => write!(f, "Untrusted"),
            SecurityLevel::Standard => write!(f, "Standard"),
            SecurityLevel::Elevated => write!(f, "Elevated"),
            SecurityLevel::System => write!(f, "System"),
        }
    }
}

/// セキュリティトークン
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecurityToken {
    /// トークンID
    id: String,
    
    /// 発行者
    issuer: String,
    
    /// 対象
    subject: String,
    
    /// セキュリティレベル
    level: SecurityLevel,
    
    /// 発行日時
    issued_at: SystemTime,
    
    /// 有効期限
    expires_at: Option<SystemTime>,
    
    /// 権限リスト
    permissions: HashSet<Permission>,
    
    /// 追加情報
    claims: HashMap<String, String>,
    
    /// 署名
    signature: String,
}

impl SecurityToken {
    /// 新しいセキュリティトークンを作成
    pub fn new(
        issuer: &str,
        subject: &str,
        level: SecurityLevel,
        ttl: Option<Duration>,
        permissions: HashSet<Permission>,
        claims: HashMap<String, String>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let issued_at = SystemTime::now();
        let expires_at = ttl.map(|ttl| issued_at + ttl);
        
        let mut token = Self {
            id,
            issuer: issuer.to_string(),
            subject: subject.to_string(),
            level,
            issued_at,
            expires_at,
            permissions,
            claims,
            signature: String::new(),
        };
        
        // 署名を生成
        token.signature = token.generate_signature();
        
        token
    }
    
    /// トークンが有効かどうかを確認
    pub fn is_valid(&self) -> bool {
        // 有効期限チェック
        if let Some(expires_at) = self.expires_at {
            if SystemTime::now() > expires_at {
                return false;
            }
        }
        
        // 署名の検証
        let expected_signature = self.generate_signature();
        self.signature == expected_signature
    }
    
    /// トークンが指定されたセキュリティレベル以上かどうかを確認
    pub fn has_level(&self, level: SecurityLevel) -> bool {
        self.level >= level
    }
    
    /// トークンが指定された権限を持っているかどうかを確認
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
    
    /// トークンID取得
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// 発行者取得
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    
    /// 対象取得
    pub fn subject(&self) -> &str {
        &self.subject
    }
    
    /// セキュリティレベル取得
    pub fn level(&self) -> SecurityLevel {
        self.level
    }
    
    /// 発行日時取得
    pub fn issued_at(&self) -> SystemTime {
        self.issued_at
    }
    
    /// 有効期限取得
    pub fn expires_at(&self) -> Option<SystemTime> {
        self.expires_at
    }
    
    /// 権限リスト取得
    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }
    
    /// 追加情報取得
    pub fn claims(&self) -> &HashMap<String, String> {
        &self.claims
    }
    
    /// 追加情報を取得
    pub fn get_claim(&self, key: &str) -> Option<&String> {
        self.claims.get(key)
    }
    
    /// 署名を生成
    fn generate_signature(&self) -> String {
        let mut hasher = Sha256::new();
        
        hasher.update(self.id.as_bytes());
        hasher.update(self.issuer.as_bytes());
        hasher.update(self.subject.as_bytes());
        hasher.update(&[self.level as u8]);
        
        // 発行日時
        if let Ok(duration) = self.issued_at.duration_since(SystemTime::UNIX_EPOCH) {
            hasher.update(&duration.as_secs().to_le_bytes());
        }
        
        // 有効期限
        if let Some(expires_at) = self.expires_at {
            if let Ok(duration) = expires_at.duration_since(SystemTime::UNIX_EPOCH) {
                hasher.update(&duration.as_secs().to_le_bytes());
            }
        }
        
        // 権限
        let mut sorted_permissions: Vec<_> = self.permissions.iter().collect();
        sorted_permissions.sort();
        for permission in sorted_permissions {
            hasher.update(permission.to_string().as_bytes());
        }
        
        // 追加情報
        let mut sorted_claims: Vec<_> = self.claims.iter().collect();
        sorted_claims.sort_by_key(|k| k.0);
        for (key, value) in sorted_claims {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }
        
        // HMAC-SHA256とする（実際の実装では適切な秘密鍵を使用）
        let hash = hasher.finalize();
        hex::encode(hash)
    }
}

/// セキュリティポリシー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// ポリシーID
    id: String,
    
    /// ポリシー名
    name: String,
    
    /// 説明
    description: String,
    
    /// 最小セキュリティレベル
    min_level: SecurityLevel,
    
    /// 必要な権限リスト
    required_permissions: HashSet<Permission>,
    
    /// 追加条件
    conditions: HashMap<String, String>,
    
    /// 有効かどうか
    enabled: bool,
}

impl SecurityPolicy {
    /// 新しいセキュリティポリシーを作成
    pub fn new(
        name: &str,
        description: &str,
        min_level: SecurityLevel,
        required_permissions: HashSet<Permission>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            min_level,
            required_permissions,
            conditions: HashMap::new(),
            enabled: true,
        }
    }
    
    /// ポリシーが有効かどうかを確認
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// ポリシーを有効にする
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// ポリシーを無効にする
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// トークンがポリシーに適合しているかどうかを確認
    pub fn check_compliance(&self, token: &SecurityToken) -> bool {
        if !self.enabled {
            return false;
        }
        
        // セキュリティレベルチェック
        if token.level() < self.min_level {
            return false;
        }
        
        // 権限チェック
        for permission in &self.required_permissions {
            if !token.has_permission(permission) {
                return false;
            }
        }
        
        // 追加条件チェック
        for (key, value) in &self.conditions {
            if let Some(claim) = token.get_claim(key) {
                if claim != value {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
}

/// セキュリティコンテキストトレイト
pub trait SecurityContext {
    /// セキュリティトークンを作成
    fn create_token(
        &self,
        subject: &str,
        level: SecurityLevel,
        ttl: Option<Duration>,
        permissions: HashSet<Permission>,
        claims: HashMap<String, String>,
    ) -> SecurityResult<SecurityToken>;
    
    /// セキュリティトークンを検証
    fn verify_token(&self, token: &SecurityToken) -> SecurityResult<bool>;
    
    /// セキュリティポリシーを追加
    fn add_policy(&self, policy: SecurityPolicy) -> SecurityResult<()>;
    
    /// セキュリティポリシーを取得
    fn get_policy(&self, id: &str) -> SecurityResult<Option<SecurityPolicy>>;
    
    /// セキュリティポリシーを削除
    fn remove_policy(&self, id: &str) -> SecurityResult<()>;
    
    /// トークンがポリシーに適合しているかどうかを確認
    fn check_policy_compliance(&self, token: &SecurityToken, policy_id: &str) -> SecurityResult<bool>;
    
    /// 権限を昇格
    fn elevate_privileges(
        &self,
        token: &SecurityToken,
        target_level: SecurityLevel,
        reason: &str,
    ) -> SecurityResult<SecurityToken>;
    
    /// 権限を降格
    fn drop_privileges(
        &self,
        token: &SecurityToken,
        target_level: SecurityLevel,
    ) -> SecurityResult<SecurityToken>;
}

/// セキュリティコンテキスト実装
pub struct SecurityContextImpl {
    /// コンテキスト名
    name: String,
    
    /// 発行者ID
    issuer_id: String,
    
    /// 初期化状態
    initialized: bool,
    
    /// トークンデータベース
    tokens: HashMap<String, SecurityToken>,
    
    /// ポリシーデータベース
    policies: HashMap<String, SecurityPolicy>,
    
    /// 秘密鍵（実際の実装では適切な暗号化キーを使用）
    secret_key: String,
}

impl SecurityContextImpl {
    /// 新しいセキュリティコンテキストを作成
    pub fn new() -> Self {
        Self {
            name: "AetherOS.SecurityContext".to_string(),
            issuer_id: "aetheros.system".to_string(),
            initialized: false,
            tokens: HashMap::new(),
            policies: HashMap::new(),
            secret_key: String::new(),
        }
    }
    
    /// セキュリティコンテキストを初期化
    pub fn initialize(&mut self, config_path: Option<&str>) -> SecurityResult<()> {
        if self.initialized {
            return Err(SecurityError::InitializationError("すでに初期化されています".to_string()));
        }
        
        info!("セキュリティコンテキストを初期化中...");
        
        // 秘密鍵を生成
        self.generate_secret_key();
        
        // 設定の読み込み
        if let Some(path) = config_path {
            self.load_config(path)?;
        } else {
            self.load_default_config()?;
        }
        
        self.initialized = true;
        info!("セキュリティコンテキストの初期化が完了しました");
        
        Ok(())
    }
    
    /// 秘密鍵を生成
    fn generate_secret_key(&mut self) {
        let mut rng = thread_rng();
        let key: String = (0..64)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();
        self.secret_key = key;
    }
    
    /// 設定を読み込む
    fn load_config(&mut self, path: &str) -> SecurityResult<()> {
        if !Path::new(path).exists() {
            return Err(SecurityError::InitializationError(format!("設定ファイルが見つかりません: {}", path)));
        }
        
        // 実際の実装では設定ファイルからロードする
        self.load_default_config()
    }
    
    /// デフォルト設定を読み込む
    fn load_default_config(&mut self) -> SecurityResult<()> {
        // デフォルトポリシーの追加
        let mut required_permissions = HashSet::new();
        required_permissions.insert(Permission::new("system.read"));
        
        let policy = SecurityPolicy::new(
            "default.system.read",
            "システム情報の読み取り",
            SecurityLevel::Standard,
            required_permissions,
        );
        
        self.policies.insert(policy.id.clone(), policy);
        
        let mut required_permissions = HashSet::new();
        required_permissions.insert(Permission::new("system.write"));
        
        let policy = SecurityPolicy::new(
            "default.system.write",
            "システム情報の書き込み",
            SecurityLevel::Elevated,
            required_permissions,
        );
        
        self.policies.insert(policy.id.clone(), policy);
        
        let mut required_permissions = HashSet::new();
        required_permissions.insert(Permission::new("system.admin"));
        
        let policy = SecurityPolicy::new(
            "default.system.admin",
            "システム管理",
            SecurityLevel::System,
            required_permissions,
        );
        
        self.policies.insert(policy.id.clone(), policy);
        
        Ok(())
    }
    
    /// トークンを登録
    fn register_token(&mut self, token: SecurityToken) -> SecurityResult<()> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        self.tokens.insert(token.id.clone(), token);
        Ok(())
    }
    
    /// トークンを取得
    fn get_token(&self, token_id: &str) -> SecurityResult<Option<SecurityToken>> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        Ok(self.tokens.get(token_id).cloned())
    }
    
    /// トークンを無効化
    fn invalidate_token(&mut self, token_id: &str) -> SecurityResult<()> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        self.tokens.remove(token_id);
        Ok(())
    }
}

impl SecurityContext for SecurityContextImpl {
    fn create_token(
        &self,
        subject: &str,
        level: SecurityLevel,
        ttl: Option<Duration>,
        permissions: HashSet<Permission>,
        claims: HashMap<String, String>,
    ) -> SecurityResult<SecurityToken> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        let token = SecurityToken::new(
            &self.issuer_id,
            subject,
            level,
            ttl,
            permissions,
            claims,
        );
        
        Ok(token)
    }
    
    fn verify_token(&self, token: &SecurityToken) -> SecurityResult<bool> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        // 発行者の確認
        if token.issuer() != self.issuer_id {
            return Ok(false);
        }
        
        // トークンの有効性を確認
        if !token.is_valid() {
            return Ok(false);
        }
        
        // 登録済みトークンの確認
        if let Some(registered_token) = self.tokens.get(token.id()) {
            return Ok(registered_token == token);
        }
        
        Ok(false)
    }
    
    fn add_policy(&self, policy: SecurityPolicy) -> SecurityResult<()> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        // 実際の実装では相互排除を適切に処理する必要がある
        let mut policies = self.policies.clone();
        policies.insert(policy.id.clone(), policy);
        
        Ok(())
    }
    
    fn get_policy(&self, id: &str) -> SecurityResult<Option<SecurityPolicy>> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        Ok(self.policies.get(id).cloned())
    }
    
    fn remove_policy(&self, id: &str) -> SecurityResult<()> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        // 実際の実装では相互排除を適切に処理する必要がある
        let mut policies = self.policies.clone();
        policies.remove(id);
        
        Ok(())
    }
    
    fn check_policy_compliance(&self, token: &SecurityToken, policy_id: &str) -> SecurityResult<bool> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        // トークンの有効性を確認
        if !token.is_valid() {
            return Ok(false);
        }
        
        // ポリシーを取得
        let policy = match self.policies.get(policy_id) {
            Some(policy) => policy,
            None => return Err(SecurityError::InvalidContext(format!("ポリシーが見つかりません: {}", policy_id))),
        };
        
        // ポリシー適合性を確認
        Ok(policy.check_compliance(token))
    }
    
    fn elevate_privileges(
        &self,
        token: &SecurityToken,
        target_level: SecurityLevel,
        reason: &str,
    ) -> SecurityResult<SecurityToken> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        // トークンの有効性を確認
        if !token.is_valid() {
            return Err(SecurityError::AuthenticationError("無効なトークンです".to_string()));
        }
        
        // 現在のレベルを確認
        if token.level() >= target_level {
            return Ok(token.clone());
        }
        
        // 実際の実装では特権昇格の条件をチェックする
        // ここでは簡易的な実装として、理由を記録するだけ
        let mut claims = token.claims().clone();
        claims.insert("elevation_reason".to_string(), reason.to_string());
        claims.insert("elevation_time".to_string(), SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string());
        
        // 新しいトークンを作成
        let new_token = SecurityToken::new(
            token.issuer(),
            token.subject(),
            target_level,
            token.expires_at().map(|t| t.duration_since(SystemTime::now()).unwrap_or(Duration::from_secs(3600))),
            token.permissions().clone(),
            claims,
        );
        
        Ok(new_token)
    }
    
    fn drop_privileges(
        &self,
        token: &SecurityToken,
        target_level: SecurityLevel,
    ) -> SecurityResult<SecurityToken> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        // トークンの有効性を確認
        if !token.is_valid() {
            return Err(SecurityError::AuthenticationError("無効なトークンです".to_string()));
        }
        
        // 現在のレベルを確認
        if token.level() <= target_level {
            return Ok(token.clone());
        }
        
        // 新しいトークンを作成
        let new_token = SecurityToken::new(
            token.issuer(),
            token.subject(),
            target_level,
            token.expires_at().map(|t| t.duration_since(SystemTime::now()).unwrap_or(Duration::from_secs(3600))),
            token.permissions().clone(),
            token.claims().clone(),
        );
        
        Ok(new_token)
    }
}

// シングルトンインスタンス
lazy_static::lazy_static! {
    static ref SECURITY_CONTEXT: RwLock<SecurityContextImpl> = RwLock::new(SecurityContextImpl::new());
}

/// グローバルセキュリティコンテキストを初期化
pub fn initialize_security_context(config_path: Option<&str>) -> SecurityResult<()> {
    let mut context = SECURITY_CONTEXT.write().map_err(|_| {
        SecurityError::InternalError("セキュリティコンテキストのロックに失敗しました".to_string())
    })?;
    
    context.initialize(config_path)
}

/// グローバルセキュリティコンテキストを取得
pub fn get_security_context() -> SecurityResult<Arc<RwLock<dyn SecurityContext + Send + Sync>>> {
    let context = Arc::new(SECURITY_CONTEXT.clone()) as Arc<RwLock<dyn SecurityContext + Send + Sync>>;
    Ok(context)
}

/// セキュリティトークンを作成
pub fn create_token(
    subject: &str,
    level: SecurityLevel,
    ttl: Option<Duration>,
    permissions: HashSet<Permission>,
    claims: HashMap<String, String>,
) -> SecurityResult<SecurityToken> {
    let context = SECURITY_CONTEXT.read().map_err(|_| {
        SecurityError::InternalError("セキュリティコンテキストのロックに失敗しました".to_string())
    })?;
    
    context.create_token(subject, level, ttl, permissions, claims)
}

/// セキュリティトークンを検証
pub fn verify_token(token: &SecurityToken) -> SecurityResult<bool> {
    let context = SECURITY_CONTEXT.read().map_err(|_| {
        SecurityError::InternalError("セキュリティコンテキストのロックに失敗しました".to_string())
    })?;
    
    context.verify_token(token)
}

/// 権限を昇格
pub fn elevate_privileges(
    token: &SecurityToken,
    target_level: SecurityLevel,
    reason: &str,
) -> SecurityResult<SecurityToken> {
    let context = SECURITY_CONTEXT.read().map_err(|_| {
        SecurityError::InternalError("セキュリティコンテキストのロックに失敗しました".to_string())
    })?;
    
    context.elevate_privileges(token, target_level, reason)
}

/// 権限を降格
pub fn drop_privileges(
    token: &SecurityToken,
    target_level: SecurityLevel,
) -> SecurityResult<SecurityToken> {
    let context = SECURITY_CONTEXT.read().map_err(|_| {
        SecurityError::InternalError("セキュリティコンテキストのロックに失敗しました".to_string())
    })?;
    
    context.drop_privileges(token, target_level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_security_token() {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::new("system.read"));
        
        let mut claims = HashMap::new();
        claims.insert("app".to_string(), "test-app".to_string());
        
        let token = SecurityToken::new(
            "test-issuer",
            "test-subject",
            SecurityLevel::Standard,
            Some(Duration::from_secs(3600)),
            permissions,
            claims,
        );
        
        assert_eq!(token.issuer(), "test-issuer");
        assert_eq!(token.subject(), "test-subject");
        assert_eq!(token.level(), SecurityLevel::Standard);
        assert!(token.is_valid());
        
        // 権限チェック
        assert!(token.has_permission(&Permission::new("system.read")));
        assert!(!token.has_permission(&Permission::new("system.write")));
        
        // 権限レベルチェック
        assert!(token.has_level(SecurityLevel::Untrusted));
        assert!(token.has_level(SecurityLevel::Standard));
        assert!(!token.has_level(SecurityLevel::Elevated));
        assert!(!token.has_level(SecurityLevel::System));
        
        // クレームチェック
        assert_eq!(token.get_claim("app"), Some(&"test-app".to_string()));
        assert_eq!(token.get_claim("none"), None);
    }
    
    #[test]
    fn test_security_policy() {
        let mut required_permissions = HashSet::new();
        required_permissions.insert(Permission::new("system.read"));
        
        let mut policy = SecurityPolicy::new(
            "test-policy",
            "テストポリシー",
            SecurityLevel::Standard,
            required_permissions,
        );
        
        assert!(policy.is_enabled());
        
        policy.disable();
        assert!(!policy.is_enabled());
        
        policy.enable();
        assert!(policy.is_enabled());
        
        // トークン作成
        let mut permissions = HashSet::new();
        permissions.insert(Permission::new("system.read"));
        
        let mut claims = HashMap::new();
        claims.insert("app".to_string(), "test-app".to_string());
        
        let token = SecurityToken::new(
            "test-issuer",
            "test-subject",
            SecurityLevel::Standard,
            Some(Duration::from_secs(3600)),
            permissions,
            claims,
        );
        
        // ポリシー適合チェック
        assert!(policy.check_compliance(&token));
        
        // レベル不足のトークン
        let token_low = SecurityToken::new(
            "test-issuer",
            "test-subject",
            SecurityLevel::Untrusted,
            Some(Duration::from_secs(3600)),
            permissions.clone(),
            claims.clone(),
        );
        
        assert!(!policy.check_compliance(&token_low));
        
        // 権限不足のトークン
        let token_no_perm = SecurityToken::new(
            "test-issuer",
            "test-subject",
            SecurityLevel::Standard,
            Some(Duration::from_secs(3600)),
            HashSet::new(),
            claims.clone(),
        );
        
        assert!(!policy.check_compliance(&token_no_perm));
    }
    
    #[test]
    fn test_security_context_impl() {
        let mut context = SecurityContextImpl::new();
        
        // 初期化前はエラー
        let mut permissions = HashSet::new();
        permissions.insert(Permission::new("system.read"));
        
        let result = context.create_token(
            "test-subject",
            SecurityLevel::Standard,
            Some(Duration::from_secs(3600)),
            permissions.clone(),
            HashMap::new(),
        );
        
        assert!(result.is_err());
        
        // 初期化
        let result = context.initialize(None);
        assert!(result.is_ok());
        
        // トークン作成
        let result = context.create_token(
            "test-subject",
            SecurityLevel::Standard,
            Some(Duration::from_secs(3600)),
            permissions.clone(),
            HashMap::new(),
        );
        
        assert!(result.is_ok());
        let token = result.unwrap();
        
        // トークン検証（登録されていないため失敗）
        let result = context.verify_token(&token);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        
        // 権限昇格
        let result = context.elevate_privileges(&token, SecurityLevel::Elevated, "テスト");
        assert!(result.is_ok());
        let elevated_token = result.unwrap();
        
        assert_eq!(elevated_token.level(), SecurityLevel::Elevated);
        assert!(elevated_token.get_claim("elevation_reason").is_some());
        
        // 権限降格
        let result = context.drop_privileges(&elevated_token, SecurityLevel::Standard);
        assert!(result.is_ok());
        let dropped_token = result.unwrap();
        
        assert_eq!(dropped_token.level(), SecurityLevel::Standard);
    }
    
    #[test]
    fn test_global_security_context() {
        // 初期化
        let result = initialize_security_context(None);
        assert!(result.is_ok());
        
        // トークン作成
        let mut permissions = HashSet::new();
        permissions.insert(Permission::new("system.read"));
        
        let result = create_token(
            "test-subject",
            SecurityLevel::Standard,
            Some(Duration::from_secs(3600)),
            permissions.clone(),
            HashMap::new(),
        );
        
        assert!(result.is_ok());
        let token = result.unwrap();
        
        // トークン検証
        let result = verify_token(&token);
        assert!(result.is_ok());
        
        // 権限昇格
        let result = elevate_privileges(&token, SecurityLevel::Elevated, "テスト");
        assert!(result.is_ok());
        let elevated_token = result.unwrap();
        
        assert_eq!(elevated_token.level(), SecurityLevel::Elevated);
        
        // 権限降格
        let result = drop_privileges(&elevated_token, SecurityLevel::Standard);
        assert!(result.is_ok());
        let dropped_token = result.unwrap();
        
        assert_eq!(dropped_token.level(), SecurityLevel::Standard);
    }
} 