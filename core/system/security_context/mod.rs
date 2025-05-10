//! LumosDesktop Security Context Module
//!
//! このモジュールはアプリケーションのセキュリティコンテキストを管理します。
//! セキュリティポリシー、アクセス制御、権限管理などの機能を提供します。

pub mod permission;
pub mod policy;
pub mod sandbox;
pub mod audit;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use log::{debug, error, info, warn};
use uuid::Uuid;

use crate::core::system::logging;
use crate::core::system::process::ProcessId;
use permission::{Permission, PermissionSet, PermissionManager};
use policy::{PolicyManager, PolicyType, PolicyTarget, PolicyEvaluationContext};

/// セキュリティレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SecurityLevel {
    Root = 0,      // システム全体の制御権限
    Admin = 1,     // 管理者権限
    Normal = 2,    // 通常ユーザー権限
    Restricted = 3, // 制限付き権限
}

impl Default for SecurityLevel {
    fn default() -> Self {
        SecurityLevel::Normal
    }
}

impl SecurityLevel {
    /// セキュリティレベルの文字列表現を取得
    pub fn to_string(&self) -> &'static str {
        match self {
            SecurityLevel::Root => "root",
            SecurityLevel::Admin => "admin",
            SecurityLevel::Normal => "normal",
            SecurityLevel::Restricted => "restricted",
        }
    }
    
    /// 文字列からセキュリティレベルを取得
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "root" => Some(SecurityLevel::Root),
            "admin" => Some(SecurityLevel::Admin),
            "normal" => Some(SecurityLevel::Normal),
            "restricted" => Some(SecurityLevel::Restricted),
            _ => None,
        }
    }
    
    /// このセキュリティレベルが指定されたレベル以上かどうかを確認
    pub fn is_at_least(&self, level: SecurityLevel) -> bool {
        // 数値が小さいほど権限が高い
        *self as u8 <= level as u8
    }
    
    /// 指定されたレベルが現在のレベルより高い（数値が小さい）か同等かをチェック
    pub fn has_privilege_for(&self, required: SecurityLevel) -> bool {
        *self as u8 <= required as u8
    }
}

/// 認証情報
#[derive(Debug, Clone)]
pub struct Credentials {
    /// ユーザーID
    pub user_id: String,
    /// プロセスID
    pub process_id: Option<u32>,
    /// アプリケーションID
    pub app_id: Option<String>,
    /// セキュリティレベル
    pub security_level: SecurityLevel,
    /// セッションID
    pub session_id: Option<String>,
    /// 発行時刻
    pub issued_at: SystemTime,
    /// 有効期限
    pub expires_at: Option<SystemTime>,
    /// 追加データ
    pub metadata: HashMap<String, String>,
}

impl Credentials {
    /// 新しい認証情報を作成
    pub fn new(
        user_id: String,
        security_level: SecurityLevel,
    ) -> Self {
        Credentials {
            user_id,
            process_id: None,
            app_id: None,
            security_level,
            session_id: None,
            issued_at: SystemTime::now(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }
    
    /// 有効期限を設定
    pub fn with_expiration(mut self, expires_at: SystemTime) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// メタデータを追加
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// 認証情報が有効かどうかを確認
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expiry) => SystemTime::now() < expiry,
            None => true,
        }
    }
    
    /// ユーザーIDを取得
    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    
    /// アプリケーションIDを取得
    pub fn app_id(&self) -> Option<&str> {
        self.app_id.as_deref()
    }
    
    /// セキュリティレベルを取得
    pub fn security_level(&self) -> SecurityLevel {
        self.security_level
    }
    
    /// セッションIDを取得
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}

/// セキュリティコンテキスト
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// 認証情報
    pub credentials: Credentials,
    /// 許可された権限セット
    pub permissions: HashSet<String>,
    /// セキュリティコンテキストID
    pub context_id: String,
    /// 親コンテキストID（あれば）
    pub parent_context_id: Option<String>,
    /// サンドボックス内かどうか
    pub is_sandboxed: bool,
    /// 作成時刻
    pub created_at: SystemTime,
    /// 更新時刻
    pub updated_at: SystemTime,
}

impl SecurityContext {
    /// 新しいセキュリティコンテキストを作成
    pub fn new(
        credentials: Credentials,
        parent_context_id: Option<String>,
    ) -> Self {
        let now = SystemTime::now();
        SecurityContext {
            credentials,
            permissions: HashSet::new(),
            context_id: Uuid::new_v4().to_string(),
            parent_context_id,
            is_sandboxed: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// 親コンテキストを設定
    pub fn with_parent(mut self, parent_context_id: String) -> Self {
        self.parent_context_id = Some(parent_context_id);
        self
    }
    
    /// サンドボックスを設定
    pub fn with_sandbox(mut self, sandboxed: bool) -> Self {
        self.is_sandboxed = sandboxed;
        self.updated_at = SystemTime::now();
        self
    }
    
    /// 指定された権限を持っているかどうかを確認
    pub fn has_permission(&self, permission: &str) -> bool {
        // Rootレベルは全ての権限を持つ
        if self.credentials.security_level == SecurityLevel::Root {
            return true;
        }
        
        // 特定の権限をチェック
        self.permissions.contains(permission)
    }
    
    /// コンテキストが有効かどうかを確認
    pub fn is_valid(&self) -> bool {
        self.credentials.is_valid()
    }
    
    /// セキュリティレベルを取得
    pub fn security_level(&self) -> SecurityLevel {
        self.credentials.security_level
    }
    
    /// ユーザーIDを取得
    pub fn user_id(&self) -> &str {
        self.credentials.user_id.as_str()
    }
    
    /// アプリケーションIDを取得
    pub fn app_id(&self) -> &str {
        self.credentials.app_id.as_ref().map(|s| s.as_str()).unwrap_or("")
    }
    
    /// コンテキストIDを取得
    pub fn id(&self) -> &str {
        &self.context_id
    }
    
    /// 親コンテキストIDを取得
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_context_id.as_deref()
    }
    
    /// サンドボックスフラグを取得
    pub fn is_sandboxed(&self) -> bool {
        self.is_sandboxed
    }
    
    /// コンテキストを更新
    pub fn update(&mut self) {
        self.updated_at = SystemTime::now();
    }
    
    /// 権限を付与
    pub fn grant_permission(&mut self, permission: String) {
        self.permissions.insert(permission);
        self.updated_at = SystemTime::now();
    }
    
    /// 権限を削除
    pub fn revoke_permission(&mut self, permission: &str) -> bool {
        let result = self.permissions.remove(permission);
        if result {
            self.updated_at = SystemTime::now();
        }
        result
    }
    
    /// 作成時刻を取得
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }
    
    /// 更新時刻を取得
    pub fn updated_at(&self) -> SystemTime {
        self.updated_at
    }
}

/// セキュリティマネージャー
/// システム全体のセキュリティを管理する中央コンポーネント
pub struct SecurityManager {
    /// 権限マネージャー
    permission_manager: Arc<PermissionManager>,
    /// ポリシーマネージャー
    policy_manager: Arc<PolicyManager>,
    /// アクティブなセキュリティコンテキスト
    active_contexts: RwLock<HashMap<String, Arc<Mutex<SecurityContext>>>>,
    /// サンドボックスの設定ディレクトリ
    sandbox_config_dir: Option<PathBuf>,
}

impl SecurityManager {
    /// 新しいセキュリティマネージャーを作成
    pub fn new() -> Self {
        let logger = logging::get_logger("security_manager");
        logging::debug!(logger, "SecurityManagerを初期化中...");
        
        Self {
            permission_manager: Arc::new(PermissionManager::new()),
            policy_manager: Arc::new(PolicyManager::new()),
            active_contexts: RwLock::new(HashMap::new()),
            sandbox_config_dir: None,
        }
    }
    
    /// 設定ディレクトリを設定
    pub fn with_config_dir(mut self, config_dir: PathBuf) -> Self {
        let policy_config_path = config_dir.join("policies.json");
        self.policy_manager = Arc::new(PolicyManager::with_config(policy_config_path));
        self.sandbox_config_dir = Some(config_dir.join("sandboxes"));
        self
    }
    
    /// 権限マネージャーを取得
    pub fn permission_manager(&self) -> Arc<PermissionManager> {
        self.permission_manager.clone()
    }
    
    /// ポリシーマネージャーを取得
    pub fn policy_manager(&self) -> Arc<PolicyManager> {
        self.policy_manager.clone()
    }
    
    /// 新しいセキュリティコンテキストを作成
    pub fn create_context(
        &self,
        credentials: Credentials,
        parent_context_id: Option<String>,
    ) -> Result<Arc<Mutex<SecurityContext>>, &'static str> {
        let logger = logging::get_logger("security_manager");
        
        // 資格情報の有効性をチェック
        if !credentials.is_valid() {
            return Err("Invalid credentials");
        }
        
        // 親コンテキストが存在する場合、その有効性をチェック
        if let Some(ref parent_id) = parent_context_id {
            let contexts = self.active_contexts.read().unwrap();
            if let Some(parent_context) = contexts.get(parent_id) {
                let parent = parent_context.lock().unwrap();
                if !parent.is_valid() {
                    return Err("Parent context is invalid");
                }
            } else {
                return Err("Parent context not found");
            }
        }
        
        // 新しいコンテキストを作成
        let mut context = SecurityContext::new(credentials.clone(), parent_context_id);
        
        // セキュリティレベルに基づいてデフォルトの権限を付与
        let policies = self.policy_manager.read().unwrap();
        if let Some(default_perms) = policies.get(&credentials.security_level) {
            for perm in default_perms {
                context.grant_permission(perm.clone());
            }
        }
        
        let context_id = context.id().to_string();
        debug!("Creating security context: {} for user: {}", context_id, credentials.user_id);
        
        let context = Arc::new(Mutex::new(context));
        
        // コンテキストを保存
        let mut contexts = self.active_contexts.write().unwrap();
        contexts.insert(context_id, Arc::clone(&context));
        
        Ok(context)
    }
    
    /// コンテキストを取得
    pub fn get_context(&self, context_id: &str) -> Option<Arc<Mutex<SecurityContext>>> {
        let contexts = self.active_contexts.read().unwrap();
        contexts.get(context_id).cloned()
    }
    
    /// コンテキストを削除
    pub fn remove_context(&self, context_id: &str) -> bool {
        let mut contexts = self.active_contexts.write().unwrap();
        if let Some(context) = contexts.remove(context_id) {
            let cred = context.lock().unwrap().credentials.clone();
            info!("Removed security context: {} for user: {}", context_id, cred.user_id);
            true
        } else {
            false
        }
    }
    
    /// コンテキストの権限チェック
    pub fn check_permission(&self, context_id: &str, permission: &str) -> bool {
        match self.get_context(context_id) {
            Some(context) => {
                let context = context.lock().unwrap();
                if !context.is_valid() {
                    debug!("Permission check failed: context {} is invalid", context_id);
                    return false;
                }
                
                let has_perm = context.has_permission(permission);
                debug!(
                    "Permission check for context {} and permission {}: {}",
                    context_id, permission, has_perm
                );
                has_perm
            }
            None => {
                debug!("Permission check failed: context {} not found", context_id);
                false
            }
        }
    }
    
    /// サンドボックスコンテキストを作成
    pub fn create_sandbox_context(
        &self,
        parent_context_id: &str,
        allowed_permissions: HashSet<String>,
    ) -> Result<Arc<Mutex<SecurityContext>>, &'static str> {
        let parent_context = match self.get_context(parent_context_id) {
            Some(ctx) => ctx,
            None => return Err("Parent context not found"),
        };
        
        let parent = parent_context.lock().unwrap();
        if !parent.is_valid() {
            return Err("Parent context is invalid");
        }
        
        // サンドボックスのクレデンシャルを作成（親から継承して制限付き）
        let mut credentials = parent.credentials.clone();
        
        // サンドボックスは常にRestricted権限
        let sandbox_credentials = Credentials {
            user_id: credentials.user_id.clone(),
            process_id: credentials.process_id,
            app_id: credentials.app_id.clone(),
            security_level: SecurityLevel::Restricted,
            session_id: credentials.session_id.clone(),
            issued_at: SystemTime::now(),
            expires_at: credentials.expires_at,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("sandbox".to_string(), "true".to_string());
                if let Some(app_id) = credentials.app_id {
                    meta.insert("sandbox_app".to_string(), app_id);
                }
                meta
            },
        };
        
        // サンドボックスコンテキストを作成
        let mut sandbox_context = SecurityContext::new(
            sandbox_credentials,
            Some(parent_context_id.to_string()),
        );
        
        // サンドボックスフラグを設定
        sandbox_context.with_sandbox(true);
        
        // 許可された権限のみを付与
        for permission in allowed_permissions {
            // 親コンテキストが持っている権限のみを付与可能
            if parent.has_permission(&permission) {
                sandbox_context.grant_permission(permission);
            }
        }
        
        let context_id = sandbox_context.id().to_string();
        info!(
            "Created sandbox context: {} from parent: {}",
            context_id, parent_context_id
        );
        
        let sandbox_context = Arc::new(Mutex::new(sandbox_context));
        
        // コンテキストを保存
        let mut contexts = self.active_contexts.write().unwrap();
        contexts.insert(context_id, Arc::clone(&sandbox_context));
        
        Ok(sandbox_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::system::process::ProcessId;
    
    #[test]
    fn test_security_level() {
        assert_eq!(SecurityLevel::Root.to_string(), "root");
        assert_eq!(SecurityLevel::from_string("admin"), Some(SecurityLevel::Admin));
        assert_eq!(SecurityLevel::from_string("invalid"), None);
        
        assert!(SecurityLevel::Root.is_at_least(SecurityLevel::Admin));
        assert!(SecurityLevel::Admin.is_at_least(SecurityLevel::Admin));
        assert!(!SecurityLevel::Normal.is_at_least(SecurityLevel::Admin));
    }
    
    #[test]
    fn test_credentials() {
        let creds = Credentials::new("user1".to_string(), SecurityLevel::Normal)
            .with_expiration(SystemTime::now() + Duration::from_secs(3600))
            .with_metadata("device".to_string(), "laptop".to_string());
        
        assert_eq!(creds.user_id, "user1");
        assert_eq!(creds.security_level, SecurityLevel::Normal);
        assert!(creds.is_valid());
        
        // セキュリティレベルに基づくデフォルト権限を取得
        let default_permissions = self.permission_manager
            .get_default_permissions(credentials.security_level)
            .ok_or_else(|| {
                format!("セキュリティレベル '{:?}' のデフォルト権限が見つかりません", 
                       credentials.security_level)
            })?;
        
        // 要求された権限とデフォルト権限をマージ
        let permissions = if let Some(req_perms) = requested_permissions {
            // 危険な権限がリクエストされているかチェック
            let dangerous_perms = req_perms.get_dangerous_permissions();
            
            if !dangerous_perms.is_empty() && credentials.security_level != SecurityLevel::Root {
                for perm in dangerous_perms {
                    logging::warn!(
                        logger,
                        "アプリケーション '{}' が危険な権限 '{:?}' をリクエストしました",
                        credentials.app_id,
                        perm
                    );
                }
            }
            
            // セキュリティレベルがAdminより低い場合、要求された権限を制限
            if credentials.security_level != SecurityLevel::Root 
               && credentials.security_level != SecurityLevel::Admin {
                
                // デフォルト権限との交差部分のみを許可
                default_permissions.intersection(&req_perms)
            } else {
                // Root/Adminはリクエストされた権限をすべて取得
                req_perms
            }
        } else {
            // リクエストがない場合はデフォルト権限を使用
            default_permissions
        };
        
        // コンテキストIDを生成
        let context_id = format!(
            "ctx_{}_{}_{}",
            credentials.user_id,
            credentials.app_id,
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or(""),
        );
        
        logging::info!(
            logger,
            "新しいセキュリティコンテキストを作成: {}, セキュリティレベル: {:?}, 権限数: {}",
            context_id,
            credentials.security_level,
            permissions.len(),
        );
        
        // コンテキストを作成
        let context = Arc::new(SecurityContext::new(
            credentials,
            permissions,
            context_id.clone(),
        ));
        
        // アクティブコンテキストに追加
        let mut contexts = self.active_contexts.write().map_err(|_| {
            "アクティブコンテキストへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        contexts.insert(context_id.clone(), context.clone());
        
        Ok(context)
    }
    
    /// コンテキストを取得
    pub fn get_context(&self, context_id: &str) -> Option<Arc<SecurityContext>> {
        if let Ok(contexts) = self.active_contexts.read() {
            contexts.get(context_id).cloned()
        } else {
            None
        }
    }
    
    /// コンテキストを削除
    pub fn remove_context(&self, context_id: &str) -> Result<(), String> {
        let mut contexts = self.active_contexts.write().map_err(|_| {
            "アクティブコンテキストへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        let removed = contexts.remove(context_id);
        
        if removed.is_none() {
            return Err(format!("コンテキスト '{}' が見つかりません", context_id));
        }
        
        Ok(())
    }
    
    /// コンテキストの権限チェック
    pub fn check_permission(
        &self,
        context: &SecurityContext,
        permission: &Permission,
        target: &PolicyTarget,
    ) -> Result<PolicyType, String> {
        let logger = logging::get_logger("security_manager");
        
        // コンテキストが有効かチェック
        if !context.is_valid() {
            return Err("無効なセキュリティコンテキストです".to_string());
        }
        
        // 権限があるかチェック
        if !context.has_permission(permission) {
            logging::warn!(
                logger,
                "コンテキスト '{}' には権限 '{:?}' がありません",
                context.context_id,
                permission
            );
            return Ok(PolicyType::Deny);
        }
        
        // ポリシー評価コンテキストを作成
        let eval_context = PolicyEvaluationContext::new()
            .with_user_id(context.credentials.user_id.clone())
            .with_current_time();
        
        // ポリシーを評価
        self.policy_manager.evaluate(permission, target, &eval_context)
    }
    
    /// サンドボックスコンテキストを作成
    pub fn create_sandbox_context(
        &self,
        parent_context: &SecurityContext,
        sandbox_name: &str,
        permissions: Option<PermissionSet>,
    ) -> Result<Arc<SecurityContext>, String> {
        let logger = logging::get_logger("security_manager");
        
        // 親コンテキストが有効かチェック
        if !parent_context.is_valid() {
            return Err("無効な親セキュリティコンテキストです".to_string());
        }
        
        // サンドボックスのパスを設定
        let sandbox_path = self.sandbox_config_dir.as_ref()
            .ok_or_else(|| "サンドボックス設定ディレクトリが設定されていません".to_string())?
            .join(format!("{}_{}", parent_context.app_id(), sandbox_name));
        
        // 権限を制限
        let sandbox_permissions = if let Some(perms) = permissions {
            // 親の権限とのインターセクションを取る（親が持っていない権限は付与できない）
            parent_context.permissions.intersection(&perms)
        } else {
            // デフォルトでは親の権限の一部を継承
            let mut restricted_permissions = parent_context.permissions.clone();
            
            // 危険な権限を削除
            for perm in restricted_permissions.get_dangerous_permissions() {
                restricted_permissions.remove(&perm);
            }
            
            restricted_permissions
        };
        
        // 認証情報を作成（親と同じユーザー・アプリだが、制限されたセキュリティレベル）
        let credentials = Credentials::new(
            parent_context.credentials.user_id.clone(),
            parent_context.credentials.process_id.clone(),
            format!("{}_sandbox_{}", parent_context.credentials.app_id, sandbox_name),
            SecurityLevel::Restricted,
            format!("session_{}", uuid::Uuid::new_v4()),
        );
        
        // コンテキストIDを生成
        let context_id = format!(
            "sandbox_{}_{}_{}",
            credentials.user_id,
            credentials.app_id,
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or(""),
        );
        
        logging::info!(
            logger,
            "新しいサンドボックスコンテキストを作成: {}, 親: {}, 権限数: {}",
            context_id,
            parent_context.context_id,
            sandbox_permissions.len(),
        );
        
        // コンテキストを作成
        let context = Arc::new(
            SecurityContext::new(
                credentials,
                sandbox_permissions,
                context_id.clone(),
            )
            .with_parent(parent_context.context_id.clone())
            .with_sandbox(sandbox_path)
        );
        
        // アクティブコンテキストに追加
        let mut contexts = self.active_contexts.write().map_err(|_| {
            "アクティブコンテキストへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        contexts.insert(context_id.clone(), context.clone());
        
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::system::process::ProcessId;
    
    #[test]
    fn test_security_level() {
        assert_eq!(SecurityLevel::Root.to_string(), "root");
        assert_eq!(SecurityLevel::from_string("admin"), Some(SecurityLevel::Admin));
        assert_eq!(SecurityLevel::from_string("invalid"), None);
        
        assert!(SecurityLevel::Root.is_at_least(SecurityLevel::Admin));
        assert!(SecurityLevel::Admin.is_at_least(SecurityLevel::Admin));
        assert!(!SecurityLevel::Normal.is_at_least(SecurityLevel::Admin));
    }
    
    #[test]
    fn test_credentials() {
        let creds = Credentials::new(
            "user1".to_string(),
            ProcessId::new(1234),
            "app1".to_string(),
            SecurityLevel::Normal,
            "session1".to_string(),
        );
        
        assert_eq!(creds.user_id, "user1");
        assert_eq!(creds.app_id, "app1");
        assert_eq!(creds.security_level, SecurityLevel::Normal);
        assert!(creds.is_valid());
        
        // 有効期限付きの認証情報
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        let expired_creds = Credentials::new(
            "user1".to_string(),
            ProcessId::new(1234),
            "app1".to_string(),
            SecurityLevel::Normal,
            "session1".to_string(),
        ).with_expiration(now - 1000); // 過去の時間
        
        assert!(!expired_creds.is_valid());
    }
    
    #[test]
    fn test_security_context() {
        let creds = Credentials::new(
            "user1".to_string(),
            ProcessId::new(1234),
            "app1".to_string(),
            SecurityLevel::Normal,
            "session1".to_string(),
        );
        
        let mut permissions = PermissionSet::new();
        permissions.add(&Permission::FileRead);
        permissions.add(&Permission::FileWrite);
        
        let context = SecurityContext::new(
            creds,
            permissions,
            "ctx1".to_string(),
        );
        
        assert_eq!(context.context_id, "ctx1");
        assert!(context.has_permission(&Permission::FileRead));
        assert!(!context.has_permission(&Permission::NetworkConnect));
        assert_eq!(context.security_level(), SecurityLevel::Normal);
        assert_eq!(context.user_id(), "user1");
        assert_eq!(context.app_id(), "app1");
    }
    
    #[test]
    fn test_security_manager() {
        let manager = SecurityManager::new();
        
        // テスト用の権限セットを作成
        let mut permissions = PermissionSet::new();
        permissions.add(&Permission::FileRead);
        permissions.add(&Permission::FileWrite);
        
        // 認証情報を作成
        let creds = Credentials::new(
            "user1".to_string(),
            ProcessId::new(1234),
            "app1".to_string(),
            SecurityLevel::Admin,
            "session1".to_string(),
        );
        
        // コンテキストを作成
        let context_result = manager.create_context(creds, Some(permissions));
        assert!(context_result.is_ok());
        
        let context = context_result.unwrap();
        
        // コンテキストを取得
        let retrieved = manager.get_context(&context.context_id);
        assert!(retrieved.is_some());
        
        // コンテキストを削除
        let remove_result = manager.remove_context(&context.context_id);
        assert!(remove_result.is_ok());
        
        // 削除後に取得を試みる
        let not_found = manager.get_context(&context.context_id);
        assert!(not_found.is_none());
    }
} 