// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
// 
// Windows ハプティックシステム実装
// Copyright (c) 2023-2024 AetherOS Team.

use log::{debug, info, warn, error};
use crate::system::haptics::{
    HapticDevice, 
    HapticDeviceType, 
    HapticEvent,
    HapticEventResult,
    HapticError,
    HapticPattern,
    HapticIntensity
};
use std::sync::{Arc, Mutex, Once};
use std::collections::HashMap;
use lazy_static::lazy_static;
use winapi::um::xinput;
use winapi::shared::minwindef::DWORD;

// Windows デバイス識別子
const DEVICE_PRECISION_TOUCHPAD: &str = "Windows_PrecisionTouchpad";
const DEVICE_TOUCH_DISPLAY: &str = "Windows_TouchDisplay";
const DEVICE_XBOX_CONTROLLER: &str = "Windows_XboxController";
const DEVICE_SURFACE_DIAL: &str = "Windows_SurfaceDial";
const DEVICE_SURFACE_PEN: &str = "Windows_SurfacePen";

// Windows ハプティックデバイスタイプ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowsHapticDeviceType {
    // Windows Vibration API (コントローラー)
    GameController,
    // Windows Precision Touchpad API
    PrecisionTouchpad,
    // Windows タッチディスプレイ
    TouchDisplay,
    // Surface Dial
    SurfaceDial,
    // Surface Pen
    SurfacePen,
    // その他のデバイス
    Other,
}

// Windows ハプティックフィードバックタイプ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowsHapticFeedbackType {
    // Windows.UI.Input.Preview.Feedback API (Precision Touchpad)
    PrecisionTouchpadFeedback,
    // XInput バイブレーション (Xbox/互換コントローラー)
    XInputVibration,
    // Windows.Devices.Haptics API
    WindowsHapticsAPI,
    // Surface Dial 専用 API
    SurfaceDialFeedback,
    // その他のフィードバック方式
    Other,
}

// Windows ハプティックパターン
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowsHapticPattern {
    // 標準パターン
    Click,
    DoubleClick,
    Press,
    Release,
    RightEdge,
    LeftEdge,
    TopEdge,
    BottomEdge,
    // Precision Touchpad 専用
    TouchpadButtonDown,
    TouchpadButtonUp,
    TouchpadGestureStart,
    TouchpadGestureEnd,
    // Surface ハードウェア専用
    SurfaceDialRotate,
    SurfaceDialClick,
    SurfacePenTap,
    // システム通知
    SystemAttention,
    SystemNotification,
    // Xbox コントローラー
    XboxLeftMotor,
    XboxRightMotor,
    XboxTrigger,
    // カスタムパターン
    Custom(u16),
}

// Windowsハプティックデバイス
#[derive(Debug, Clone)]
pub struct WindowsHapticDevice {
    // 基本情報
    pub id: String,
    pub name: String,
    pub device_type: WindowsHapticDeviceType,
    pub feedback_type: WindowsHapticFeedbackType,
    pub connected: bool,
    
    // デバイス固有情報
    pub xinput_index: Option<u32>,          // XInputデバイスの場合
    pub touchpad_info: Option<String>,      // Precision Touchpadの場合
    pub device_instance_id: Option<String>, // Windows デバイスインスタンスID
    
    // 機能サポート
    pub supports_pressure: bool,
    pub supports_custom_patterns: bool,
    
    // 設定
    pub max_intensity: f32,
    pub min_intensity: f32,
}

// Windows ハプティックシステム状態
struct WindowsHapticSystem {
    initialized: bool,
    devices: HashMap<String, WindowsHapticDevice>,
}

lazy_static! {
    static ref WINDOWS_HAPTIC_SYSTEM: Arc<Mutex<WindowsHapticSystem>> = Arc::new(Mutex::new(
        WindowsHapticSystem {
            initialized: false,
            devices: HashMap::new(),
        }
    ));
    static ref INIT_ONCE: Once = Once::new();
    static ref DEVICE_STATES: Arc<Mutex<HashMap<String, HapticState>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

/// Windows ハプティックシステムを初期化
pub fn initialize_windows_haptics() -> Result<(), HapticError> {
    let mut system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
    
    if system.initialized {
        debug!("Windows ハプティックシステムは既に初期化されています");
        return Ok(());
    }
    
    info!("Windows ハプティックシステムを初期化しています...");
    
    // 実際の実装では、次のようなことを行います:
    // 1. Windows.Devices.Haptics API が利用可能かチェック
    // 2. XInput DLL をロード
    // 3. 利用可能なデバイスを検索
    
    // この実装ではモックデバイスを追加します
    add_mock_devices(&mut system);
    
    system.initialized = true;
    info!("Windows ハプティックシステムの初期化が完了しました");
    Ok(())
}

/// Windows ハプティックシステムをシャットダウン
pub fn shutdown_windows_haptics() {
    let mut system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
    
    if !system.initialized {
        return;
    }
    
    info!("Windows ハプティックシステムをシャットダウンしています...");
    
    // 全てのデバイスを停止
    for (id, _) in system.devices.iter() {
        let _ = stop_windows_haptic_device(Some(id));
    }
    
    system.devices.clear();
    system.initialized = false;
    
    info!("Windows ハプティックシステムがシャットダウンされました");
}

/// Windowsのハプティックイベントを再生
pub fn play_windows_haptic_event(event: &HapticEvent) -> Result<HapticEventResult, HapticError> {
    let system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
    
    if !system.initialized {
        return Err(HapticError::SystemNotInitialized);
    }
    
    debug!("Windows ハプティックイベントを再生: {:?}", event.pattern);
    
    // 使用するデバイスを決定
    let device_id = match &event.target_device_id {
        Some(id) => id.clone(),
        None => {
            // 最適なデバイスを選択
            select_best_device_for_pattern(&system, &event.pattern)?
        }
    };
    
    // デバイスを取得
    let device = match system.devices.get(&device_id) {
        Some(dev) => dev,
        None => return Err(HapticError::DeviceNotFound(device_id)),
    };
    
    if !device.connected {
        return Err(HapticError::ConnectionFailed(format!("デバイス {} は接続されていません", device.name)));
    }
    
    // パターンをWindows固有のパターンに変換
    let windows_pattern = convert_generic_to_windows_pattern(&event.pattern);
    
    // 強度を取得
    let intensity = event.intensity.unwrap_or(HapticIntensity::Medium);
    let intensity_value = match intensity {
        HapticIntensity::None => 0.0,
        HapticIntensity::VeryLight => 0.2,
        HapticIntensity::Light => 0.4,
        HapticIntensity::Medium => 0.6,
        HapticIntensity::Strong => 0.8,
        HapticIntensity::VeryStrong => 1.0,
    };
    
    // デバイスタイプに基づいてハプティックイベントを再生
    match device.feedback_type {
        WindowsHapticFeedbackType::XInputVibration => {
            // XInputコントローラーの場合
            if let Some(index) = device.xinput_index {
                debug!("XInput コントローラー {} のバイブレーションを再生", index);
                
                // 実際の実装ではここでXInputSetState APIを呼び出します
                // 開発段階ではダミー実装のみ
                debug!("XInput: Index={}, Intensity={}", index, intensity_value);
            } else {
                return Err(HapticError::Unknown("XInputインデックスが不正です".to_string()));
            }
        },
        WindowsHapticFeedbackType::PrecisionTouchpadFeedback => {
            // Precision Touchpadの場合
            debug!("Precision Touchpad ハプティックフィードバックを再生: {:?}", windows_pattern);
            
            // 実際の実装ではここでWindows.UI.Input.Preview.Feedback APIを呼び出します
            // 開発段階ではダミー実装のみ
            debug!("Touchpad: Pattern={:?}, Intensity={}", windows_pattern, intensity_value);
        },
        WindowsHapticFeedbackType::WindowsHapticsAPI => {
            // Windows.Devices.Haptics APIを使用するデバイス
            debug!("Windows Haptics API フィードバックを再生: {:?}", windows_pattern);
            
            // 実際の実装ではここでWindows Runtime APIを呼び出します
            // 開発段階ではダミー実装のみ
            debug!("WinRT Haptics: Pattern={:?}, Intensity={}", windows_pattern, intensity_value);
        },
        WindowsHapticFeedbackType::SurfaceDialFeedback => {
            // Surface Dial
            debug!("Surface Dial ハプティックフィードバックを再生: {:?}", windows_pattern);
            
            // 実際の実装ではここでSurface Dial専用APIを呼び出します
            // 開発段階ではダミー実装のみ
            debug!("Surface Dial: Pattern={:?}, Intensity={}", windows_pattern, intensity_value);
        },
        _ => {
            return Err(HapticError::UnsupportedPattern(
                format!("サポートされていないフィードバックタイプ: {:?}", device.feedback_type)
            ));
        }
    }
    
    // 成功結果を返す
    Ok(HapticEventResult {
        success: true,
        device_id: device.id.clone(),
    })
}

/// Windows ハプティックデバイスを停止
pub fn stop_windows_haptic_device(device_id: Option<&str>) -> Result<(), HapticError> {
    let system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
    
    if !system.initialized {
        return Err(HapticError::SystemNotInitialized);
    }
    
    match device_id {
        Some(id) => {
            // 特定のデバイスを停止
            if let Some(device) = system.devices.get(id) {
                debug!("Windows ハプティックデバイスを停止: {}", device.name);
                
                match device.feedback_type {
                    WindowsHapticFeedbackType::XInputVibration => {
                        if let Some(index) = device.xinput_index {
                            // XInputSetState で振動を0に設定
                            debug!("XInput コントローラー {} の振動を停止", index);
                        }
                    },
                    _ => {
                        // その他のデバイスタイプを停止
                        debug!("{:?} タイプのデバイスを停止", device.feedback_type);
                    }
                }
                
                Ok(())
            } else {
                Err(HapticError::DeviceNotFound(id.to_string()))
            }
        },
        None => {
            // すべてのデバイスを停止
            debug!("すべての Windows ハプティックデバイスを停止");
            
            for (_, device) in system.devices.iter() {
                if device.connected {
                    match device.feedback_type {
                        WindowsHapticFeedbackType::XInputVibration => {
                            if let Some(index) = device.xinput_index {
                                // XInputSetState で振動を0に設定
                                debug!("XInput コントローラー {} の振動を停止", index);
                            }
                        },
                        _ => {
                            // その他のデバイスタイプを停止
                            debug!("{:?} タイプのデバイスを停止", device.feedback_type);
                        }
                    }
                }
            }
            
            Ok(())
        }
    }
}

/// 内蔵ハプティックデバイスを検出
pub fn detect_internal_haptic_devices() -> Result<Vec<HapticDevice>, HapticError> {
    let system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
    
    if !system.initialized {
        if let Err(e) = initialize_windows_haptics() {
            return Err(e);
        }
    }
    
    // Windows では標準的な「内蔵モーター」はほとんどないため、空のリストを返す
    // Surface デバイスなど一部のケースを除く
    let mut result = Vec::new();
    
    for (_, device) in system.devices.iter() {
        // Surface デバイスの一部が内部モーターを持っている可能性がある
        if device.device_type == WindowsHapticDeviceType::SurfaceDial && device.connected {
            result.push(HapticDevice {
                id: device.id.clone(),
                name: device.name.clone(),
                device_type: HapticDeviceType::InternalMotor,
            });
        }
    }
    
    Ok(result)
}

/// タッチパッドハプティックデバイスを検出
pub fn detect_touchpad_haptic_devices() -> Result<Vec<HapticDevice>, HapticError> {
    let system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
    
    if !system.initialized {
        if let Err(e) = initialize_windows_haptics() {
            return Err(e);
        }
    }
    
    let mut result = Vec::new();
    
    for (_, device) in system.devices.iter() {
        if device.device_type == WindowsHapticDeviceType::PrecisionTouchpad && device.connected {
            result.push(HapticDevice {
                id: device.id.clone(),
                name: device.name.clone(),
                device_type: HapticDeviceType::Touchpad,
            });
        }
    }
    
    Ok(result)
}

/// タッチスクリーンハプティックデバイスを検出
pub fn detect_touchscreen_haptic_devices() -> Result<Vec<HapticDevice>, HapticError> {
    let system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
    
    if !system.initialized {
        if let Err(e) = initialize_windows_haptics() {
            return Err(e);
        }
    }
    
    let mut result = Vec::new();
    
    for (_, device) in system.devices.iter() {
        if device.device_type == WindowsHapticDeviceType::TouchDisplay && device.connected {
            result.push(HapticDevice {
                id: device.id.clone(),
                name: device.name.clone(),
                device_type: HapticDeviceType::TouchScreen,
            });
        }
    }
    
    Ok(result)
}

/// コントローラーハプティックデバイスを検出（ゲームパッド、VRコントローラーなど）
pub fn detect_controller_haptic_devices() -> Result<Vec<HapticDevice>, HapticError> {
    let system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
    
    if !system.initialized {
        if let Err(e) = initialize_windows_haptics() {
            return Err(e);
        }
    }
    
    let mut result = Vec::new();
    
    for (_, device) in system.devices.iter() {
        if device.device_type == WindowsHapticDeviceType::GameController && device.connected {
            result.push(HapticDevice {
                id: device.id.clone(),
                name: device.name.clone(),
                device_type: HapticDeviceType::GameController,
            });
        }
    }
    
    Ok(result)
}

// 汎用パターンをWindows固有のパターンに変換
fn convert_generic_to_windows_pattern(pattern: &HapticPattern) -> WindowsHapticPattern {
    match pattern {
        HapticPattern::Click => WindowsHapticPattern::Click,
        HapticPattern::DoubleClick => WindowsHapticPattern::DoubleClick,
        HapticPattern::LongPress => WindowsHapticPattern::Press,
        HapticPattern::Success => WindowsHapticPattern::SystemNotification,
        HapticPattern::Warning => WindowsHapticPattern::SystemAttention,
        HapticPattern::Error => WindowsHapticPattern::SystemAttention,
        HapticPattern::Custom(name) => {
            if name == "touchpad_button_down" {
                WindowsHapticPattern::TouchpadButtonDown
            } else if name == "touchpad_button_up" {
                WindowsHapticPattern::TouchpadButtonUp
            } else if name == "surface_dial_rotate" {
                WindowsHapticPattern::SurfaceDialRotate
            } else if name == "surface_dial_click" {
                WindowsHapticPattern::SurfaceDialClick
            } else if name == "surface_pen_tap" {
                WindowsHapticPattern::SurfacePenTap
            } else if name == "xbox_left_motor" {
                WindowsHapticPattern::XboxLeftMotor
            } else if name == "xbox_right_motor" {
                WindowsHapticPattern::XboxRightMotor
            } else if name == "xbox_trigger" {
                WindowsHapticPattern::XboxTrigger
            } else {
                // デフォルトはカスタムパターン0
                WindowsHapticPattern::Custom(0)
            }
        }
    }
}

// パターンに最適なデバイスを選択
fn select_best_device_for_pattern(
    system: &WindowsHapticSystem,
    pattern: &HapticPattern
) -> Result<String, HapticError> {
    // パターンに基づいて最適なデバイスを選択
    match pattern {
        HapticPattern::Click | HapticPattern::DoubleClick | HapticPattern::LongPress => {
            // まずタッチパッドを試す
            for (id, device) in system.devices.iter() {
                if device.device_type == WindowsHapticDeviceType::PrecisionTouchpad && device.connected {
                    return Ok(id.clone());
                }
            }
            
            // 次に Surface デバイス
            for (id, device) in system.devices.iter() {
                if (device.device_type == WindowsHapticDeviceType::SurfaceDial || 
                    device.device_type == WindowsHapticDeviceType::SurfacePen) && 
                   device.connected {
                    return Ok(id.clone());
                }
            }
        },
        HapticPattern::Custom(name) => {
            // パターン名に基づいて特定のデバイスを選択
            if name.contains("touchpad") {
                for (id, device) in system.devices.iter() {
                    if device.device_type == WindowsHapticDeviceType::PrecisionTouchpad && device.connected {
                        return Ok(id.clone());
                    }
                }
            } else if name.contains("surface_dial") {
                for (id, device) in system.devices.iter() {
                    if device.device_type == WindowsHapticDeviceType::SurfaceDial && device.connected {
                        return Ok(id.clone());
                    }
                }
            } else if name.contains("surface_pen") {
                for (id, device) in system.devices.iter() {
                    if device.device_type == WindowsHapticDeviceType::SurfacePen && device.connected {
                        return Ok(id.clone());
                    }
                }
            } else if name.contains("xbox") || name.contains("controller") {
                for (id, device) in system.devices.iter() {
                    if device.device_type == WindowsHapticDeviceType::GameController && device.connected {
                        return Ok(id.clone());
                    }
                }
            }
        },
        _ => {}
    }
    
    // デフォルトでは接続された最初のデバイスを使用
    for (id, device) in system.devices.iter() {
        if device.connected {
            return Ok(id.clone());
        }
    }
    
    // 利用可能なデバイスがない場合はエラー
    Err(HapticError::NoDeviceFound)
}

// モックデバイスを追加（テスト用）
fn add_mock_devices(system: &mut WindowsHapticSystem) {
    // Xbox コントローラー
    let xbox_controller = WindowsHapticDevice {
        id: DEVICE_XBOX_CONTROLLER.to_string(),
        name: "Xbox Wireless Controller".to_string(),
        device_type: WindowsHapticDeviceType::GameController,
        feedback_type: WindowsHapticFeedbackType::XInputVibration,
        connected: true,
        xinput_index: Some(0),
        touchpad_info: None,
        device_instance_id: Some("HID\\VID_045E&PID_02FD\\6&3bd4b5c&0&0000".to_string()),
        supports_pressure: false,
        supports_custom_patterns: false,
        max_intensity: 1.0,
        min_intensity: 0.0,
    };
    
    // Precision Touchpad
    let precision_touchpad = WindowsHapticDevice {
        id: DEVICE_PRECISION_TOUCHPAD.to_string(),
        name: "Precision Touchpad".to_string(),
        device_type: WindowsHapticDeviceType::PrecisionTouchpad,
        feedback_type: WindowsHapticFeedbackType::PrecisionTouchpadFeedback,
        connected: true,
        xinput_index: None,
        touchpad_info: Some("HID-compliant precision touchpad".to_string()),
        device_instance_id: Some("HID\\VID_06CB&PID_0000\\2&33d83c38&0&0000".to_string()),
        supports_pressure: true,
        supports_custom_patterns: true,
        max_intensity: 1.0,
        min_intensity: 0.0,
    };
    
    // Surface Dial
    let surface_dial = WindowsHapticDevice {
        id: DEVICE_SURFACE_DIAL.to_string(),
        name: "Surface Dial".to_string(),
        device_type: WindowsHapticDeviceType::SurfaceDial,
        feedback_type: WindowsHapticFeedbackType::SurfaceDialFeedback,
        connected: false, // 接続なし (テスト用)
        xinput_index: None,
        touchpad_info: None,
        device_instance_id: Some("HID\\VID_045E&PID_091B\\6&3bd4b5c&0&0000".to_string()),
        supports_pressure: false,
        supports_custom_patterns: true,
        max_intensity: 1.0,
        min_intensity: 0.0,
    };
    
    // デバイスを追加
    system.devices.insert(DEVICE_XBOX_CONTROLLER.to_string(), xbox_controller);
    system.devices.insert(DEVICE_PRECISION_TOUCHPAD.to_string(), precision_touchpad);
    system.devices.insert(DEVICE_SURFACE_DIAL.to_string(), surface_dial);
    
    info!("モックデバイスを追加しました: {} 台のデバイス", system.devices.len());
}

// デバイスIDからデバイスを検索
fn find_device(system: &WindowsHapticSystem, device_id: &str) -> Option<&WindowsHapticDevice> {
    system.devices.get(device_id)
}

impl std::fmt::Display for WindowsHapticFeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowsHapticFeedbackType::PrecisionTouchpadFeedback => write!(f, "Precision Touchpad"),
            WindowsHapticFeedbackType::XInputVibration => write!(f, "XInput バイブレーション"),
            WindowsHapticFeedbackType::WindowsHapticsAPI => write!(f, "Windows Haptics API"),
            WindowsHapticFeedbackType::SurfaceDialFeedback => write!(f, "Surface Dial"),
            WindowsHapticFeedbackType::Other => write!(f, "その他"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::haptics::HapticEvent;
    
    #[test]
    fn test_initialize_shutdown() {
        // 初期化テスト
        let init_result = initialize_windows_haptics();
        assert!(init_result.is_ok());
        
        // システム状態を確認
        {
            let system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
            assert!(system.initialized);
            assert!(!system.devices.is_empty());
        }
        
        // シャットダウンテスト
        shutdown_windows_haptics();
        
        // システム状態を確認
        {
            let system = WINDOWS_HAPTIC_SYSTEM.lock().unwrap();
            assert!(!system.initialized);
            assert!(system.devices.is_empty());
        }
    }
    
    #[test]
    fn test_device_detection() {
        // 初期化
        let _ = initialize_windows_haptics();
        
        // タッチパッド検出
        let touchpads = detect_touchpad_haptic_devices().unwrap();
        assert!(!touchpads.is_empty());
        assert_eq!(touchpads[0].device_type, HapticDeviceType::Touchpad);
        
        // コントローラー検出
        let controllers = detect_controller_haptic_devices().unwrap();
        assert!(!controllers.is_empty());
        assert_eq!(controllers[0].device_type, HapticDeviceType::GameController);
        
        // シャットダウン
        shutdown_windows_haptics();
    }
    
    #[test]
    fn test_play_feedback() {
        // 初期化
        let _ = initialize_windows_haptics();
        
        // クリックイベントを作成
        let click_event = HapticEvent::new(HapticPattern::Click);
        
        // イベントを再生
        let result = play_windows_haptic_event(&click_event);
        assert!(result.is_ok());
        let result_data = result.unwrap();
        assert!(result_data.success);
        
        // カスタムイベントを作成
        let mut custom_event = HapticEvent::new(HapticPattern::Custom("xbox_left_motor".to_string()));
        custom_event.set_intensity(HapticIntensity::Strong);
        
        // イベントを再生
        let result = play_windows_haptic_event(&custom_event);
        assert!(result.is_ok());
        
        // シャットダウン
        shutdown_windows_haptics();
    }
    
    #[test]
    fn test_stop_device() {
        // 初期化
        let _ = initialize_windows_haptics();
        
        // Xboxコントローラーを停止
        let result = stop_windows_haptic_device(Some(DEVICE_XBOX_CONTROLLER));
        assert!(result.is_ok());
        
        // 存在しないデバイスを停止
        let result = stop_windows_haptic_device(Some("non_existent_device"));
        assert!(result.is_err());
        
        // すべてのデバイスを停止
        let result = stop_windows_haptic_device(None);
        assert!(result.is_ok());
        
        // シャットダウン
        shutdown_windows_haptics();
    }
    
    #[test]
    fn test_pattern_conversion() {
        // 標準パターン
        assert_eq!(convert_generic_to_windows_pattern(&HapticPattern::Click), WindowsHapticPattern::Click);
        assert_eq!(convert_generic_to_windows_pattern(&HapticPattern::DoubleClick), WindowsHapticPattern::DoubleClick);
        assert_eq!(convert_generic_to_windows_pattern(&HapticPattern::LongPress), WindowsHapticPattern::Press);
        
        // カスタムパターン
        assert_eq!(
            convert_generic_to_windows_pattern(&HapticPattern::Custom("touchpad_button_down".to_string())),
            WindowsHapticPattern::TouchpadButtonDown
        );
        assert_eq!(
            convert_generic_to_windows_pattern(&HapticPattern::Custom("xbox_left_motor".to_string())),
            WindowsHapticPattern::XboxLeftMotor
        );
    }
} 