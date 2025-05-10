// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
//
// セキュリティシステム
// Copyright (c) 2023-2024 AetherOS Team.

pub mod security_context;
pub mod permissions;
pub mod sandbox;
pub mod audit;

pub use security_context::{
    SecurityContext, 
    SecurityLevel, 
    SecurityToken, 
    SecurityPolicy,
    initialize_security_context,
    get_security_context,
    create_token,
    verify_token,
    elevate_privileges,
    drop_privileges,
};

pub use permissions::{
    Permission,
    PermissionSet,
    PermissionRequest,
    PermissionStatus,
    request_permission,
    check_permission,
    grant_permission,
    revoke_permission,
};

pub use sandbox::{
    SandboxConfig,
    SandboxEnvironment,
    SandboxStatus,
    create_sandbox,
    destroy_sandbox,
    run_in_sandbox,
};

pub use audit::{
    AuditEvent,
    AuditLevel,
    AuditLog,
    log_security_event,
    get_audit_logs,
    clear_audit_logs,
};

use log::{debug, info, warn, error};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;

/// エラー型
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("無効なセキュリティコンテキスト: {0}")]
    InvalidContext(String),
    
    #[error("認証エラー: {0}")]
    AuthenticationError(String),
    
    #[error("権限エラー: {0}")]
    PermissionDenied(String),
    
    #[error("サンドボックスエラー: {0}")]
    SandboxError(String),
    
    #[error("初期化エラー: {0}")]
    InitializationError(String),
    
    #[error("監査エラー: {0}")]
    AuditError(String),
    
    #[error("内部エラー: {0}")]
    InternalError(String),
}

/// 結果型
pub type SecurityResult<T> = Result<T, SecurityError>;

/// セキュリティマネージャー
/// システム全体のセキュリティを管理する
pub struct SecurityManager {
    // グローバルセキュリティコンテキスト
    context: Arc<RwLock<security_context::SecurityContextImpl>>,
    
    // パーミッション管理
    permission_manager: Arc<RwLock<permissions::PermissionManager>>,
    
    // サンドボックス管理
    sandbox_manager: Arc<RwLock<sandbox::SandboxManager>>,
    
    // 監査ロギング
    audit_logger: Arc<Mutex<audit::AuditLogger>>,
    
    // 初期化状態
    initialized: bool,
}

impl SecurityManager {
    /// 新しいセキュリティマネージャーを作成
    pub fn new() -> Self {
        Self {
            context: Arc::new(RwLock::new(security_context::SecurityContextImpl::new())),
            permission_manager: Arc::new(RwLock::new(permissions::PermissionManager::new())),
            sandbox_manager: Arc::new(RwLock::new(sandbox::SandboxManager::new())),
            audit_logger: Arc::new(Mutex::new(audit::AuditLogger::new())),
            initialized: false,
        }
    }
    
    /// セキュリティマネージャーを初期化
    pub fn initialize(&mut self, config_path: Option<&str>) -> SecurityResult<()> {
        if self.initialized {
            return Err(SecurityError::InitializationError("すでに初期化されています".to_string()));
        }
        
        info!("セキュリティマネージャーを初期化中...");
        
        // コンテキストの初期化
        {
            let mut context = self.context.write().map_err(|_| {
                SecurityError::InternalError("コンテキストのロックに失敗しました".to_string())
            })?;
            
            context.initialize(config_path)?;
        }
        
        // パーミッションマネージャーの初期化
        {
            let mut permission_manager = self.permission_manager.write().map_err(|_| {
                SecurityError::InternalError("パーミッションマネージャーのロックに失敗しました".to_string())
            })?;
            
            permission_manager.initialize(config_path)?;
        }
        
        // サンドボックスマネージャーの初期化
        {
            let mut sandbox_manager = self.sandbox_manager.write().map_err(|_| {
                SecurityError::InternalError("サンドボックスマネージャーのロックに失敗しました".to_string())
            })?;
            
            sandbox_manager.initialize(config_path)?;
        }
        
        // 監査ロガーの初期化
        {
            let mut audit_logger = self.audit_logger.lock().map_err(|_| {
                SecurityError::InternalError("監査ロガーのロックに失敗しました".to_string())
            })?;
            
            audit_logger.initialize(config_path)?;
        }
        
        self.initialized = true;
        info!("セキュリティマネージャーの初期化が完了しました");
        
        // 初期化完了イベントをログに記録
        self.log_audit_event(
            audit::AuditEvent::new(
                "security_manager.initialized",
                audit::AuditLevel::Info,
                "セキュリティマネージャーが初期化されました"
            )
        )?;
        
        Ok(())
    }
    
    /// 監査イベントを記録
    pub fn log_audit_event(&self, event: audit::AuditEvent) -> SecurityResult<()> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        let mut audit_logger = self.audit_logger.lock().map_err(|_| {
            SecurityError::InternalError("監査ロガーのロックに失敗しました".to_string())
        })?;
        
        audit_logger.log_event(event)
    }
    
    /// シャットダウン
    pub fn shutdown(&mut self) -> SecurityResult<()> {
        if !self.initialized {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        info!("セキュリティマネージャーをシャットダウン中...");
        
        // 監査イベントを記録
        self.log_audit_event(
            audit::AuditEvent::new(
                "security_manager.shutdown",
                audit::AuditLevel::Info,
                "セキュリティマネージャーがシャットダウンされます"
            )
        )?;
        
        // サンドボックスマネージャーのシャットダウン
        {
            let mut sandbox_manager = self.sandbox_manager.write().map_err(|_| {
                SecurityError::InternalError("サンドボックスマネージャーのロックに失敗しました".to_string())
            })?;
            
            sandbox_manager.shutdown()?;
        }
        
        // 監査ロガーのシャットダウン
        {
            let mut audit_logger = self.audit_logger.lock().map_err(|_| {
                SecurityError::InternalError("監査ロガーのロックに失敗しました".to_string())
            })?;
            
            audit_logger.shutdown()?;
        }
        
        self.initialized = false;
        info!("セキュリティマネージャーのシャットダウンが完了しました");
        
        Ok(())
    }
    
    /// セキュリティコンテキストへの参照を取得
    pub fn get_context(&self) -> Arc<RwLock<security_context::SecurityContextImpl>> {
        self.context.clone()
    }
    
    /// パーミッションマネージャーへの参照を取得
    pub fn get_permission_manager(&self) -> Arc<RwLock<permissions::PermissionManager>> {
        self.permission_manager.clone()
    }
    
    /// サンドボックスマネージャーへの参照を取得
    pub fn get_sandbox_manager(&self) -> Arc<RwLock<sandbox::SandboxManager>> {
        self.sandbox_manager.clone()
    }
    
    /// 監査ロガーへの参照を取得
    pub fn get_audit_logger(&self) -> Arc<Mutex<audit::AuditLogger>> {
        self.audit_logger.clone()
    }
}

// シングルトンインスタンス
lazy_static::lazy_static! {
    static ref SECURITY_MANAGER: Mutex<SecurityManager> = Mutex::new(SecurityManager::new());
}

/// グローバルセキュリティマネージャーを初期化
pub fn initialize_security_manager(config_path: Option<&str>) -> SecurityResult<()> {
    let mut manager = SECURITY_MANAGER.lock().map_err(|_| {
        SecurityError::InternalError("セキュリティマネージャーのロックに失敗しました".to_string())
    })?;
    
    manager.initialize(config_path)
}

/// グローバルセキュリティマネージャーをシャットダウン
pub fn shutdown_security_manager() -> SecurityResult<()> {
    let mut manager = SECURITY_MANAGER.lock().map_err(|_| {
        SecurityError::InternalError("セキュリティマネージャーのロックに失敗しました".to_string())
    })?;
    
    manager.shutdown()
}

/// グローバルセキュリティマネージャーを取得
pub fn get_security_manager() -> SecurityResult<MutexGuard<'static, SecurityManager>> {
    SECURITY_MANAGER.lock().map_err(|_| {
        SecurityError::InternalError("セキュリティマネージャーのロックに失敗しました".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_manager_initialization() {
        let mut manager = SecurityManager::new();
        assert!(!manager.initialized);
        
        let result = manager.initialize(None);
        assert!(result.is_ok());
        assert!(manager.initialized);
        
        // 二重初期化のテスト
        let result = manager.initialize(None);
        assert!(result.is_err());
        
        let result = manager.shutdown();
        assert!(result.is_ok());
        assert!(!manager.initialized);
    }
    
    #[test]
    fn test_log_audit_event() {
        let mut manager = SecurityManager::new();
        
        // 初期化前はエラーになるはず
        let event = audit::AuditEvent::new(
            "test.event",
            audit::AuditLevel::Info,
            "テストイベント"
        );
        let result = manager.log_audit_event(event);
        assert!(result.is_err());
        
        // 初期化後は成功するはず
        manager.initialize(None).unwrap();
        
        let event = audit::AuditEvent::new(
            "test.event",
            audit::AuditLevel::Info,
            "テストイベント"
        );
        let result = manager.log_audit_event(event);
        assert!(result.is_ok());
        
        manager.shutdown().unwrap();
    }
} 