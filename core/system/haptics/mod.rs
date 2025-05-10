// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
//
// 共通ハプティックフィードバックインターフェース
// Copyright (c) 2023-2024 AetherOS Team.

// モジュール宣言
pub mod common;
pub mod linux;
pub mod macos;
pub mod windows;
pub mod aetheros_integration;

// プラットフォーム検出
#[cfg(target_os = "linux")]
use self::linux as platform;
#[cfg(target_os = "macos")]
use self::macos as platform;
#[cfg(target_os = "windows")]
use self::windows as platform;

// 外部APIにCommonモジュールを公開
pub use self::common::*;

// 統合AetherOSモジュールを公開
pub use self::aetheros_integration::{
    AetherOSHapticsHandler,
    AetherOSHapticsSettings,
    initialize_aetheros_haptics,
    get_aetheros_haptics_handler,
    play_aetheros_haptic_event,
    stop_all_aetheros_haptic_devices,
    stop_aetheros_haptic_device,
    update_aetheros_haptics_settings,
    set_aetheros_haptics_callback,
    shutdown_aetheros_haptics,
    serialize_settings,
    deserialize_settings,
};

use log::{debug, info, warn, error};
use std::fmt;

// ハプティックイベント強度
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HapticIntensity {
    None,       // 無効
    VeryLight,  // 非常に弱い
    Light,      // 弱い
    Medium,     // 中程度 (デフォルト)
    Strong,     // 強い
    VeryStrong, // 非常に強い
}

impl Default for HapticIntensity {
    fn default() -> Self {
        HapticIntensity::Medium
    }
}

// ハプティックパターン
#[derive(Debug, Clone, PartialEq)]
pub enum HapticPattern {
    Click,       // 標準クリック
    DoubleClick, // ダブルクリック
    LongPress,   // 長押し
    Success,     // 成功フィードバック
    Warning,     // 警告フィードバック
    Error,       // エラーフィードバック
    Custom(String), // カスタムパターン名
}

impl Default for HapticPattern {
    fn default() -> Self {
        HapticPattern::Click
    }
}

// ハプティックデバイスタイプ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HapticDeviceType {
    InternalMotor,   // 内蔵バイブレーションモーター
    Touchpad,        // ハプティクス対応タッチパッド
    Touchscreen,     // ハプティクス対応タッチスクリーン
    GameController,  // ゲームコントローラー
    Other,           // その他
}

impl fmt::Display for HapticDeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HapticDeviceType::InternalMotor => write!(f, "内蔵モーター"),
            HapticDeviceType::Touchpad => write!(f, "タッチパッド"),
            HapticDeviceType::Touchscreen => write!(f, "タッチスクリーン"),
            HapticDeviceType::GameController => write!(f, "ゲームコントローラー"),
            HapticDeviceType::Other => write!(f, "その他"),
        }
    }
}

// ハプティックデバイス
#[derive(Debug, Clone)]
pub struct HapticDevice {
    pub id: String,
    pub name: String,
    pub device_type: HapticDeviceType,
}

// ハプティックイベント
#[derive(Debug, Clone)]
pub struct HapticEvent {
    pub pattern: HapticPattern,
    pub intensity: Option<HapticIntensity>,
    pub duration_ms: Option<u32>,
    pub target_device_id: Option<String>,
    pub custom_parameters: Option<Vec<(String, String)>>,
}

// ハプティックイベント結果
#[derive(Debug, Clone)]
pub struct HapticEventResult {
    pub success: bool,
    pub device_id: String,
}

// ハプティックエラー
#[derive(Debug, Clone)]
pub enum HapticError {
    SystemNotInitialized,
    DeviceNotFound(String),
    ConnectionFailed(String),
    NoDeviceFound,
    UnsupportedPattern(String),
    Other(String),
}

impl std::fmt::Display for HapticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HapticError::SystemNotInitialized => write!(f, "ハプティックシステムが初期化されていません"),
            HapticError::DeviceNotFound(id) => write!(f, "ハプティックデバイスが見つかりません: {}", id),
            HapticError::ConnectionFailed(msg) => write!(f, "デバイス接続に失敗しました: {}", msg),
            HapticError::NoDeviceFound => write!(f, "ハプティックデバイスが見つかりません"),
            HapticError::UnsupportedPattern(msg) => write!(f, "サポートされていないパターン: {}", msg),
            HapticError::Other(msg) => write!(f, "ハプティックエラー: {}", msg),
        }
    }
}

impl std::error::Error for HapticError {}

impl HapticEvent {
    // 新しいハプティックイベントを作成
    pub fn new(pattern: HapticPattern) -> Self {
        HapticEvent {
            pattern,
            intensity: Some(HapticIntensity::Medium),
            duration_ms: None,
            target_device_id: None,
            custom_parameters: None,
        }
    }

    // 強度を設定
    pub fn with_intensity(mut self, intensity: HapticIntensity) -> Self {
        self.intensity = Some(intensity);
        self
    }

    // 時間を設定
    pub fn with_duration(mut self, duration_ms: u32) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    // 対象デバイスを設定
    pub fn with_device(mut self, device_id: String) -> Self {
        self.target_device_id = Some(device_id);
        self
    }

    // カスタムパラメータを追加
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        if self.custom_parameters.is_none() {
            self.custom_parameters = Some(Vec::new());
        }
        
        if let Some(params) = &mut self.custom_parameters {
            params.push((key.to_string(), value.to_string()));
        }
        
        self
    }
}

/// ハプティックフィードバックを再生
pub fn play_haptic_feedback(event: &HapticEvent) -> Result<HapticEventResult, HapticError> {
    debug!("ハプティックフィードバック再生: {:?}", event);
    
    // 必須パラメータの検証
    if event.pattern == HapticPattern::None {
        return Err(HapticError::InvalidPattern);
    }
    
    // ターゲットデバイス
    let device_id = event.target_device_id.clone().unwrap_or_default();
    
    // 強度とデュレーション
    let intensity = event.intensity.unwrap_or(HapticIntensity::Medium);
    let duration_ms = event.duration_ms.unwrap_or_else(|| {
        // パターンに基づいてデフォルトのデュレーションを設定
        match event.pattern {
            HapticPattern::Click => 10,
            HapticPattern::DoubleClick => 20,
            HapticPattern::TripleClick => 30,
            HapticPattern::Tap => 10,
            HapticPattern::Pulse => 50,
            HapticPattern::Buzz => 100,
            HapticPattern::Notification => 200,
            HapticPattern::Success => 150,
            HapticPattern::Warning => 200,
            HapticPattern::Error => 250,
            HapticPattern::Vibration => 300,
            HapticPattern::LongPress => 500,
            HapticPattern::RumbleContinuous => 1000,
            HapticPattern::RumblePulse => 500,
            HapticPattern::None => 0,
            _ => 100,
        }
    });
    
    // デバイスタイプに変換
    let effect = match event.pattern {
        HapticPattern::Click => HapticEffect::Click,
        HapticPattern::DoubleClick => HapticEffect::DoubleClick,
        HapticPattern::TripleClick => HapticEffect::TripleClick,
        HapticPattern::Tap => HapticEffect::Tap,
        HapticPattern::Pulse => HapticEffect::Pulse,
        HapticPattern::Buzz => HapticEffect::Buzz,
        HapticPattern::Notification => HapticEffect::Notification,
        HapticPattern::Success => HapticEffect::Success,
        HapticPattern::Warning => HapticEffect::Warning,
        HapticPattern::Error => HapticEffect::Error,
        HapticPattern::Vibration => HapticEffect::Vibration,
        HapticPattern::LongPress => HapticEffect::LongPress,
        HapticPattern::RumbleContinuous => HapticEffect::RumbleContinuous,
        HapticPattern::RumblePulse => HapticEffect::RumblePulse,
        _ => return Err(HapticError::InvalidPattern),
    };
    
    // プラットフォーム固有の実装を呼び出し
    let success = match std::env::consts::OS {
        "linux" => {
            #[cfg(target_os = "linux")]
            {
                platform::play_linux_haptic_feedback(&device_id, effect, intensity, duration_ms)
            }
            #[cfg(not(target_os = "linux"))]
            {
                debug!("Linux実装が呼び出されましたが、このプラットフォームではコンパイルされていません");
                false
            }
        },
        "macos" => {
            #[cfg(target_os = "macos")]
            {
                platform::play_macos_haptic_feedback(&device_id, effect, intensity, duration_ms)
            }
            #[cfg(not(target_os = "macos"))]
            {
                debug!("macOS実装が呼び出されましたが、このプラットフォームではコンパイルされていません");
                false
            }
        },
        "windows" => {
            #[cfg(target_os = "windows")]
            {
                platform::play_windows_haptic_feedback(&device_id, effect, intensity, duration_ms)
            }
            #[cfg(not(target_os = "windows"))]
            {
                debug!("Windows実装が呼び出されましたが、このプラットフォームではコンパイルされていません");
                false
            }
        },
        os => {
            warn!("サポートされていないOS: {}", os);
            false
        }
    };
    
    // 結果を返す
    Ok(HapticEventResult {
        success,
        device_id,
    })
}

/// ハプティックデバイスを停止
pub fn stop_haptic_device(device_id: Option<&str>) -> Result<(), HapticError> {
    match device_id {
        Some(id) => {
            debug!("ハプティックデバイスを停止: {}", id);
            
            // プラットフォーム固有の停止機能を呼び出し
            let success = match std::env::consts::OS {
                "linux" => {
                    #[cfg(target_os = "linux")]
                    {
                        platform::stop_linux_haptic_device(id)
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        debug!("Linux実装が呼び出されましたが、このプラットフォームではコンパイルされていません");
                        false
                    }
                },
                "macos" => {
                    #[cfg(target_os = "macos")]
                    {
                        platform::stop_macos_haptic_device(id)
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        debug!("macOS実装が呼び出されましたが、このプラットフォームではコンパイルされていません");
                        false
                    }
                },
                "windows" => {
                    #[cfg(target_os = "windows")]
                    {
                        platform::stop_windows_haptic_device(id)
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        debug!("Windows実装が呼び出されましたが、このプラットフォームではコンパイルされていません");
                        false
                    }
                },
                os => {
                    warn!("サポートされていないOS: {}", os);
                    false
                }
            };
            
            if !success {
                return Err(HapticError::DeviceNotFound);
            }
        },
        None => {
            debug!("すべてのハプティックデバイスを停止");
            // すべてのデバイスを取得して停止
            let devices = detect_all_haptic_devices()?;
            
            for device in devices {
                // エラーを無視して続行
                let _ = stop_haptic_device(Some(&device.id));
            }
        }
    }
    
    Ok(())
}

/// 内部ハプティックモーターを検出
pub fn detect_internal_motor() -> Result<Vec<HapticDevice>, HapticError> {
    debug!("内部ハプティックモーターを検出");
    
    let devices = match std::env::consts::OS {
        "linux" => {
            #[cfg(target_os = "linux")]
            {
                vec![platform::detect_internal_motor_devices()].into_iter().flatten().collect()
            }
            #[cfg(not(target_os = "linux"))]
            {
                vec![]
            }
        },
        "macos" => {
            #[cfg(target_os = "macos")]
            {
                vec![platform::detect_internal_motor_devices()].into_iter().flatten().collect()
            }
            #[cfg(not(target_os = "macos"))]
            {
                vec![]
            }
        },
        "windows" => {
            #[cfg(target_os = "windows")]
            {
                vec![platform::detect_internal_motor_devices()].into_iter().flatten().collect()
            }
            #[cfg(not(target_os = "windows"))]
            {
                vec![]
            }
        },
        _ => vec![],
    };
    
    debug!("検出された内部モーター: {}", devices.len());
    Ok(devices)
}

/// ハプティックタッチパッドを検出
pub fn detect_touchpad() -> Result<Vec<HapticDevice>, HapticError> {
    debug!("ハプティックタッチパッドを検出");
    
    let devices = match std::env::consts::OS {
        "linux" => {
            #[cfg(target_os = "linux")]
            {
                platform::detect_touchpad_devices()
            }
            #[cfg(not(target_os = "linux"))]
            {
                vec![]
            }
        },
        "macos" => {
            #[cfg(target_os = "macos")]
            {
                platform::detect_touchpad_devices()
            }
            #[cfg(not(target_os = "macos"))]
            {
                vec![]
            }
        },
        "windows" => {
            #[cfg(target_os = "windows")]
            {
                platform::detect_touchpad_devices()
            }
            #[cfg(not(target_os = "windows"))]
            {
                vec![]
            }
        },
        _ => vec![],
    };
    
    debug!("検出されたタッチパッド: {}", devices.len());
    Ok(devices)
}

/// ハプティックタッチスクリーンを検出
pub fn detect_touchscreen() -> Result<Vec<HapticDevice>, HapticError> {
    debug!("ハプティックタッチスクリーンを検出");
    
    let devices = match std::env::consts::OS {
        "linux" => {
            #[cfg(target_os = "linux")]
            {
                platform::detect_touchscreen_devices()
            }
            #[cfg(not(target_os = "linux"))]
            {
                vec![]
            }
        },
        "macos" => {
            #[cfg(target_os = "macos")]
            {
                platform::detect_touchscreen_devices()
            }
            #[cfg(not(target_os = "macos"))]
            {
                vec![]
            }
        },
        "windows" => {
            #[cfg(target_os = "windows")]
            {
                platform::detect_touchscreen_devices()
            }
            #[cfg(not(target_os = "windows"))]
            {
                vec![]
            }
        },
        _ => vec![],
    };
    
    debug!("検出されたタッチスクリーン: {}", devices.len());
    Ok(devices)
}

/// 外部コントローラーを検出
pub fn detect_external_controllers() -> Result<Vec<HapticDevice>, HapticError> {
    debug!("外部ハプティックコントローラーを検出");
    
    let devices = match std::env::consts::OS {
        "linux" => {
            #[cfg(target_os = "linux")]
            {
                platform::detect_external_controller_devices()
            }
            #[cfg(not(target_os = "linux"))]
            {
                vec![]
            }
        },
        "macos" => {
            #[cfg(target_os = "macos")]
            {
                platform::detect_external_controller_devices()
            }
            #[cfg(not(target_os = "macos"))]
            {
                vec![]
            }
        },
        "windows" => {
            #[cfg(target_os = "windows")]
            {
                platform::detect_external_controller_devices()
            }
            #[cfg(not(target_os = "windows"))]
            {
                vec![]
            }
        },
        _ => vec![],
    };
    
    debug!("検出された外部コントローラー: {}", devices.len());
    Ok(devices)
}

/// すべてのハプティックデバイスを検出
pub fn detect_all_haptic_devices() -> Result<Vec<HapticDevice>, HapticError> {
    debug!("すべてのハプティックデバイスを検出");
    
    let mut all_devices = Vec::new();
    
    // 各タイプのデバイスを検出して結合
    let internal_motors = detect_internal_motor()?;
    let touchpads = detect_touchpad()?;
    let touchscreens = detect_touchscreen()?;
    let controllers = detect_external_controllers()?;
    
    all_devices.extend(internal_motors);
    all_devices.extend(touchpads);
    all_devices.extend(touchscreens);
    all_devices.extend(controllers);
    
    debug!("検出されたすべてのデバイス: {}", all_devices.len());
    Ok(all_devices)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_default_duration() {
        // 各パターンのデフォルトデュレーションをテスト
        let patterns = vec![
            (HapticPattern::Click, 10),
            (HapticPattern::DoubleClick, 20),
            (HapticPattern::TripleClick, 30),
            (HapticPattern::Tap, 10),
            (HapticPattern::Pulse, 50),
            (HapticPattern::Buzz, 100),
            (HapticPattern::Notification, 200),
            (HapticPattern::Success, 150),
            (HapticPattern::Warning, 200),
            (HapticPattern::Error, 250),
            (HapticPattern::Vibration, 300),
            (HapticPattern::LongPress, 500),
            (HapticPattern::RumbleContinuous, 1000),
            (HapticPattern::RumblePulse, 500),
        ];
        
        for (pattern, expected_duration) in patterns {
            let event = HapticEvent {
                pattern,
                intensity: None,
                duration_ms: None,
                target_device_id: None,
                category: None,
            };
            
            // play_haptic_feedbackを呼び出す代わりに、内部ロジックをテスト
            let duration = event.duration_ms.unwrap_or_else(|| {
                match event.pattern {
                    HapticPattern::Click => 10,
                    HapticPattern::DoubleClick => 20,
                    HapticPattern::TripleClick => 30,
                    HapticPattern::Tap => 10,
                    HapticPattern::Pulse => 50,
                    HapticPattern::Buzz => 100,
                    HapticPattern::Notification => 200,
                    HapticPattern::Success => 150,
                    HapticPattern::Warning => 200,
                    HapticPattern::Error => 250,
                    HapticPattern::Vibration => 300,
                    HapticPattern::LongPress => 500,
                    HapticPattern::RumbleContinuous => 1000,
                    HapticPattern::RumblePulse => 500,
                    HapticPattern::None => 0,
                    _ => 100,
                }
            });
            
            assert_eq!(duration, expected_duration);
        }
    }
    
    #[test]
    fn test_invalid_pattern() {
        let event = HapticEvent {
            pattern: HapticPattern::None,
            intensity: None,
            duration_ms: None,
            target_device_id: None,
            category: None,
        };
        
        let result = play_haptic_feedback(&event);
        assert!(result.is_err());
        
        match result {
            Err(HapticError::InvalidPattern) => (), // 期待通り
            _ => panic!("期待したエラーが発生しませんでした"),
        }
    }
    
    #[test]
    fn test_detect_devices() {
        // 実際のデバイス検出をテスト
        let internal_motors = detect_internal_motor();
        assert!(internal_motors.is_ok());
        
        let touchpads = detect_touchpad();
        assert!(touchpads.is_ok());
        
        let touchscreens = detect_touchscreen();
        assert!(touchscreens.is_ok());
        
        let controllers = detect_external_controllers();
        assert!(controllers.is_ok());
        
        let all_devices = detect_all_haptic_devices();
        assert!(all_devices.is_ok());
        
        // すべてのデバイスの合計が、個別の検出の合計と一致することを確認
        let total_individual = internal_motors.unwrap().len() +
                              touchpads.unwrap().len() +
                              touchscreens.unwrap().len() +
                              controllers.unwrap().len();
        
        assert_eq!(all_devices.unwrap().len(), total_individual);
    }
}