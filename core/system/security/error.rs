// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
//
// セキュリティ関連のエラー定義
// Copyright (c) 2023-2024 AetherOS Team.

use std::fmt;
use std::error::Error;
use thiserror::Error;

/// セキュリティシステムの結果型
pub type SecurityResult<T> = Result<T, SecurityError>;

/// セキュリティエラー
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SecurityError {
    /// 初期化エラー
    #[error("初期化エラー: {0}")]
    InitializationError(String),
    
    /// 認証エラー
    #[error("認証エラー: {0}")]
    AuthenticationError(String),
    
    /// 権限エラー
    #[error("権限エラー: {0}")]
    PermissionError(String),
    
    /// 無効なトークン
    #[error("無効なトークン: {0}")]
    InvalidTokenError(String),
    
    /// 期限切れのトークン
    #[error("期限切れのトークン: {0}")]
    ExpiredTokenError(String),
    
    /// 暗号化エラー
    #[error("暗号化エラー: {0}")]
    CryptographyError(String),
    
    /// 入力検証エラー
    #[error("入力検証エラー: {0}")]
    ValidationError(String),
    
    /// 構成エラー
    #[error("構成エラー: {0}")]
    ConfigurationError(String),
    
    /// 内部エラー
    #[error("内部エラー: {0}")]
    InternalError(String),
    
    /// I/Oエラー
    #[error("I/Oエラー: {0}")]
    IoError(String),
    
    /// その他のエラー
    #[error("その他のエラー: {0}")]
    OtherError(String),
}

impl SecurityError {
    /// エラーの簡潔な説明を取得する
    pub fn brief_description(&self) -> String {
        match self {
            SecurityError::InitializationError(_) => "初期化エラー".to_string(),
            SecurityError::AuthenticationError(_) => "認証エラー".to_string(),
            SecurityError::PermissionError(_) => "権限エラー".to_string(),
            SecurityError::InvalidTokenError(_) => "無効なトークン".to_string(),
            SecurityError::ExpiredTokenError(_) => "期限切れのトークン".to_string(),
            SecurityError::CryptographyError(_) => "暗号化エラー".to_string(),
            SecurityError::ValidationError(_) => "入力検証エラー".to_string(),
            SecurityError::ConfigurationError(_) => "構成エラー".to_string(),
            SecurityError::InternalError(_) => "内部エラー".to_string(),
            SecurityError::IoError(_) => "I/Oエラー".to_string(),
            SecurityError::OtherError(_) => "その他のエラー".to_string(),
        }
    }
    
    /// エラーの詳細説明を取得する
    pub fn detailed_description(&self) -> String {
        match self {
            SecurityError::InitializationError(msg) => format!("初期化エラー: {}", msg),
            SecurityError::AuthenticationError(msg) => format!("認証エラー: {}", msg),
            SecurityError::PermissionError(msg) => format!("権限エラー: {}", msg),
            SecurityError::InvalidTokenError(msg) => format!("無効なトークン: {}", msg),
            SecurityError::ExpiredTokenError(msg) => format!("期限切れのトークン: {}", msg),
            SecurityError::CryptographyError(msg) => format!("暗号化エラー: {}", msg),
            SecurityError::ValidationError(msg) => format!("入力検証エラー: {}", msg),
            SecurityError::ConfigurationError(msg) => format!("構成エラー: {}", msg),
            SecurityError::InternalError(msg) => format!("内部エラー: {}", msg),
            SecurityError::IoError(msg) => format!("I/Oエラー: {}", msg),
            SecurityError::OtherError(msg) => format!("その他のエラー: {}", msg),
        }
    }
    
    /// エラーのログレベルを取得する
    pub fn log_level(&self) -> log::Level {
        match self {
            SecurityError::InitializationError(_) => log::Level::Error,
            SecurityError::AuthenticationError(_) => log::Level::Warn,
            SecurityError::PermissionError(_) => log::Level::Warn,
            SecurityError::InvalidTokenError(_) => log::Level::Warn,
            SecurityError::ExpiredTokenError(_) => log::Level::Info,
            SecurityError::CryptographyError(_) => log::Level::Error,
            SecurityError::ValidationError(_) => log::Level::Warn,
            SecurityError::ConfigurationError(_) => log::Level::Error,
            SecurityError::InternalError(_) => log::Level::Error,
            SecurityError::IoError(_) => log::Level::Error,
            SecurityError::OtherError(_) => log::Level::Warn,
        }
    }
    
    /// エラーコードを取得する
    pub fn error_code(&self) -> u32 {
        match self {
            SecurityError::InitializationError(_) => 1001,
            SecurityError::AuthenticationError(_) => 1002,
            SecurityError::PermissionError(_) => 1003,
            SecurityError::InvalidTokenError(_) => 1004,
            SecurityError::ExpiredTokenError(_) => 1005,
            SecurityError::CryptographyError(_) => 1006,
            SecurityError::ValidationError(_) => 1007,
            SecurityError::ConfigurationError(_) => 1008,
            SecurityError::InternalError(_) => 1009,
            SecurityError::IoError(_) => 1010,
            SecurityError::OtherError(_) => 1099,
        }
    }
}

impl From<std::io::Error> for SecurityError {
    fn from(error: std::io::Error) -> Self {
        SecurityError::IoError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_error() {
        let error = SecurityError::AuthenticationError("認証に失敗しました".to_string());
        
        assert_eq!(error.brief_description(), "認証エラー");
        assert_eq!(error.detailed_description(), "認証エラー: 認証に失敗しました");
        assert_eq!(error.log_level(), log::Level::Warn);
        assert_eq!(error.error_code(), 1002);
        
        let error = SecurityError::PermissionError("権限がありません".to_string());
        
        assert_eq!(error.brief_description(), "権限エラー");
        assert_eq!(error.detailed_description(), "権限エラー: 権限がありません");
        assert_eq!(error.log_level(), log::Level::Warn);
        assert_eq!(error.error_code(), 1003);
        
        let error = SecurityError::from(std::io::Error::new(std::io::ErrorKind::NotFound, "ファイルが見つかりません"));
        
        assert_eq!(error.brief_description(), "I/Oエラー");
        assert_eq!(error.log_level(), log::Level::Error);
        assert_eq!(error.error_code(), 1010);
    }
} 