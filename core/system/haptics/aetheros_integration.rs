// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
//
// AetherOS ハプティクス統合
// Copyright (c) 2023-2024 AetherOS Team.

use log::{debug, info, warn, error};
use std::sync::{Arc, Mutex};

use super::{
    HapticDevice, 
    HapticDeviceType, 
    HapticEvent, 
    HapticEventResult, 
    HapticError, 
    HapticIntensity, 
    HapticPattern,
    play_haptic_feedback,
    stop_haptic_device,
    detect_internal_motor,
    detect_touchpad,
    detect_touchscreen,
    detect_external_controllers
};

// AetherOS ハプティクスハンドラー
// AetherOSカーネルとの統合ポイント
pub struct AetherOSHapticsHandler {
    // AetherOSカーネルのネイティブハンドル
    native_handle: Option<u64>,
    
    // 利用可能なデバイス
    available_devices: Vec<HapticDevice>,
    
    // デフォルトデバイス
    default_device_id: Option<String>,
    
    // 設定フラグ
    settings: AetherOSHapticsSettings,
    
    // イベントコールバック
    event_callback: Option<Box<dyn Fn(&HapticEventResult) + Send + Sync>>,
}

// ハプティクス設定
#[derive(Debug, Clone)]
pub struct AetherOSHapticsSettings {
    // 自動デバイス検出を有効にする
    auto_detect_devices: bool,
    
    // システム全体のハプティクスを有効にする
    system_haptics_enabled: bool,
    
    // UI要素のハプティクスを有効にする
    ui_haptics_enabled: bool,
    
    // ゲーム/アプリのハプティクスを有効にする
    app_haptics_enabled: bool,
    
    // アクセシビリティのハプティクスを有効にする
    accessibility_haptics_enabled: bool,
    
    // システム設定で定義された強度
    system_intensity: HapticIntensity,
}

impl Default for AetherOSHapticsSettings {
    fn default() -> Self {
        Self {
            auto_detect_devices: true,
            system_haptics_enabled: true,
            ui_haptics_enabled: true,
            app_haptics_enabled: true,
            accessibility_haptics_enabled: true,
            system_intensity: HapticIntensity::Medium,
        }
    }
}

// AetherOSハプティクス設定を表すJSON文字列
pub fn serialize_settings(settings: &AetherOSHapticsSettings) -> String {
    serde_json::to_string(settings).unwrap_or_else(|_| "{}".to_string())
}

// JSON文字列からAetherOSハプティクス設定をパース
pub fn deserialize_settings(json: &str) -> Result<AetherOSHapticsSettings, HapticError> {
    serde_json::from_str(json).map_err(|e| {
        HapticError::Other(format!("設定のパースに失敗しました: {}", e))
    })
}

impl AetherOSHapticsHandler {
    // 新しいハンドラーを作成
    pub fn new() -> Self {
        Self {
            native_handle: None,
            available_devices: Vec::new(),
            default_device_id: None,
            settings: AetherOSHapticsSettings::default(),
            event_callback: None,
        }
    }
    
    // AetherOSハプティクスハンドラーを初期化
    pub fn initialize(&mut self) -> Result<(), HapticError> {
        info!("AetherOSハプティクスハンドラーを初期化中...");
        
        // AetherOSカーネルとネイティブハンドルを確立
        self.establish_native_handle();
        
        // デバイスを検出
        if self.settings.auto_detect_devices {
            self.detect_devices()?;
        }
        
        info!("AetherOSハプティクスハンドラー初期化完了。利用可能なデバイス: {}", self.available_devices.len());
        Ok(())
    }
    
    // ネイティブハンドルを確立
    fn establish_native_handle(&mut self) {
        debug!("AetherOSネイティブハプティクスハンドルを確立中...");
        
        // 実際の実装ではAetherOSカーネルAPIを呼び出す
        // モックアップではダミーハンドルを使用
        self.native_handle = Some(0xAE7H3R05);
        
        debug!("AetherOSネイティブハプティクスハンドル確立: {:?}", self.native_handle);
    }
    
    // 利用可能なデバイスを検出
    pub fn detect_devices(&mut self) -> Result<(), HapticError> {
        debug!("AetherOSハプティクスデバイスを検出中...");
        
        self.available_devices.clear();
        
        // 内部モーターを検出
        let internal_motors = detect_internal_motor()?;
        if !internal_motors.is_empty() {
            debug!("内部モーターデバイスを検出: {}", internal_motors.len());
            self.available_devices.extend(internal_motors);
        }
        
        // タッチパッドを検出
        let touchpads = detect_touchpad()?;
        if !touchpads.is_empty() {
            debug!("タッチパッドデバイスを検出: {}", touchpads.len());
            self.available_devices.extend(touchpads);
        }
        
        // タッチスクリーンを検出
        let touchscreens = detect_touchscreen()?;
        if !touchscreens.is_empty() {
            debug!("タッチスクリーンデバイスを検出: {}", touchscreens.len());
            self.available_devices.extend(touchscreens);
        }
        
        // 外部コントローラーを検出
        let controllers = detect_external_controllers()?;
        if !controllers.is_empty() {
            debug!("外部コントローラーデバイスを検出: {}", controllers.len());
            self.available_devices.extend(controllers);
        }
        
        // デフォルトデバイスを設定
        self.set_default_device();
        
        info!("AetherOSハプティクスデバイス検出完了。合計デバイス数: {}", self.available_devices.len());
        Ok(())
    }
    
    // デフォルトデバイスを設定
    fn set_default_device(&mut self) {
        if self.available_devices.is_empty() {
            self.default_device_id = None;
            return;
        }
        
        // 優先順位: タッチパッド > 内部モーター > タッチスクリーン > 外部コントローラー
        let mut default_device_id = None;
        
        for device in &self.available_devices {
            match device.device_type {
                HapticDeviceType::Touchpad => {
                    default_device_id = Some(device.id.clone());
                    break;
                }
                HapticDeviceType::InternalMotor => {
                    if default_device_id.is_none() {
                        default_device_id = Some(device.id.clone());
                    }
                }
                HapticDeviceType::Touchscreen => {
                    if default_device_id.is_none() || 
                       self.get_device_by_id(default_device_id.as_ref().unwrap())
                         .map(|d| d.device_type == HapticDeviceType::GameController)
                         .unwrap_or(false) {
                        default_device_id = Some(device.id.clone());
                    }
                }
                HapticDeviceType::GameController => {
                    if default_device_id.is_none() {
                        default_device_id = Some(device.id.clone());
                    }
                }
                _ => {}
            }
        }
        
        // デフォルトデバイスIDを設定
        self.default_device_id = default_device_id;
        
        if let Some(device_id) = &self.default_device_id {
            if let Some(device) = self.get_device_by_id(device_id) {
                debug!("デフォルトハプティクスデバイスを設定: {} ({})", device.name, device_id);
            }
        }
    }
    
    // IDによるデバイスの取得
    fn get_device_by_id(&self, device_id: &str) -> Option<&HapticDevice> {
        self.available_devices.iter().find(|d| d.id == device_id)
    }
    
    // デフォルトデバイスIDの取得
    pub fn get_default_device_id(&self) -> Option<String> {
        self.default_device_id.clone()
    }
    
    // 設定を更新
    pub fn update_settings(&mut self, new_settings: AetherOSHapticsSettings) {
        debug!("AetherOSハプティクス設定を更新中...");
        self.settings = new_settings;
        
        // 設定に変更があった場合、デバイスを再検出
        if self.settings.auto_detect_devices {
            if let Err(e) = self.detect_devices() {
                warn!("設定更新後のデバイス検出に失敗しました: {:?}", e);
            }
        }
    }
    
    // イベントコールバックを設定
    pub fn set_event_callback<F>(&mut self, callback: F)
    where
        F: Fn(&HapticEventResult) + Send + Sync + 'static,
    {
        self.event_callback = Some(Box::new(callback));
    }
    
    // ハプティックイベントを再生
    pub fn play_event(&self, mut event: HapticEvent) -> Result<HapticEventResult, HapticError> {
        // ハプティクスが無効な場合はスキップ
        if !self.is_event_allowed(&event) {
            debug!("ハプティクスイベントがスキップされました (設定によって無効): {:?}", event);
            return Ok(HapticEventResult {
                success: false,
                device_id: event.target_device_id.clone().unwrap_or_default(),
            });
        }
        
        // ターゲットデバイスが指定されていない場合、デフォルトを使用
        if event.target_device_id.is_none() {
            event.target_device_id = self.default_device_id.clone();
        }
        
        // システム設定の強度を適用（イベントで強度が指定されていない場合）
        if event.intensity.is_none() {
            event.intensity = Some(self.settings.system_intensity);
        }
        
        // イベントを再生
        debug!("AetherOSハプティクスイベント再生中: {:?}", event);
        
        // 基本のハプティクス機能を使用してイベントを再生
        let result = play_haptic_feedback(&event)?;
        
        // イベント結果をAetherOSカーネルに通知
        self.notify_event_to_kernel(&result);
        
        // コールバックがあれば呼び出し
        if let Some(callback) = &self.event_callback {
            callback(&result);
        }
        
        Ok(result)
    }
    
    // イベントが現在の設定で許可されているか確認
    fn is_event_allowed(&self, event: &HapticEvent) -> bool {
        // システム全体のハプティクスが無効ならすべてのイベントをスキップ
        if !self.settings.system_haptics_enabled {
            return false;
        }
        
        // イベントカテゴリに基づいて確認
        match event.category.as_deref() {
            Some("system") => self.settings.system_haptics_enabled,
            Some("ui") => self.settings.ui_haptics_enabled,
            Some("app") => self.settings.app_haptics_enabled,
            Some("accessibility") => self.settings.accessibility_haptics_enabled,
            _ => true, // カテゴリが指定されていない場合は許可
        }
    }
    
    // イベント結果をAetherOSカーネルに通知
    fn notify_event_to_kernel(&self, result: &HapticEventResult) {
        if let Some(handle) = self.native_handle {
            debug!("ハプティクスイベント結果をAetherOSカーネルに通知: ハンドル={:x}, 成功={}", 
                handle, result.success);
            
            // 実際の実装ではAetherOSカーネルAPIを呼び出す
            // モックアップでは何もしない
        }
    }
    
    // すべてのデバイスを停止
    pub fn stop_all_devices(&self) -> Result<(), HapticError> {
        debug!("すべてのAetherOSハプティクスデバイスを停止中...");
        
        // 基本のハプティクス機能を使用してすべてのデバイスを停止
        stop_haptic_device(None)?;
        
        // AetherOSカーネルに通知
        if let Some(handle) = self.native_handle {
            debug!("ハプティクス停止をAetherOSカーネルに通知: ハンドル={:x}", handle);
            
            // 実際の実装ではAetherOSカーネルAPIを呼び出す
            // モックアップでは何もしない
        }
        
        Ok(())
    }
    
    // 特定のデバイスを停止
    pub fn stop_device(&self, device_id: &str) -> Result<(), HapticError> {
        debug!("AetherOSハプティクスデバイスを停止中: {}", device_id);
        
        // 基本のハプティクス機能を使用してデバイスを停止
        stop_haptic_device(Some(device_id))?;
        
        // AetherOSカーネルに通知
        if let Some(handle) = self.native_handle {
            debug!("ハプティクス停止をAetherOSカーネルに通知: ハンドル={:x}, デバイス={}", 
                handle, device_id);
            
            // 実際の実装ではAetherOSカーネルAPIを呼び出す
            // モックアップでは何もしない
        }
        
        Ok(())
    }
    
    // シャットダウン
    pub fn shutdown(&mut self) -> Result<(), HapticError> {
        debug!("AetherOSハプティクスハンドラーをシャットダウン中...");
        
        // すべてのデバイスを停止
        if let Err(e) = self.stop_all_devices() {
            warn!("ハプティクスデバイス停止中にエラーが発生: {:?}", e);
            // エラーを抑制して続行
        }
        
        // AetherOSカーネル接続をクリーンアップ
        if let Some(handle) = self.native_handle.take() {
            debug!("AetherOSハプティクスネイティブハンドルを解放: {:x}", handle);
            
            // 実際の実装ではAetherOSカーネルAPIを呼び出す
            // モックアップでは何もしない
        }
        
        self.available_devices.clear();
        self.default_device_id = None;
        self.event_callback = None;
        
        Ok(())
    }
    
    // 利用可能なデバイスのリストを取得
    pub fn get_available_devices(&self) -> Vec<HapticDevice> {
        self.available_devices.clone()
    }
    
    // 現在の設定を取得
    pub fn get_settings(&self) -> AetherOSHapticsSettings {
        self.settings.clone()
    }
}

// グローバルハンドラーのシングルトン
lazy_static::lazy_static! {
    static ref AETHEROS_HAPTICS: Arc<Mutex<AetherOSHapticsHandler>> = Arc::new(Mutex::new(
        AetherOSHapticsHandler::new()
    ));
}

// グローバルハンドラーを初期化
pub fn initialize_aetheros_haptics() -> Result<(), HapticError> {
    let mut handler = AETHEROS_HAPTICS.lock().map_err(|_| {
        HapticError::Other("AetherOSハプティクスハンドラーロックの取得に失敗しました".to_string())
    })?;
    
    handler.initialize()
}

// グローバルハンドラーを取得
pub fn get_aetheros_haptics_handler() -> Result<Arc<Mutex<AetherOSHapticsHandler>>, HapticError> {
    Ok(AETHEROS_HAPTICS.clone())
}

// AetherOSハプティクスイベントを再生（グローバルハンドラー経由）
pub fn play_aetheros_haptic_event(event: HapticEvent) -> Result<HapticEventResult, HapticError> {
    let handler = AETHEROS_HAPTICS.lock().map_err(|_| {
        HapticError::Other("AetherOSハプティクスハンドラーロックの取得に失敗しました".to_string())
    })?;
    
    handler.play_event(event)
}

// AetherOSですべてのハプティクスデバイスを停止（グローバルハンドラー経由）
pub fn stop_all_aetheros_haptic_devices() -> Result<(), HapticError> {
    let handler = AETHEROS_HAPTICS.lock().map_err(|_| {
        HapticError::Other("AetherOSハプティクスハンドラーロックの取得に失敗しました".to_string())
    })?;
    
    handler.stop_all_devices()
}

// AetherOSで特定のハプティクスデバイスを停止（グローバルハンドラー経由）
pub fn stop_aetheros_haptic_device(device_id: &str) -> Result<(), HapticError> {
    let handler = AETHEROS_HAPTICS.lock().map_err(|_| {
        HapticError::Other("AetherOSハプティクスハンドラーロックの取得に失敗しました".to_string())
    })?;
    
    handler.stop_device(device_id)
}

// AetherOSハプティクス設定を更新（グローバルハンドラー経由）
pub fn update_aetheros_haptics_settings(settings: AetherOSHapticsSettings) -> Result<(), HapticError> {
    let mut handler = AETHEROS_HAPTICS.lock().map_err(|_| {
        HapticError::Other("AetherOSハプティクスハンドラーロックの取得に失敗しました".to_string())
    })?;
    
    handler.update_settings(settings);
    Ok(())
}

// AetherOSハプティクスイベントコールバックを設定（グローバルハンドラー経由）
pub fn set_aetheros_haptics_callback<F>(callback: F) -> Result<(), HapticError> 
where
    F: Fn(&HapticEventResult) + Send + Sync + 'static,
{
    let mut handler = AETHEROS_HAPTICS.lock().map_err(|_| {
        HapticError::Other("AetherOSハプティクスハンドラーロックの取得に失敗しました".to_string())
    })?;
    
    handler.set_event_callback(callback);
    Ok(())
}

// AetherOSハプティクスをシャットダウン（グローバルハンドラー経由）
pub fn shutdown_aetheros_haptics() -> Result<(), HapticError> {
    let mut handler = AETHEROS_HAPTICS.lock().map_err(|_| {
        HapticError::Other("AetherOSハプティクスハンドラーロックの取得に失敗しました".to_string())
    })?;
    
    handler.shutdown()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // モックハプティクスイベントを作成
    fn create_mock_event() -> HapticEvent {
        HapticEvent {
            pattern: HapticPattern::Click,
            intensity: Some(HapticIntensity::Medium),
            duration_ms: Some(50),
            target_device_id: None,
            category: Some("ui".to_string()),
        }
    }
    
    #[test]
    fn test_haptics_settings_default() {
        let settings = AetherOSHapticsSettings::default();
        
        assert!(settings.auto_detect_devices);
        assert!(settings.system_haptics_enabled);
        assert!(settings.ui_haptics_enabled);
        assert!(settings.app_haptics_enabled);
        assert!(settings.accessibility_haptics_enabled);
        assert!(matches!(settings.system_intensity, HapticIntensity::Medium));
    }
    
    #[test]
    fn test_settings_serialization() {
        let settings = AetherOSHapticsSettings::default();
        let json = serialize_settings(&settings);
        
        assert!(!json.is_empty());
        assert!(json.contains("system_haptics_enabled"));
        
        let deserialized = deserialize_settings(&json).unwrap();
        assert_eq!(deserialized.system_haptics_enabled, settings.system_haptics_enabled);
    }
    
    #[test]
    fn test_initialize_and_shutdown() {
        let mut handler = AetherOSHapticsHandler::new();
        
        // 初期化
        let init_result = handler.initialize();
        assert!(init_result.is_ok());
        
        // ネイティブハンドルが設定されていることを確認
        assert!(handler.native_handle.is_some());
        
        // シャットダウン
        let shutdown_result = handler.shutdown();
        assert!(shutdown_result.is_ok());
        
        // ネイティブハンドルがクリアされていることを確認
        assert!(handler.native_handle.is_none());
    }
    
    #[test]
    fn test_event_allowed_check() {
        let mut handler = AetherOSHapticsHandler::new();
        
        // デフォルトではすべてのイベントが許可される
        let event = create_mock_event();
        assert!(handler.is_event_allowed(&event));
        
        // UIハプティクスを無効にする
        let mut settings = AetherOSHapticsSettings::default();
        settings.ui_haptics_enabled = false;
        handler.update_settings(settings);
        
        // UIカテゴリのイベントが拒否されることを確認
        let ui_event = HapticEvent {
            category: Some("ui".to_string()),
            ..create_mock_event()
        };
        assert!(!handler.is_event_allowed(&ui_event));
        
        // システムカテゴリのイベントは許可される
        let system_event = HapticEvent {
            category: Some("system".to_string()),
            ..create_mock_event()
        };
        assert!(handler.is_event_allowed(&system_event));
        
        // システム全体のハプティクスを無効にする
        let mut settings = handler.settings.clone();
        settings.system_haptics_enabled = false;
        handler.update_settings(settings);
        
        // すべてのイベントが拒否される
        assert!(!handler.is_event_allowed(&system_event));
    }
    
    #[test]
    fn test_device_detection_and_default_device() {
        let mut handler = AetherOSHapticsHandler::new();
        handler.initialize().unwrap();
        
        // 再検出を強制
        let detect_result = handler.detect_devices();
        assert!(detect_result.is_ok());
        
        // 実際のテストでは、モックデバイスを追加して検証する
        // このテストでは簡易的な確認のみ
        
        // デフォルトデバイスの取得
        let default_id = handler.get_default_device_id();
        
        // 利用可能なデバイスがある場合、デフォルトデバイスも設定されているはず
        if !handler.available_devices.is_empty() {
            assert!(default_id.is_some());
        } else {
            assert!(default_id.is_none());
        }
    }
} 