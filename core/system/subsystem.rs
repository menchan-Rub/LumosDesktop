// Copyright (c) 2023-2024 AetherOS Project
//
// システムコンポーネントのためのサブシステムステータス定義

/// サブシステムの動作状態を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsystemStatus {
    /// 初期化されていない状態
    Uninitialized,
    
    /// 初期化中の状態
    Initializing,
    
    /// 実行中の状態
    Running,
    
    /// 一時停止中の状態
    Paused,
    
    /// 停止中の状態
    Stopping,
    
    /// 完全に停止した状態
    Stopped,
    
    /// エラー状態
    Error,
}

impl Default for SubsystemStatus {
    fn default() -> Self {
        Self::Uninitialized
    }
}

impl std::fmt::Display for SubsystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized => write!(f, "未初期化"),
            Self::Initializing => write!(f, "初期化中"),
            Self::Running => write!(f, "実行中"),
            Self::Paused => write!(f, "一時停止中"),
            Self::Stopping => write!(f, "停止中"),
            Self::Stopped => write!(f, "停止済み"),
            Self::Error => write!(f, "エラー"),
        }
    }
}

/// サブシステム共通の動作を定義するトレイト
pub trait Subsystem {
    /// システムを初期化する
    async fn initialize(&mut self) -> Result<(), String>;
    
    /// システムをシャットダウンする
    async fn shutdown(&mut self) -> Result<(), String>;
    
    /// 現在のステータスを取得する
    fn status(&self) -> SubsystemStatus;
    
    /// システムを一時停止する（オプション）
    async fn pause(&mut self) -> Result<(), String> {
        Err("一時停止機能は実装されていません".to_string())
    }
    
    /// 一時停止したシステムを再開する（オプション）
    async fn resume(&mut self) -> Result<(), String> {
        Err("再開機能は実装されていません".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subsystem_status_display() {
        assert_eq!(format!("{}", SubsystemStatus::Uninitialized), "未初期化");
        assert_eq!(format!("{}", SubsystemStatus::Running), "実行中");
        assert_eq!(format!("{}", SubsystemStatus::Stopped), "停止済み");
    }
    
    #[test]
    fn test_subsystem_status_default() {
        let status = SubsystemStatus::default();
        assert_eq!(status, SubsystemStatus::Uninitialized);
    }
} 