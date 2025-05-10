// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
//
// 権限モジュール
// Copyright (c) 2023-2024 AetherOS Team.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, RwLock};
use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use crate::system::security::{
    SecurityError,
    SecurityResult,
    security_context::SecurityLevel,
};

/// 権限
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// 権限名
    name: String,
    
    /// 説明
    description: Option<String>,
    
    /// 最小権限レベル
    min_level: Option<SecurityLevel>,
}

impl Permission {
    /// 新しい権限を作成
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            min_level: None,
        }
    }
    
    /// 説明付きの新しい権限を作成
    pub fn with_description(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: Some(description.to_string()),
            min_level: None,
        }
    }
    
    /// 最小権限レベル付きの新しい権限を作成
    pub fn with_level(name: &str, min_level: SecurityLevel) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            min_level: Some(min_level),
        }
    }
    
    /// 説明と最小権限レベル付きの新しい権限を作成
    pub fn with_description_and_level(name: &str, description: &str, min_level: SecurityLevel) -> Self {
        Self {
            name: name.to_string(),
            description: Some(description.to_string()),
            min_level: Some(min_level),
        }
    }
    
    /// 権限名を取得
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 説明を取得
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    /// 最小権限レベルを取得
    pub fn min_level(&self) -> Option<SecurityLevel> {
        self.min_level
    }
    
    /// 指定されたセキュリティレベルがこの権限に必要な最小レベル以上かどうかを確認
    pub fn check_level(&self, level: SecurityLevel) -> bool {
        if let Some(min_level) = self.min_level {
            level >= min_level
        } else {
            true
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<&str> for Permission {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

/// ワイルドカード権限マッチングの実装
/// 例: "system.*" は "system.read", "system.write" などにマッチする
fn permission_matches(pattern: &str, permission: &str) -> bool {
    if pattern == permission {
        return true;
    }
    
    if pattern.ends_with(".*") {
        let prefix = &pattern[..pattern.len() - 2];
        return permission.starts_with(prefix) && permission[prefix.len()..].starts_with('.');
    }
    
    false
}

/// 権限マネージャ
pub struct PermissionManager {
    /// 登録された権限
    permissions: HashMap<String, Permission>,
    
    /// 権限グループ
    groups: HashMap<String, HashSet<String>>,
    
    /// 初期化状態
    initialized: bool,
}

impl PermissionManager {
    /// 新しい権限マネージャを作成
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            groups: HashMap::new(),
            initialized: false,
        }
    }
    
    /// 権限マネージャを初期化
    pub fn initialize(&mut self) -> SecurityResult<()> {
        if self.initialized {
            return Err(SecurityError::InitializationError("すでに初期化されています".to_string()));
        }
        
        info!("権限マネージャを初期化中...");
        
        // デフォルト権限を登録
        self.register_default_permissions()?;
        
        self.initialized = true;
        info!("権限マネージャの初期化が完了しました");
        
        Ok(())
    }
    
    /// デフォルト権限を登録
    fn register_default_permissions(&mut self) -> SecurityResult<()> {
        // システム関連
        self.register_permission(Permission::with_description_and_level(
            "system.read",
            "システム情報の読み取り",
            SecurityLevel::Standard,
        ))?;
        
        self.register_permission(Permission::with_description_and_level(
            "system.write",
            "システム情報の書き込み",
            SecurityLevel::Elevated,
        ))?;
        
        self.register_permission(Permission::with_description_and_level(
            "system.admin",
            "システム管理",
            SecurityLevel::System,
        ))?;
        
        // アプリケーション関連
        self.register_permission(Permission::with_description_and_level(
            "app.install",
            "アプリケーションのインストール",
            SecurityLevel::Elevated,
        ))?;
        
        self.register_permission(Permission::with_description_and_level(
            "app.uninstall",
            "アプリケーションのアンインストール",
            SecurityLevel::Elevated,
        ))?;
        
        self.register_permission(Permission::with_description_and_level(
            "app.run",
            "アプリケーションの実行",
            SecurityLevel::Standard,
        ))?;
        
        // ファイル関連
        self.register_permission(Permission::with_description_and_level(
            "file.read",
            "ファイルの読み取り",
            SecurityLevel::Standard,
        ))?;
        
        self.register_permission(Permission::with_description_and_level(
            "file.write",
            "ファイルの書き込み",
            SecurityLevel::Standard,
        ))?;
        
        self.register_permission(Permission::with_description_and_level(
            "file.delete",
            "ファイルの削除",
            SecurityLevel::Standard,
        ))?;
        
        // ネットワーク関連
        self.register_permission(Permission::with_description_and_level(
            "network.connect",
            "ネットワーク接続",
            SecurityLevel::Standard,
        ))?;
        
        self.register_permission(Permission::with_description_and_level(
            "network.listen",
            "ネットワークリスニング",
            SecurityLevel::Elevated,
        ))?;
        
        // デバイス関連
        self.register_permission(Permission::with_description_and_level(
            "device.access",
            "デバイスアクセス",
            SecurityLevel::Standard,
        ))?;
        
        self.register_permission(Permission::with_description_and_level(
            "device.configure",
            "デバイス設定",
            SecurityLevel::Elevated,
        ))?;
        
        // ハードウェア関連
        self.register_permission(Permission::with_description_and_level(
            "hardware.access",
            "ハードウェアアクセス",
            SecurityLevel::Elevated,
        ))?;
        
        self.register_permission(Permission::with_description_and_level(
            "hardware.configure",
            "ハードウェア設定",
            SecurityLevel::System,
        ))?;
        
        // グループの作成
        let mut file_permissions = HashSet::new();
        file_permissions.insert("file.read".to_string());
        file_permissions.insert("file.write".to_string());
        file_permissions.insert("file.delete".to_string());
        
        self.register_permission_group("file.*", file_permissions)?;
        
        let mut network_permissions = HashSet::new();
        network_permissions.insert("network.connect".to_string());
        network_permissions.insert("network.listen".to_string());
        
        self.register_permission_group("network.*", network_permissions)?;
        
        let mut device_permissions = HashSet::new();
        device_permissions.insert("device.access".to_string());
        device_permissions.insert("device.configure".to_string());
        
        self.register_permission_group("device.*", device_permissions)?;
        
        let mut hardware_permissions = HashSet::new();
        hardware_permissions.insert("hardware.access".to_string());
        hardware_permissions.insert("hardware.configure".to_string());
        
        self.register_permission_group("hardware.*", hardware_permissions)?;
        
        let mut system_permissions = HashSet::new();
        system_permissions.insert("system.read".to_string());
        system_permissions.insert("system.write".to_string());
        system_permissions.insert("system.admin".to_string());
        
        self.register_permission_group("system.*", system_permissions)?;
        
        Ok(())
    }
    
    /// 権限を登録
    pub fn register_permission(&mut self, permission: Permission) -> SecurityResult<()> {
        if !self.initialized && permission.name() != "system.read" && permission.name() != "system.write" && permission.name() != "system.admin" {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        self.permissions.insert(permission.name().to_string(), permission);
        Ok(())
    }
    
    /// 権限を取得
    pub fn get_permission(&self, name: &str) -> Option<&Permission> {
        self.permissions.get(name)
    }
    
    /// 権限グループを登録
    pub fn register_permission_group(&mut self, group_name: &str, permissions: HashSet<String>) -> SecurityResult<()> {
        if !self.initialized && !group_name.starts_with("system.") && !group_name.starts_with("file.") && !group_name.starts_with("network.") && !group_name.starts_with("device.") && !group_name.starts_with("hardware.") {
            return Err(SecurityError::InitializationError("初期化されていません".to_string()));
        }
        
        self.groups.insert(group_name.to_string(), permissions);
        Ok(())
    }
    
    /// 権限をチェック
    pub fn check_permission(&self, required: &str, granted: &HashSet<Permission>) -> bool {
        // 直接一致する権限があるかチェック
        if granted.iter().any(|p| p.name() == required) {
            return true;
        }
        
        // ワイルドカード権限をチェック
        for permission in granted.iter() {
            if permission_matches(permission.name(), required) {
                return true;
            }
        }
        
        // 権限グループをチェック
        for (group_name, permissions) in &self.groups {
            if granted.iter().any(|p| p.name() == group_name) && permissions.contains(required) {
                return true;
            }
        }
        
        false
    }
    
    /// 権限セットをチェック
    pub fn check_permissions(&self, required: &HashSet<&str>, granted: &HashSet<Permission>) -> bool {
        for permission in required {
            if !self.check_permission(permission, granted) {
                return false;
            }
        }
        
        true
    }
    
    /// 全ての権限を取得
    pub fn get_all_permissions(&self) -> Vec<&Permission> {
        self.permissions.values().collect()
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

// シングルトンインスタンス
lazy_static::lazy_static! {
    static ref PERMISSION_MANAGER: RwLock<PermissionManager> = RwLock::new(PermissionManager::new());
}

/// グローバル権限マネージャを初期化
pub fn initialize_permission_manager() -> SecurityResult<()> {
    let mut manager = PERMISSION_MANAGER.write().map_err(|_| {
        SecurityError::InternalError("権限マネージャのロックに失敗しました".to_string())
    })?;
    
    manager.initialize()
}

/// 権限を登録
pub fn register_permission(permission: Permission) -> SecurityResult<()> {
    let mut manager = PERMISSION_MANAGER.write().map_err(|_| {
        SecurityError::InternalError("権限マネージャのロックに失敗しました".to_string())
    })?;
    
    manager.register_permission(permission)
}

/// 権限グループを登録
pub fn register_permission_group(group_name: &str, permissions: HashSet<String>) -> SecurityResult<()> {
    let mut manager = PERMISSION_MANAGER.write().map_err(|_| {
        SecurityError::InternalError("権限マネージャのロックに失敗しました".to_string())
    })?;
    
    manager.register_permission_group(group_name, permissions)
}

/// 権限をチェック
pub fn check_permission(required: &str, granted: &HashSet<Permission>) -> SecurityResult<bool> {
    let manager = PERMISSION_MANAGER.read().map_err(|_| {
        SecurityError::InternalError("権限マネージャのロックに失敗しました".to_string())
    })?;
    
    Ok(manager.check_permission(required, granted))
}

/// 権限セットをチェック
pub fn check_permissions(required: &HashSet<&str>, granted: &HashSet<Permission>) -> SecurityResult<bool> {
    let manager = PERMISSION_MANAGER.read().map_err(|_| {
        SecurityError::InternalError("権限マネージャのロックに失敗しました".to_string())
    })?;
    
    Ok(manager.check_permissions(required, granted))
}

/// 全ての権限を取得
pub fn get_all_permissions() -> SecurityResult<Vec<Permission>> {
    let manager = PERMISSION_MANAGER.read().map_err(|_| {
        SecurityError::InternalError("権限マネージャのロックに失敗しました".to_string())
    })?;
    
    let permissions = manager.get_all_permissions().into_iter().cloned().collect();
    Ok(permissions)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission() {
        let permission = Permission::new("test.permission");
        assert_eq!(permission.name(), "test.permission");
        assert_eq!(permission.description(), None);
        assert_eq!(permission.min_level(), None);
        
        let permission = Permission::with_description("test.permission", "テスト権限");
        assert_eq!(permission.name(), "test.permission");
        assert_eq!(permission.description(), Some("テスト権限"));
        assert_eq!(permission.min_level(), None);
        
        let permission = Permission::with_level("test.permission", SecurityLevel::Elevated);
        assert_eq!(permission.name(), "test.permission");
        assert_eq!(permission.description(), None);
        assert_eq!(permission.min_level(), Some(SecurityLevel::Elevated));
        
        let permission = Permission::with_description_and_level("test.permission", "テスト権限", SecurityLevel::Elevated);
        assert_eq!(permission.name(), "test.permission");
        assert_eq!(permission.description(), Some("テスト権限"));
        assert_eq!(permission.min_level(), Some(SecurityLevel::Elevated));
        
        // レベルチェック
        assert!(permission.check_level(SecurityLevel::Elevated));
        assert!(permission.check_level(SecurityLevel::System));
        assert!(!permission.check_level(SecurityLevel::Standard));
        assert!(!permission.check_level(SecurityLevel::Untrusted));
    }
    
    #[test]
    fn test_permission_matches() {
        assert!(permission_matches("test.permission", "test.permission"));
        assert!(permission_matches("test.*", "test.permission"));
        assert!(permission_matches("test.*", "test.other"));
        assert!(!permission_matches("test.*", "other.permission"));
        assert!(!permission_matches("test.permission", "test.other"));
    }
    
    #[test]
    fn test_permission_manager() {
        let mut manager = PermissionManager::new();
        
        // 初期化
        let result = manager.initialize();
        assert!(result.is_ok());
        
        // 権限登録
        let permission = Permission::new("test.permission");
        let result = manager.register_permission(permission.clone());
        assert!(result.is_ok());
        
        // 権限取得
        let retrieved = manager.get_permission("test.permission");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test.permission");
        
        // 権限チェック
        let mut granted = HashSet::new();
        granted.insert(Permission::new("test.permission"));
        
        assert!(manager.check_permission("test.permission", &granted));
        assert!(!manager.check_permission("other.permission", &granted));
        
        // ワイルドカード権限
        let mut granted = HashSet::new();
        granted.insert(Permission::new("test.*"));
        
        assert!(manager.check_permission("test.permission", &granted));
        assert!(manager.check_permission("test.other", &granted));
        assert!(!manager.check_permission("other.permission", &granted));
        
        // 権限グループ
        let mut permissions = HashSet::new();
        permissions.insert("test.read".to_string());
        permissions.insert("test.write".to_string());
        
        let result = manager.register_permission_group("test.*", permissions);
        assert!(result.is_ok());
        
        let mut granted = HashSet::new();
        granted.insert(Permission::new("test.*"));
        
        assert!(manager.check_permission("test.read", &granted));
        assert!(manager.check_permission("test.write", &granted));
        assert!(!manager.check_permission("test.delete", &granted));
    }
    
    #[test]
    fn test_global_permission_manager() {
        // 初期化
        let result = initialize_permission_manager();
        assert!(result.is_ok());
        
        // 権限登録
        let permission = Permission::new("test.permission");
        let result = register_permission(permission.clone());
        assert!(result.is_ok());
        
        // 権限チェック
        let mut granted = HashSet::new();
        granted.insert(Permission::new("test.permission"));
        
        let result = check_permission("test.permission", &granted);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        let result = check_permission("other.permission", &granted);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
} 