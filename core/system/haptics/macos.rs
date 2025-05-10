// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
// 
// macOS ハプティックシステム実装
// Copyright (c) 2023-2024 AetherOS Team.

use log::{debug, info, warn, error};
use std::sync::{Arc, Mutex, Once};
use std::collections::HashMap;
use std::time::Duration;
use lazy_static::lazy_static;

use crate::core::system::haptics::{
    HapticDevice, HapticDeviceType, HapticPattern, HapticIntensity,
    HapticEvent, HapticEventResult, HapticError
};

// macOS API用の定数
const DEVICE_FORCE_TOUCH: &str = "Force Touchトラックパッド";
const DEVICE_TOUCH_BAR: &str = "Touch Bar";
const DEVICE_TRACKPAD: &str = "Trackpad";
const DEVICE_TOUCH_ID: &str = "Touch ID";

/// macOS用ハプティックデバイスの種類
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MacOSHapticDeviceType {
    /// Force Touchトラックパッド
    ForceTouchTrackpad,
    /// タッチバー
    TouchBar,
    /// 一般的なトラックパッド（ハプティック機能なし）
    Trackpad,
    /// その他のハプティックデバイス
    Other,
}

/// macOS用ハプティックフィードバックタイプ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MacOSHapticFeedbackType {
    /// NSHapticFeedbackManager
    NSHapticFeedback,
    /// UI Kit Feedback Generator（Catalyst用）
    UIKitFeedback,
    /// Core Haptics
    CoreHaptics,
    /// サウンドフィードバック（ハプティック対応デバイスがない場合）
    SoundFeedback,
}

/// macOS用ハプティックフィードバックパターン（NSHapticFeedbackManager）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MacOSHapticPattern {
    /// Generic - 一般的なフィードバック
    Generic,
    /// Alignment - 整列
    Alignment,
    /// LevelChange - レベル変更
    LevelChange,
}

/// macOS用ハプティックデバイス情報
#[derive(Debug, Clone)]
struct MacOSHapticDevice {
    /// デバイスID
    id: String,
    /// デバイス名
    name: String,
    /// デバイスタイプ
    device_type: MacOSHapticDeviceType,
    /// フィードバックタイプ
    feedback_type: MacOSHapticFeedbackType,
    /// 接続状態
    connected: bool,
    /// デバイス情報（macOS固有）
    device_info: HashMap<String, String>,
}

// macOS ハプティックシステムの状態
static MACOS_HAPTIC_SYSTEM_INITIALIZED: Once = Once::new();
static MACOS_HAPTIC_DEVICES: once_cell::sync::Lazy<Mutex<Vec<MacOSHapticDevice>>> = 
    once_cell::sync::Lazy::new(|| Mutex::new(Vec::new()));

/// macOSハプティックシステムAPI初期化
pub fn initialize_macos_haptics() -> Result<(), HapticError> {
    let mut initialized = false;
    
    MACOS_HAPTIC_SYSTEM_INITIALIZED.call_once(|| {
        // macOS APIのハプティックサブシステムを初期化
        info!("macOS ハプティックサブシステムを初期化しています...");
        
        // デバイスの検出と登録を行う
        let mut devices = MACOS_HAPTIC_DEVICES.lock().unwrap();
        devices.clear();
        
        // 利用可能なデバイスを追加（現状はモックデータ）
        add_mock_devices(&mut devices);
        
        info!("macOS ハプティックサブシステムを初期化しました: {} デバイス検出", devices.len());
        initialized = true;
    });
    
    if initialized {
        Ok(())
    } else {
        Err(HapticError::Unknown("macOS ハプティックシステムの初期化に失敗しました".to_string()))
    }
}

/// macOSハプティックシステムのシャットダウン
pub fn shutdown_macos_haptics() {
    info!("macOS ハプティックサブシステムをシャットダウンしています...");
    
    // 実行中のすべてのハプティックフィードバックを停止
    let _ = stop_macos_haptic_device(None);
    
    // デバイスリストをクリア
    let mut devices = MACOS_HAPTIC_DEVICES.lock().unwrap();
    devices.clear();
    
    info!("macOS ハプティックサブシステムをシャットダウンしました");
}

/// macOS ハプティックイベントを再生
pub fn play_macos_haptic_event(event: &HapticEvent) -> Result<HapticEventResult, HapticError> {
    debug!("macOS ハプティックイベントを再生: {:?}", event);
    
    // ターゲットデバイスの選択
    let device = if let Some(device_id) = &event.target_device_id {
        find_device(device_id)?
    } else {
        select_best_device_for_pattern(&event.pattern)?
    };
    
    // デバイスが接続されているか確認
    if !device.connected {
        return Err(HapticError::ConnectionFailed(format!("デバイス {} は接続されていません", device.name)));
    }
    
    // 強度の設定
    let intensity = event.intensity.unwrap_or(HapticIntensity::Medium);
    
    // パターンをmacOS固有パターンに変換
    let macos_pattern = convert_generic_to_macos_pattern(&event.pattern);
    
    // イベントタイプに基づいて処理
    match device.feedback_type {
        MacOSHapticFeedbackType::NSHapticFeedback => {
            play_nshaptic_feedback(&device, macos_pattern, intensity)?;
        },
        MacOSHapticFeedbackType::UIKitFeedback => {
            play_uikit_feedback(&device, &event.pattern, intensity)?;
        },
        MacOSHapticFeedbackType::CoreHaptics => {
            play_core_haptics(&device, &event.pattern, intensity)?;
        },
        MacOSHapticFeedbackType::SoundFeedback => {
            play_sound_feedback(&device, &event.pattern, intensity)?;
        },
    }
    
    Ok(HapticEventResult {
        success: true,
        device_id: device.id.clone(),
    })
}

/// macOS ハプティックデバイスを停止
pub fn stop_macos_haptic_device(device_id: Option<&str>) -> Result<(), HapticError> {
    if let Some(id) = device_id {
        // 特定のデバイスを停止
        debug!("macOS ハプティックデバイスを停止: {}", id);
        let device = find_device(id)?;
        stop_device_feedback(&device)
    } else {
        // すべてのデバイスを停止
        debug!("すべての macOS ハプティックデバイスを停止");
        let devices = MACOS_HAPTIC_DEVICES.lock().unwrap();
        for device in devices.iter() {
            if device.connected {
                if let Err(e) = stop_device_feedback(device) {
                    warn!("デバイス {} の停止中にエラー: {:?}", device.name, e);
                }
            }
        }
        Ok(())
    }
}

/// 内蔵ハプティックデバイスを検出
pub fn detect_internal_haptic_devices() -> Result<Vec<HapticDevice>, HapticError> {
    let devices = MACOS_HAPTIC_DEVICES.lock().unwrap();
    let internal_devices: Vec<HapticDevice> = devices.iter()
        .filter(|d| matches!(d.device_type, 
            MacOSHapticDeviceType::ForceTouchTrackpad |
            MacOSHapticDeviceType::TouchBar))
        .filter(|d| d.connected)
        .map(macos_device_to_generic)
        .collect();
    
    Ok(internal_devices)
}

/// タッチパッドハプティックデバイスを検出
pub fn detect_touchpad_haptic_devices() -> Result<Vec<HapticDevice>, HapticError> {
    let devices = MACOS_HAPTIC_DEVICES.lock().unwrap();
    let touchpad_devices: Vec<HapticDevice> = devices.iter()
        .filter(|d| matches!(d.device_type, 
            MacOSHapticDeviceType::ForceTouchTrackpad |
            MacOSHapticDeviceType::Trackpad))
        .filter(|d| d.connected)
        .map(macos_device_to_generic)
        .collect();
    
    Ok(touchpad_devices)
}

/// タッチスクリーンハプティックデバイスを検出
pub fn detect_touchscreen_haptic_devices() -> Result<Vec<HapticDevice>, HapticError> {
    // macOS では、標準のmacのラップトップやデスクトップには現在タッチスクリーンが搭載されていないため、空のリストを返す
    Ok(Vec::new())
}

/// コントローラーハプティックデバイスを検出
pub fn detect_controller_haptic_devices() -> Result<Vec<HapticDevice>, HapticError> {
    // 現在の実装では外部コントローラを検出していないため、空のリストを返す
    // 将来的にはここにゲームコントローラーなどの検出ロジックを追加
    Ok(Vec::new())
}

// ヘルパー関数

/// 指定されたIDのデバイスを検索
fn find_device(device_id: &str) -> Result<MacOSHapticDevice, HapticError> {
    let devices = MACOS_HAPTIC_DEVICES.lock().unwrap();
    devices.iter()
        .find(|d| d.id == device_id)
        .cloned()
        .ok_or_else(|| HapticError::DeviceNotFound(device_id.to_string()))
}

/// 指定されたパターンに最適なデバイスを選択
fn select_best_device_for_pattern(pattern: &HapticPattern) -> Result<MacOSHapticDevice, HapticError> {
    let devices = MACOS_HAPTIC_DEVICES.lock().unwrap();
    
    // 接続済みデバイスのみをフィルタリング
    let connected_devices: Vec<&MacOSHapticDevice> = devices.iter()
        .filter(|d| d.connected)
        .collect();
    
    if connected_devices.is_empty() {
        return Err(HapticError::NoDeviceFound);
    }
    
    // Force Touchトラックパッドを優先
    if let Some(device) = connected_devices.iter()
        .find(|d| d.device_type == MacOSHapticDeviceType::ForceTouchTrackpad) {
        return Ok((*device).clone());
    }
    
    // Force Touchトラックパッドがない場合は最初の接続済みデバイスを返す
    Ok(connected_devices[0].clone())
}

/// 一般的なパターンをmacOS固有パターンに変換
fn convert_generic_to_macos_pattern(pattern: &HapticPattern) -> MacOSHapticPattern {
    match pattern {
        HapticPattern::Click | HapticPattern::DoubleClick => MacOSHapticPattern::Generic,
        HapticPattern::LongPress => MacOSHapticPattern::LevelChange,
        HapticPattern::Success | HapticPattern::Warning => MacOSHapticPattern::Alignment,
        HapticPattern::Error => MacOSHapticPattern::Generic,
        HapticPattern::Custom(_) => MacOSHapticPattern::Generic,
    }
}

/// NSHapticFeedbackManagerを使用してフィードバックを再生
fn play_nshaptic_feedback(
    device: &MacOSHapticDevice,
    pattern: MacOSHapticPattern,
    intensity: HapticIntensity
) -> Result<(), HapticError> {
    debug!("NSHapticFeedback: {:?} パターンを再生 (デバイス: {})", pattern, device.name);
    
    // 強度を0.0-1.0に変換
    let intensity_value = convert_intensity_to_float(intensity);
    
    // 実際のmacOS APIの代わりにデバッグログを表示
    debug!("NSHapticFeedbackManager.performFeedback({:?}, intensity: {:.1})", pattern, intensity_value);
    
    // 実際のAPIコール
    // TODO: Objective-C連携で実際にNSHapticFeedbackManagerを呼び出す
    
    Ok(())
}

/// UIKitフィードバックジェネレーター（Catalyst用）を使用
fn play_uikit_feedback(
    device: &MacOSHapticDevice,
    pattern: &HapticPattern,
    intensity: HapticIntensity
) -> Result<(), HapticError> {
    debug!("UIKit Feedback Generator: {:?} パターンを再生 (デバイス: {})", pattern, device.name);
    
    // iOS/macOS Catalystスタイルのフィードバック
    match pattern {
        HapticPattern::Click => debug!("UISelectionFeedbackGenerator.selectionChanged()"),
        HapticPattern::DoubleClick => debug!("UIImpactFeedbackGenerator.impactOccurred()"),
        HapticPattern::Success => debug!("UINotificationFeedbackGenerator.notificationOccurred(.success)"),
        HapticPattern::Warning => debug!("UINotificationFeedbackGenerator.notificationOccurred(.warning)"),
        HapticPattern::Error => debug!("UINotificationFeedbackGenerator.notificationOccurred(.error)"),
        HapticPattern::LongPress => debug!("UIImpactFeedbackGenerator.impactOccurred(.heavy)"),
        HapticPattern::Custom(_) => debug!("カスタムUIKitフィードバック（複数のフィードバックの組み合わせ）"),
    }
    
    // 実際のAPIコール
    // TODO: Objective-C連携で実際にUIKitフィードバックジェネレーターを呼び出す
    
    Ok(())
}

/// Core Hapticsを使用（iPhone/iPadシミュレーター環境向け）
fn play_core_haptics(
    device: &MacOSHapticDevice,
    pattern: &HapticPattern,
    intensity: HapticIntensity
) -> Result<(), HapticError> {
    debug!("Core Haptics: {:?} パターンを再生 (デバイス: {})", pattern, device.name);
    
    // 強度を0.0-1.0に変換
    let intensity_value = convert_intensity_to_float(intensity);
    
    // 実際のAPIコール
    // TODO: Objective-C連携で実際にCore Hapticsを呼び出す
    
    Ok(())
}

/// サウンドフィードバック（ハプティック非対応デバイス向け）
fn play_sound_feedback(
    device: &MacOSHapticDevice,
    pattern: &HapticPattern,
    intensity: HapticIntensity
) -> Result<(), HapticError> {
    debug!("サウンドフィードバック: {:?} パターンを再生 (デバイス: {})", pattern, device.name);
    
    // パターンに応じた音声ファイルを選択
    let sound_file = match pattern {
        HapticPattern::Click => "click.wav",
        HapticPattern::DoubleClick => "double_click.wav",
        HapticPattern::LongPress => "long_press.wav",
        HapticPattern::Success => "success.wav",
        HapticPattern::Warning => "warning.wav",
        HapticPattern::Error => "error.wav",
        HapticPattern::Custom(_) => "custom.wav",
    };
    
    debug!("サウンドフィードバックファイル: {}", sound_file);
    
    // 実際のAPIコール
    // TODO: macOSのCore Audioなどを使用して音声を再生
    
    Ok(())
}

/// デバイスのフィードバックを停止
fn stop_device_feedback(device: &MacOSHapticDevice) -> Result<(), HapticError> {
    debug!("macOS ハプティックフィードバックを停止: デバイス {}", device.name);
    
    match device.feedback_type {
        MacOSHapticFeedbackType::NSHapticFeedback => {
            // NSHapticFeedbackManagerには明示的な停止APIはない
            debug!("NSHapticFeedbackの停止（明示的なAPIなし）");
        },
        MacOSHapticFeedbackType::UIKitFeedback => {
            // UIKitフィードバックは自動的に停止する
            debug!("UIKitフィードバックの停止（自動的に停止）");
        },
        MacOSHapticFeedbackType::CoreHaptics => {
            // Core Hapticsエンジンを停止
            debug!("CHHapticEngineを停止");
        },
        MacOSHapticFeedbackType::SoundFeedback => {
            // 音声再生を停止
            debug!("サウンドフィードバックを停止");
        },
    }
    
    Ok(())
}

/// 強度列挙型を0.0-1.0の浮動小数点に変換
fn convert_intensity_to_float(intensity: HapticIntensity) -> f32 {
    match intensity {
        HapticIntensity::None => 0.0,
        HapticIntensity::VeryLight => 0.2,
        HapticIntensity::Light => 0.4,
        HapticIntensity::Medium => 0.6,
        HapticIntensity::Strong => 0.8,
        HapticIntensity::VeryStrong => 1.0,
    }
}

/// macOS固有のデバイス情報を汎用フォーマットに変換
fn macos_device_to_generic(device: &MacOSHapticDevice) -> HapticDevice {
    let device_type = match device.device_type {
        MacOSHapticDeviceType::ForceTouchTrackpad | MacOSHapticDeviceType::Trackpad => HapticDeviceType::Touchpad,
        MacOSHapticDeviceType::TouchBar => HapticDeviceType::TouchBar,
        MacOSHapticDeviceType::Other => HapticDeviceType::Other,
    };
    
    HapticDevice {
        id: device.id.clone(),
        name: device.name.clone(),
        device_type,
    }
}

/// テスト/開発用にモックデバイスをシステムに追加
fn add_mock_devices(devices: &mut Vec<MacOSHapticDevice>) {
    // Force Touchトラックパッド
    devices.push(MacOSHapticDevice {
        id: "mac-forcetouch-001".to_string(),
        name: DEVICE_FORCE_TOUCH.to_string(),
        device_type: MacOSHapticDeviceType::ForceTouchTrackpad,
        feedback_type: MacOSHapticFeedbackType::NSHapticFeedback,
        connected: true,
        device_info: HashMap::new(),
    });
    
    // Touch Bar
    devices.push(MacOSHapticDevice {
        id: "mac-touchbar-001".to_string(),
        name: DEVICE_TOUCH_BAR.to_string(),
        device_type: MacOSHapticDeviceType::TouchBar,
        feedback_type: MacOSHapticFeedbackType::UIKitFeedback,
        connected: true,
        device_info: HashMap::new(),
    });
    
    // 通常のトラックパッド（ハプティック機能なし）
    devices.push(MacOSHapticDevice {
        id: "mac-trackpad-001".to_string(),
        name: DEVICE_TRACKPAD.to_string(),
        device_type: MacOSHapticDeviceType::Trackpad,
        feedback_type: MacOSHapticFeedbackType::SoundFeedback,
        connected: false, // 接続されていない状態
        device_info: HashMap::new(),
    });
    
    // Touch ID（その他のデバイス）
    devices.push(MacOSHapticDevice {
        id: "mac-touchid-001".to_string(),
        name: DEVICE_TOUCH_ID.to_string(),
        device_type: MacOSHapticDeviceType::Other,
        feedback_type: MacOSHapticFeedbackType::UIKitFeedback,
        connected: true,
        device_info: HashMap::new(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::system::haptics::HapticEvent;
    
    #[test]
    fn test_initialize_shutdown() {
        // 初期化テスト
        let init_result = initialize_macos_haptics();
        assert!(init_result.is_ok());
        
        // デバイス検出テスト
        let devices = MACOS_HAPTIC_DEVICES.lock().unwrap();
        assert!(!devices.is_empty());
        
        // シャットダウンテスト
        shutdown_macos_haptics();
    }
    
    #[test]
    fn test_device_detection() {
        initialize_macos_haptics().unwrap();
        
        // 内蔵デバイス検出
        let internal_devices = detect_internal_haptic_devices().unwrap();
        assert!(!internal_devices.is_empty());
        
        // タッチパッド検出
        let touchpad_devices = detect_touchpad_haptic_devices().unwrap();
        assert!(!touchpad_devices.is_empty());
        
        // タッチスクリーン検出（macOSでは現在サポートされていないため空のはず）
        let touchscreen_devices = detect_touchscreen_haptic_devices().unwrap();
        assert!(touchscreen_devices.is_empty());
        
        shutdown_macos_haptics();
    }
    
    #[test]
    fn test_haptic_playback() {
        initialize_macos_haptics().unwrap();
        
        // クリックパターンテスト
        let event = HapticEvent::new(HapticPattern::Click);
        let result = play_macos_haptic_event(&event);
        assert!(result.is_ok());
        
        // 特定のデバイスを対象にしたテスト
        let mut event = HapticEvent::new(HapticPattern::Success);
        event.set_target_device("mac-forcetouch-001");
        let result = play_macos_haptic_event(&event);
        assert!(result.is_ok());
        
        // 強度設定テスト
        let mut event = HapticEvent::new(HapticPattern::Warning);
        event.set_intensity(HapticIntensity::Strong);
        let result = play_macos_haptic_event(&event);
        assert!(result.is_ok());
        
        shutdown_macos_haptics();
    }
    
    #[test]
    fn test_device_stop() {
        initialize_macos_haptics().unwrap();
        
        // すべてのデバイスを停止
        let result = stop_macos_haptic_device(None);
        assert!(result.is_ok());
        
        // 特定のデバイスを停止
        let result = stop_macos_haptic_device(Some("mac-forcetouch-001"));
        assert!(result.is_ok());
        
        // 存在しないデバイスを停止（エラーになるはず）
        let result = stop_macos_haptic_device(Some("non-existent-device"));
        assert!(result.is_err());
        
        shutdown_macos_haptics();
    }
    
    #[test]
    fn test_pattern_conversion() {
        // 一般的なパターンからmacOS固有パターンへの変換をテスト
        assert_eq!(convert_generic_to_macos_pattern(&HapticPattern::Click), MacOSHapticPattern::Generic);
        assert_eq!(convert_generic_to_macos_pattern(&HapticPattern::Success), MacOSHapticPattern::Alignment);
        assert_eq!(convert_generic_to_macos_pattern(&HapticPattern::LongPress), MacOSHapticPattern::LevelChange);
        assert_eq!(convert_generic_to_macos_pattern(&HapticPattern::Error), MacOSHapticPattern::Generic);
    }
} 