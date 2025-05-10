// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
//
// ハプティクス共通定義
// Copyright (c) 2023-2024 AetherOS Team.

use std::collections::HashMap;
use std::fmt;
use std::time::Instant;
use serde::{Serialize, Deserialize};

// 強度の最小値と最大値
pub const MIN_INTENSITY: u8 = 0;
pub const MAX_INTENSITY: u8 = 100;

/// ハプティックデバイスの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HapticDeviceType {
    /// 内蔵振動モーター
    InternalMotor,
    
    /// ハプティック対応タッチパッド
    Touchpad,
    
    /// ハプティック対応タッチスクリーン
    Touchscreen,
    
    /// ハプティック対応ゲームコントローラー
    GameController,
    
    /// その他のハプティックデバイス
    Other,
}

/// ハプティックフィードバックパターン
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HapticPattern {
    /// パターンなし
    None,
    
    /// クリック感
    Click,
    
    /// ダブルクリック感
    DoubleClick,
    
    /// トリプルクリック感
    TripleClick,
    
    /// 軽いタップ感
    Tap,
    
    /// 短いパルス
    Pulse,
    
    /// ブザー音のような振動
    Buzz,
    
    /// 通知
    Notification,
    
    /// 成功フィードバック
    Success,
    
    /// 警告フィードバック
    Warning,
    
    /// エラーフィードバック
    Error,
    
    /// 一般的な振動
    Vibration,
    
    /// 長押し感
    LongPress,
    
    /// 連続的な振動（ゲームコントローラーなど）
    RumbleContinuous,
    
    /// パルス状の振動（ゲームコントローラーなど）
    RumblePulse,
    
    /// カスタムパターン
    Custom(String),
}

/// ハプティックデバイスが対応する効果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticEffect {
    /// クリック感
    Click,
    
    /// ダブルクリック感
    DoubleClick,
    
    /// トリプルクリック感
    TripleClick,
    
    /// 軽いタップ感
    Tap,
    
    /// 短いパルス
    Pulse,
    
    /// ブザー音のような振動
    Buzz,
    
    /// 通知
    Notification,
    
    /// 成功フィードバック
    Success,
    
    /// 警告フィードバック
    Warning,
    
    /// エラーフィードバック
    Error,
    
    /// 一般的な振動
    Vibration,
    
    /// 長押し感
    LongPress,
    
    /// 連続的な振動（ゲームコントローラーなど）
    RumbleContinuous,
    
    /// パルス状の振動（ゲームコントローラーなど）
    RumblePulse,
}

impl fmt::Display for HapticEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HapticEffect::Click => write!(f, "Click"),
            HapticEffect::DoubleClick => write!(f, "DoubleClick"),
            HapticEffect::TripleClick => write!(f, "TripleClick"),
            HapticEffect::Tap => write!(f, "Tap"),
            HapticEffect::Pulse => write!(f, "Pulse"),
            HapticEffect::Buzz => write!(f, "Buzz"),
            HapticEffect::Notification => write!(f, "Notification"),
            HapticEffect::Success => write!(f, "Success"),
            HapticEffect::Warning => write!(f, "Warning"),
            HapticEffect::Error => write!(f, "Error"),
            HapticEffect::Vibration => write!(f, "Vibration"),
            HapticEffect::LongPress => write!(f, "LongPress"),
            HapticEffect::RumbleContinuous => write!(f, "RumbleContinuous"),
            HapticEffect::RumblePulse => write!(f, "RumblePulse"),
        }
    }
}

/// ハプティックフィードバックの強度
pub type HapticIntensity = u8;

/// ハプティックデバイス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticDevice {
    /// デバイスID
    pub id: String,
    
    /// デバイス名
    pub name: String,
    
    /// デバイスタイプ
    pub device_type: HapticDeviceType,
    
    /// 最大強度（0-100）
    pub max_intensity: u8,
    
    /// サポートされている効果
    pub supported_effects: Vec<HapticEffect>,
    
    /// 利用可能かどうか
    pub available: bool,
}

/// ハプティックイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticEvent {
    /// フィードバックパターン
    pub pattern: HapticPattern,
    
    /// 強度（オプション、指定しない場合はデフォルト値が使用される）
    pub intensity: Option<HapticIntensity>,
    
    /// 持続時間（ミリ秒、オプション、指定しない場合はパターンに応じたデフォルト値が使用される）
    pub duration_ms: Option<u32>,
    
    /// ターゲットデバイスID（オプション、指定しない場合はデフォルトデバイスが使用される）
    pub target_device_id: Option<String>,
    
    /// イベントカテゴリ（"ui", "system", "app", "accessibility" など）
    pub category: Option<String>,
}

/// ハプティックイベント結果
#[derive(Debug, Clone)]
pub struct HapticEventResult {
    /// 成功したかどうか
    pub success: bool,
    
    /// 使用されたデバイスID
    pub device_id: String,
}

/// ハプティックデバイスの状態
#[derive(Debug, Clone)]
pub struct HapticState {
    /// アクティブかどうか
    pub active: bool,
    
    /// 最後の効果
    pub last_effect: Option<HapticEffect>,
    
    /// 最後の強度
    pub last_intensity: u8,
    
    /// 開始時間
    pub start_time: Instant,
}

/// ハプティックエラー
#[derive(Debug, Clone)]
pub enum HapticError {
    /// システムが初期化されていない
    SystemNotInitialized,
    
    /// デバイスが見つからない
    DeviceNotFound,
    
    /// 無効なパターン
    InvalidPattern,
    
    /// デバイスが利用できない
    DeviceNotAvailable,
    
    /// その他のエラー
    Other(String),
}

impl fmt::Display for HapticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HapticError::SystemNotInitialized => write!(f, "ハプティックシステムが初期化されていません"),
            HapticError::DeviceNotFound => write!(f, "ハプティックデバイスが見つかりません"),
            HapticError::InvalidPattern => write!(f, "無効なハプティックパターンです"),
            HapticError::DeviceNotAvailable => write!(f, "ハプティックデバイスが利用できません"),
            HapticError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for HapticError {}

impl HapticEvent {
    /// 新しいハプティックイベントを作成
    pub fn new(pattern: HapticPattern) -> Self {
        Self {
            pattern,
            intensity: Some(50), // デフォルトは中程度
            duration_ms: None,
            target_device_id: None,
            category: None,
        }
    }
    
    /// 強度を設定
    pub fn with_intensity(mut self, intensity: HapticIntensity) -> Self {
        self.intensity = Some(intensity);
        self
    }
    
    /// 持続時間を設定
    pub fn with_duration(mut self, duration_ms: u32) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
    
    /// ターゲットデバイスを設定
    pub fn with_device(mut self, device_id: String) -> Self {
        self.target_device_id = Some(device_id);
        self
    }
    
    /// カテゴリを設定
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }
}

/// UI要素のためのハプティックイベントを作成
pub fn create_ui_haptic_event(pattern: HapticPattern) -> HapticEvent {
    HapticEvent::new(pattern)
        .with_category("ui".to_string())
}

/// システムのためのハプティックイベントを作成
pub fn create_system_haptic_event(pattern: HapticPattern) -> HapticEvent {
    HapticEvent::new(pattern)
        .with_category("system".to_string())
}

/// アプリケーションのためのハプティックイベントを作成
pub fn create_app_haptic_event(pattern: HapticPattern) -> HapticEvent {
    HapticEvent::new(pattern)
        .with_category("app".to_string())
}

/// アクセシビリティのためのハプティックイベントを作成
pub fn create_accessibility_haptic_event(pattern: HapticPattern) -> HapticEvent {
    HapticEvent::new(pattern)
        .with_category("accessibility".to_string())
}

/// 指定されたデバイスタイプに適した効果に変換
pub fn convert_to_device_effect(pattern: HapticPattern, device_type: HapticDeviceType) -> Option<HapticEffect> {
    match device_type {
        HapticDeviceType::InternalMotor => {
            match pattern {
                HapticPattern::Click | HapticPattern::Tap => Some(HapticEffect::Click),
                HapticPattern::DoubleClick => Some(HapticEffect::DoubleClick),
                HapticPattern::TripleClick => Some(HapticEffect::TripleClick),
                HapticPattern::Pulse => Some(HapticEffect::Pulse),
                HapticPattern::Buzz => Some(HapticEffect::Buzz),
                HapticPattern::Notification => Some(HapticEffect::Notification),
                HapticPattern::Success => Some(HapticEffect::Success),
                HapticPattern::Warning => Some(HapticEffect::Warning),
                HapticPattern::Error => Some(HapticEffect::Error),
                HapticPattern::Vibration => Some(HapticEffect::Vibration),
                HapticPattern::LongPress => Some(HapticEffect::LongPress),
                _ => None,
            }
        },
        HapticDeviceType::Touchpad => {
            match pattern {
                HapticPattern::Click | HapticPattern::Tap => Some(HapticEffect::Click),
                HapticPattern::DoubleClick => Some(HapticEffect::DoubleClick),
                HapticPattern::TripleClick => Some(HapticEffect::TripleClick),
                HapticPattern::Pulse => Some(HapticEffect::Pulse),
                _ => None,
            }
        },
        HapticDeviceType::Touchscreen => {
            match pattern {
                HapticPattern::Click | HapticPattern::Tap => Some(HapticEffect::Tap),
                _ => None,
            }
        },
        HapticDeviceType::GameController => {
            match pattern {
                HapticPattern::Vibration => Some(HapticEffect::Vibration),
                HapticPattern::RumbleContinuous => Some(HapticEffect::RumbleContinuous),
                HapticPattern::RumblePulse => Some(HapticEffect::RumblePulse),
                _ => None,
            }
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_haptic_event_creation() {
        let event = HapticEvent::new(HapticPattern::Click);
        assert_eq!(event.pattern, HapticPattern::Click);
        assert_eq!(event.intensity, Some(50));
        assert_eq!(event.duration_ms, None);
        assert_eq!(event.target_device_id, None);
        assert_eq!(event.category, None);
        
        let event = HapticEvent::new(HapticPattern::Error)
            .with_intensity(100)
            .with_duration(500)
            .with_device("test_device".to_string())
            .with_category("system".to_string());
            
        assert_eq!(event.pattern, HapticPattern::Error);
        assert_eq!(event.intensity, Some(100));
        assert_eq!(event.duration_ms, Some(500));
        assert_eq!(event.target_device_id, Some("test_device".to_string()));
        assert_eq!(event.category, Some("system".to_string()));
    }
    
    #[test]
    fn test_create_category_events() {
        let ui_event = create_ui_haptic_event(HapticPattern::Click);
        assert_eq!(ui_event.category, Some("ui".to_string()));
        
        let system_event = create_system_haptic_event(HapticPattern::Error);
        assert_eq!(system_event.category, Some("system".to_string()));
        
        let app_event = create_app_haptic_event(HapticPattern::Notification);
        assert_eq!(app_event.category, Some("app".to_string()));
        
        let accessibility_event = create_accessibility_haptic_event(HapticPattern::Buzz);
        assert_eq!(accessibility_event.category, Some("accessibility".to_string()));
    }
    
    #[test]
    fn test_convert_to_device_effect() {
        // 内蔵モーター
        assert_eq!(
            convert_to_device_effect(HapticPattern::Click, HapticDeviceType::InternalMotor),
            Some(HapticEffect::Click)
        );
        
        // タッチパッド
        assert_eq!(
            convert_to_device_effect(HapticPattern::Pulse, HapticDeviceType::Touchpad),
            Some(HapticEffect::Pulse)
        );
        
        // タッチスクリーン - 制限された効果
        assert_eq!(
            convert_to_device_effect(HapticPattern::Tap, HapticDeviceType::Touchscreen),
            Some(HapticEffect::Tap)
        );
        assert_eq!(
            convert_to_device_effect(HapticPattern::Buzz, HapticDeviceType::Touchscreen),
            None
        );
        
        // ゲームコントローラー
        assert_eq!(
            convert_to_device_effect(HapticPattern::RumbleContinuous, HapticDeviceType::GameController),
            Some(HapticEffect::RumbleContinuous)
        );
        assert_eq!(
            convert_to_device_effect(HapticPattern::Click, HapticDeviceType::GameController),
            None
        );
    }
    
    #[test]
    fn test_haptic_error_display() {
        assert_eq!(
            HapticError::SystemNotInitialized.to_string(),
            "ハプティックシステムが初期化されていません"
        );
        
        assert_eq!(
            HapticError::DeviceNotFound.to_string(),
            "ハプティックデバイスが見つかりません"
        );
        
        assert_eq!(
            HapticError::InvalidPattern.to_string(),
            "無効なハプティックパターンです"
        );
        
        assert_eq!(
            HapticError::DeviceNotAvailable.to_string(),
            "ハプティックデバイスが利用できません"
        );
        
        assert_eq!(
            HapticError::Other("テストエラー".to_string()).to_string(),
            "テストエラー"
        );
    }
} 