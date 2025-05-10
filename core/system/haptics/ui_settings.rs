// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
// 
// AetherOS LumosDesktop ハプティクス設定UIコンポーネント
// Copyright (c) 2023-2024 AetherOS Team.

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;
use log::{debug, error, info};

use crate::ui::components::{
    Button, Checkbox, ComboBox, Label, Slider, 
    Panel, VStack, HStack, Spacer,
    UIComponent, UIEvent, UIEventType
};
use crate::ui::themes::Theme;
use crate::system::haptics::{
    HapticFeedback, HapticIntensity, HapticPattern,
    HapticDevice, HapticDeviceType
};

/// ハプティクスシステム設定パネル
pub struct HapticSettingsPanel {
    panel: Panel,
    haptic_feedback: Arc<Mutex<HapticFeedback>>,
    devices: Vec<HapticDevice>,
    enabled_checkbox: Checkbox,
    intensity_combo: ComboBox,
    selected_device_combo: ComboBox,
    test_buttons: Vec<Button>,
}

impl HapticSettingsPanel {
    /// 新しいハプティクス設定パネルを作成
    pub fn new(haptic_feedback: Arc<Mutex<HapticFeedback>>, theme: &Theme) -> Self {
        let mut panel = Panel::new(theme);
        panel.set_title("ハプティック設定");
        panel.set_size(400, 500);
        
        // 現在の設定を取得
        let enabled = haptic_feedback.lock().unwrap().is_enabled();
        let intensity = haptic_feedback.lock().unwrap().get_default_intensity();
        let devices = match haptic_feedback.lock().unwrap().get_devices() {
            Ok(devices) => devices,
            Err(_) => Vec::new(),
        };
        
        // コンポーネント作成
        let enabled_checkbox = Checkbox::new("ハプティックフィードバックを有効にする", enabled, theme);
        
        let mut intensity_combo = ComboBox::new(theme);
        intensity_combo.add_item("なし", HapticIntensity::None as i32);
        intensity_combo.add_item("非常に弱い", HapticIntensity::VeryLight as i32);
        intensity_combo.add_item("弱い", HapticIntensity::Light as i32);
        intensity_combo.add_item("中程度", HapticIntensity::Medium as i32);
        intensity_combo.add_item("強い", HapticIntensity::Strong as i32);
        intensity_combo.add_item("非常に強い", HapticIntensity::VeryStrong as i32);
        intensity_combo.set_selected_value(intensity as i32);
        
        let mut selected_device_combo = ComboBox::new(theme);
        selected_device_combo.add_item("すべてのデバイス", "all");
        for device in &devices {
            selected_device_combo.add_item(&device.name, &device.id);
        }
        selected_device_combo.set_selected_index(0);
        
        // テストボタン
        let mut test_buttons = Vec::new();
        
        let patterns = [
            ("クリック", HapticPattern::Click),
            ("ダブルクリック", HapticPattern::DoubleClick),
            ("長押し", HapticPattern::LongPress),
            ("成功", HapticPattern::Success),
            ("エラー", HapticPattern::Error),
            ("警告", HapticPattern::Warning),
        ];
        
        for (name, pattern) in &patterns {
            let mut button = Button::new(format!("テスト: {}", name), theme);
            button.set_tag(format!("test_{:?}", pattern));
            test_buttons.push(button);
        }
        
        // レイアウト構築
        let mut root_layout = VStack::new(theme);
        
        // 有効化設定
        root_layout.add_child(Box::new(enabled_checkbox.clone()));
        root_layout.add_child(Box::new(Spacer::new(0, 10)));
        
        // 強度設定
        let mut intensity_layout = HStack::new(theme);
        intensity_layout.add_child(Box::new(Label::new("デフォルト強度:", theme)));
        intensity_layout.add_child(Box::new(Spacer::new(10, 0)));
        intensity_layout.add_child(Box::new(intensity_combo.clone()));
        root_layout.add_child(Box::new(intensity_layout));
        root_layout.add_child(Box::new(Spacer::new(0, 20)));
        
        // デバイス設定
        let mut device_layout = VStack::new(theme);
        device_layout.add_child(Box::new(Label::new("デバイス設定", theme)));
        device_layout.add_child(Box::new(Spacer::new(0, 10)));
        device_layout.add_child(Box::new(selected_device_combo.clone()));
        
        // デバイスリスト
        let mut device_list = VStack::new(theme);
        for device in &devices {
            let device_info = format!("{} ({})", device.name, 
                match device.device_type {
                    HapticDeviceType::InternalMotor => "内蔵モーター",
                    HapticDeviceType::Touchpad => "タッチパッド",
                    HapticDeviceType::Touchscreen => "タッチスクリーン",
                    HapticDeviceType::ExternalController => "外部コントローラー",
                    HapticDeviceType::Generic => "汎用デバイス",
                }
            );
            device_list.add_child(Box::new(Label::new(&device_info, theme)));
        }
        device_layout.add_child(Box::new(device_list));
        root_layout.add_child(Box::new(device_layout));
        root_layout.add_child(Box::new(Spacer::new(0, 20)));
        
        // テストセクション
        root_layout.add_child(Box::new(Label::new("フィードバックテスト", theme)));
        root_layout.add_child(Box::new(Spacer::new(0, 10)));
        
        let mut test_grid = VStack::new(theme);
        for i in (0..test_buttons.len()).step_by(2) {
            let mut row = HStack::new(theme);
            row.add_child(Box::new(test_buttons[i].clone()));
            if i + 1 < test_buttons.len() {
                row.add_child(Box::new(Spacer::new(10, 0)));
                row.add_child(Box::new(test_buttons[i + 1].clone()));
            }
            test_grid.add_child(Box::new(row));
            test_grid.add_child(Box::new(Spacer::new(0, 5)));
        }
        root_layout.add_child(Box::new(test_grid));
        
        panel.set_content(Box::new(root_layout));
        
        Self {
            panel,
            haptic_feedback,
            devices,
            enabled_checkbox,
            intensity_combo,
            selected_device_combo,
            test_buttons,
        }
    }
    
    /// イベントハンドラを登録
    pub fn register_event_handlers(&mut self) {
        let haptic_feedback = Arc::clone(&self.haptic_feedback);
        
        // 有効化チェックボックス
        self.enabled_checkbox.on_change(Box::new(move |checked| {
            if let Ok(mut feedback) = haptic_feedback.lock() {
                feedback.set_enabled(checked);
                debug!("ハプティックフィードバック: {}", if checked { "有効" } else { "無効" });
                
                // 設定が変更されたことを示すハプティックを再生
                if checked {
                    let _ = feedback.play_pattern(HapticPattern::Success);
                }
            }
        }));
        
        // 強度コンボボックス
        let haptic_feedback = Arc::clone(&self.haptic_feedback);
        self.intensity_combo.on_change(Box::new(move |value| {
            if let Ok(intensity) = num::FromPrimitive::from_i32(value) {
                if let Ok(mut feedback) = haptic_feedback.lock() {
                    feedback.set_default_intensity(intensity);
                    debug!("ハプティック強度: {:?}", intensity);
                    
                    // 新しい強度でテストハプティックを再生
                    let _ = feedback.play_pattern(HapticPattern::Click);
                }
            }
        }));
        
        // テストボタン
        for button in &mut self.test_buttons {
            let haptic_feedback = Arc::clone(&self.haptic_feedback);
            let selected_device_combo = self.selected_device_combo.clone();
            let tag = button.get_tag().clone();
            
            button.on_click(Box::new(move || {
                if let Ok(mut feedback) = haptic_feedback.lock() {
                    let pattern = match tag.as_str() {
                        "test_Click" => HapticPattern::Click,
                        "test_DoubleClick" => HapticPattern::DoubleClick,
                        "test_LongPress" => HapticPattern::LongPress,
                        "test_Success" => HapticPattern::Success,
                        "test_Error" => HapticPattern::Error,
                        "test_Warning" => HapticPattern::Warning,
                        _ => HapticPattern::Click,
                    };
                    
                    // デバイス選択
                    let selected_value = selected_device_combo.get_selected_value();
                    if selected_value == "all" {
                        let _ = feedback.play_pattern(pattern);
                    } else {
                        let mut event = feedback.create_event(pattern);
                        event.set_target_device_id(&selected_value);
                        let _ = feedback.play_event(event);
                    }
                    
                    debug!("テストハプティック再生: {:?}", pattern);
                }
            }));
        }
    }
    
    /// パネルを更新（デバイスリストなど）
    pub fn update(&mut self) {
        // デバイスリストを更新
        self.devices = match self.haptic_feedback.lock().unwrap().get_devices() {
            Ok(devices) => devices,
            Err(_) => Vec::new(),
        };
        
        // デバイス選択コンボボックスを更新
        self.selected_device_combo.clear();
        self.selected_device_combo.add_item("すべてのデバイス", "all");
        for device in &self.devices {
            self.selected_device_combo.add_item(&device.name, &device.id);
        }
        self.selected_device_combo.set_selected_index(0);
    }
    
    /// パネルを取得
    pub fn get_panel(&self) -> &Panel {
        &self.panel
    }
}

/// システム設定メニューへの統合ポイント
pub fn register_haptic_settings(system_settings: &mut crate::ui::settings::SystemSettings) {
    if let Some(haptic_feedback) = system_settings.get_subsystem::<HapticFeedback>("haptics") {
        let haptic_feedback = Arc::new(Mutex::new(haptic_feedback.clone()));
        let mut settings_panel = HapticSettingsPanel::new(haptic_feedback, system_settings.get_theme());
        settings_panel.register_event_handlers();
        
        system_settings.add_settings_panel("ハプティック", Box::new(settings_panel.get_panel().clone()));
        
        info!("ハプティック設定パネルが登録されました");
    } else {
        error!("ハプティックサブシステムが見つかりませんでした");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::themes::default_theme;
    
    #[test]
    fn test_haptic_settings_panel_creation() {
        let haptic_feedback = Arc::new(Mutex::new(HapticFeedback::new_default()));
        let theme = default_theme();
        
        let panel = HapticSettingsPanel::new(haptic_feedback, &theme);
        assert!(panel.get_panel().get_title() == "ハプティック設定");
    }
    
    #[test]
    fn test_haptic_settings_event_handlers() {
        let haptic_feedback = Arc::new(Mutex::new(HapticFeedback::new_default()));
        let theme = default_theme();
        
        let mut panel = HapticSettingsPanel::new(haptic_feedback.clone(), &theme);
        panel.register_event_handlers();
        
        // 有効化設定のテスト
        panel.enabled_checkbox.set_checked(true);
        panel.enabled_checkbox.trigger_change();
        assert!(haptic_feedback.lock().unwrap().is_enabled());
        
        panel.enabled_checkbox.set_checked(false);
        panel.enabled_checkbox.trigger_change();
        assert!(!haptic_feedback.lock().unwrap().is_enabled());
        
        // 強度設定のテスト
        panel.intensity_combo.set_selected_value(HapticIntensity::Strong as i32);
        panel.intensity_combo.trigger_change();
        assert_eq!(haptic_feedback.lock().unwrap().get_default_intensity(), HapticIntensity::Strong);
    }
} 