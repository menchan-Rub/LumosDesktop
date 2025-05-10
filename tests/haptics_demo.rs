// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of AetherOS LumosDesktop.
//
// ハプティクスデモアプリケーション
// Copyright (c) 2023-2024 AetherOS Team.

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use lumos_desktop::core::system::haptics::{
    // 共通
    HapticPattern,
    HapticEvent,
    HapticDeviceType,
    
    // AetherOS統合
    AetherOSHapticsHandler,
    AetherOSHapticsSettings,
    initialize_aetheros_haptics,
    get_aetheros_haptics_handler,
    play_aetheros_haptic_event,
    stop_all_aetheros_haptic_devices,
    
    // ヘルパー関数
    play_haptic_feedback,
    stop_haptic_device,
    detect_all_haptic_devices,
    create_ui_haptic_event,
    create_system_haptic_event,
};

use log::{debug, info, warn, error, LevelFilter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ロギングの初期化
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
    
    info!("AetherOS LumosDesktop ハプティクスデモアプリケーション");
    println!("================================");
    println!("ハプティクスデモアプリケーション");
    println!("================================");
    println!();
    
    // ハプティクスシステムの初期化
    println!("ハプティクスシステムを初期化中...");
    
    // AetherOSハプティクスを初期化
    if let Err(e) = initialize_aetheros_haptics() {
        println!("AetherOSハプティクス初期化エラー: {:?}", e);
        // エラーがあっても続行
    } else {
        println!("AetherOSハプティクス初期化完了");
    }
    
    // 利用可能なデバイスを検出して表示
    println!("\n利用可能なハプティックデバイスの検出:");
    let devices = detect_all_haptic_devices()?;
    
    if devices.is_empty() {
        println!("  利用可能なハプティックデバイスが見つかりませんでした。");
        println!("  デモはシミュレーションモードで実行されます。");
    } else {
        println!("  {} デバイスが見つかりました:", devices.len());
        
        for (i, device) in devices.iter().enumerate() {
            let device_type = match device.device_type {
                HapticDeviceType::InternalMotor => "内蔵モーター",
                HapticDeviceType::Touchpad => "タッチパッド",
                HapticDeviceType::Touchscreen => "タッチスクリーン",
                HapticDeviceType::GameController => "ゲームコントローラー",
                _ => "その他",
            };
            
            println!("  {}: {} - {} ({})", i + 1, device.name, device_type, device.id);
            println!("     サポートされている効果: {} 種類", device.supported_effects.len());
        }
    }
    
    // メインメニュー
    loop {
        println!("\nハプティクスデモメニュー:");
        println!("  1. UI効果のテスト");
        println!("  2. システム効果のテスト");
        println!("  3. パターンのテスト");
        println!("  4. すべてのデバイスを停止");
        println!("  5. AetherOS機能のテスト");
        println!("  0. 終了");
        
        print!("\n選択 (0-5): ");
        io::stdout().flush().unwrap();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        
        match choice.trim() {
            "1" => test_ui_effects()?,
            "2" => test_system_effects()?,
            "3" => test_patterns()?,
            "4" => {
                stop_haptic_device(None)?;
                println!("すべてのデバイスを停止しました。");
            },
            "5" => test_aetheros_features()?,
            "0" => break,
            _ => println!("無効な選択です。もう一度試してください。"),
        }
    }
    
    // クリーンアップ
    println!("\nハプティクスシステムをシャットダウン中...");
    stop_all_aetheros_haptic_devices()?;
    
    println!("終了");
    Ok(())
}

// UI効果のテスト
fn test_ui_effects() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nUI効果テスト:");
    println!("  1. クリック");
    println!("  2. ダブルクリック");
    println!("  3. ドラッグ開始");
    println!("  4. ドロップ完了");
    println!("  5. ボタン押下");
    println!("  6. スクロール");
    println!("  0. 戻る");
    
    print!("\n選択 (0-6): ");
    io::stdout().flush().unwrap();
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    
    match choice.trim() {
        "1" => {
            println!("クリック効果を再生中...");
            let event = create_ui_haptic_event(HapticPattern::Click);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "2" => {
            println!("ダブルクリック効果を再生中...");
            let event = create_ui_haptic_event(HapticPattern::DoubleClick);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "3" => {
            println!("ドラッグ開始効果を再生中...");
            let event = create_ui_haptic_event(HapticPattern::Pulse);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "4" => {
            println!("ドロップ完了効果を再生中...");
            let event = create_ui_haptic_event(HapticPattern::Success);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "5" => {
            println!("ボタン押下効果を再生中...");
            let event = create_ui_haptic_event(HapticPattern::Click)
                .with_intensity(70);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "6" => {
            println!("スクロール効果を再生中...");
            // 複数の小さなパルスを再生して、スクロール効果をシミュレート
            for _ in 0..5 {
                let event = create_ui_haptic_event(HapticPattern::Tap)
                    .with_intensity(20)
                    .with_duration(10);
                play_haptic_feedback(&event)?;
                thread::sleep(Duration::from_millis(100));
            }
            println!("完了");
        },
        "0" => return Ok(()),
        _ => println!("無効な選択です。"),
    }
    
    Ok(())
}

// システム効果のテスト
fn test_system_effects() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nシステム効果テスト:");
    println!("  1. 通知");
    println!("  2. 警告");
    println!("  3. エラー");
    println!("  4. 成功");
    println!("  5. システム起動");
    println!("  6. システムシャットダウン");
    println!("  0. 戻る");
    
    print!("\n選択 (0-6): ");
    io::stdout().flush().unwrap();
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    
    match choice.trim() {
        "1" => {
            println!("通知効果を再生中...");
            let event = create_system_haptic_event(HapticPattern::Notification);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "2" => {
            println!("警告効果を再生中...");
            let event = create_system_haptic_event(HapticPattern::Warning);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "3" => {
            println!("エラー効果を再生中...");
            let event = create_system_haptic_event(HapticPattern::Error);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "4" => {
            println!("成功効果を再生中...");
            let event = create_system_haptic_event(HapticPattern::Success);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "5" => {
            println!("システム起動効果を再生中...");
            let event = create_system_haptic_event(HapticPattern::RumblePulse)
                .with_intensity(80)
                .with_duration(1000);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "6" => {
            println!("システムシャットダウン効果を再生中...");
            let event = create_system_haptic_event(HapticPattern::RumbleContinuous)
                .with_intensity(60)
                .with_duration(800);
            let result = play_haptic_feedback(&event)?;
            println!("結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "0" => return Ok(()),
        _ => println!("無効な選択です。"),
    }
    
    Ok(())
}

// パターンのテスト
fn test_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nパターンテスト:");
    println!("  1. Click - クリック感");
    println!("  2. DoubleClick - ダブルクリック感");
    println!("  3. TripleClick - トリプルクリック感");
    println!("  4. Tap - 軽いタップ感");
    println!("  5. Pulse - 短いパルス");
    println!("  6. Buzz - ブザー音のような振動");
    println!("  7. Vibration - 一般的な振動");
    println!("  8. LongPress - 長押し感");
    println!("  9. RumbleContinuous - 連続的な振動");
    println!(" 10. RumblePulse - パルス状の振動");
    println!("  0. 戻る");
    
    print!("\n選択 (0-10): ");
    io::stdout().flush().unwrap();
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    
    let pattern = match choice.trim() {
        "1" => HapticPattern::Click,
        "2" => HapticPattern::DoubleClick,
        "3" => HapticPattern::TripleClick,
        "4" => HapticPattern::Tap,
        "5" => HapticPattern::Pulse,
        "6" => HapticPattern::Buzz,
        "7" => HapticPattern::Vibration,
        "8" => HapticPattern::LongPress,
        "9" => HapticPattern::RumbleContinuous,
        "10" => HapticPattern::RumblePulse,
        "0" => return Ok(()),
        _ => {
            println!("無効な選択です。");
            return Ok(());
        }
    };
    
    // 強度を選択
    println!("\n強度を選択:");
    println!("  1. 弱 (25%)");
    println!("  2. 中 (50%)");
    println!("  3. 強 (75%)");
    println!("  4. 最強 (100%)");
    
    print!("\n選択 (1-4): ");
    io::stdout().flush().unwrap();
    
    let mut intensity_choice = String::new();
    io::stdin().read_line(&mut intensity_choice).unwrap();
    
    let intensity = match intensity_choice.trim() {
        "1" => 25,
        "2" => 50,
        "3" => 75,
        "4" => 100,
        _ => 50, // デフォルト
    };
    
    // イベントを作成して再生
    println!("\n{:?} パターンを強度 {}% で再生中...", pattern, intensity);
    
    let event = HapticEvent {
        pattern,
        intensity: Some(intensity),
        duration_ms: None, // デフォルト
        target_device_id: None, // デフォルト
        category: None,
    };
    
    let result = play_haptic_feedback(&event)?;
    println!("結果: {}", if result.success { "成功" } else { "失敗" });
    
    Ok(())
}

// AetherOS機能のテスト
fn test_aetheros_features() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nAetherOS機能テスト:");
    println!("  1. 利用可能なデバイスの表示");
    println!("  2. ハンドラー情報の表示");
    println!("  3. 設定の変更");
    println!("  4. カスタムイベントの再生");
    println!("  0. 戻る");
    
    print!("\n選択 (0-4): ");
    io::stdout().flush().unwrap();
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    
    match choice.trim() {
        "1" => {
            println!("\nAetherOSハプティクスデバイス:");
            
            let handler = get_aetheros_haptics_handler()?;
            let handler = handler.lock().unwrap();
            let devices = handler.get_available_devices();
            
            if devices.is_empty() {
                println!("  利用可能なデバイスが見つかりませんでした。");
            } else {
                for (i, device) in devices.iter().enumerate() {
                    let device_type = match device.device_type {
                        HapticDeviceType::InternalMotor => "内蔵モーター",
                        HapticDeviceType::Touchpad => "タッチパッド",
                        HapticDeviceType::Touchscreen => "タッチスクリーン",
                        HapticDeviceType::GameController => "ゲームコントローラー",
                        _ => "その他",
                    };
                    
                    println!("  {}: {} - {} ({})", i + 1, device.name, device_type, device.id);
                    println!("     サポートされている効果: {} 種類", device.supported_effects.len());
                }
                
                if let Some(default_id) = handler.get_default_device_id() {
                    println!("\n  デフォルトデバイス: {}", default_id);
                }
            }
        },
        "2" => {
            println!("\nAetherOSハプティクスハンドラー情報:");
            
            let handler = get_aetheros_haptics_handler()?;
            let handler = handler.lock().unwrap();
            let settings = handler.get_settings();
            
            println!("  自動デバイス検出: {}", if settings.auto_detect_devices { "有効" } else { "無効" });
            println!("  システムハプティクス: {}", if settings.system_haptics_enabled { "有効" } else { "無効" });
            println!("  UIハプティクス: {}", if settings.ui_haptics_enabled { "有効" } else { "無効" });
            println!("  アプリハプティクス: {}", if settings.app_haptics_enabled { "有効" } else { "無効" });
            println!("  アクセシビリティハプティクス: {}", if settings.accessibility_haptics_enabled { "有効" } else { "無効" });
            println!("  システム強度: {}%", settings.system_intensity);
        },
        "3" => {
            println!("\n設定の変更:");
            
            let handler = get_aetheros_haptics_handler()?;
            let mut settings = handler.lock().unwrap().get_settings();
            
            // UIハプティクスの切り替え
            settings.ui_haptics_enabled = !settings.ui_haptics_enabled;
            println!("  UIハプティクスを {} に設定しました", if settings.ui_haptics_enabled { "有効" } else { "無効" });
            
            // 強度の変更
            settings.system_intensity = if settings.system_intensity < 50 { 75 } else { 25 };
            println!("  システム強度を {}% に設定しました", settings.system_intensity);
            
            // 設定を適用
            drop(handler);
            update_aetheros_haptics_settings(settings)?;
            println!("  設定が更新されました");
            
            // テストイベントを再生
            println!("  テストイベントを再生中...");
            let event = create_ui_haptic_event(HapticPattern::Click);
            let result = play_aetheros_haptic_event(event)?;
            println!("  結果: {}", if result.success { "成功" } else { "失敗" });
        },
        "4" => {
            println!("\nカスタムイベントの再生:");
            
            // カスタムパターンの選択
            println!("  パターンを選択:");
            println!("    1. 起動シーケンス");
            println!("    2. アラート");
            println!("    3. 成功フィードバック");
            
            print!("\n  選択 (1-3): ");
            io::stdout().flush().unwrap();
            
            let mut pattern_choice = String::new();
            io::stdin().read_line(&mut pattern_choice).unwrap();
            
            match pattern_choice.trim() {
                "1" => {
                    println!("  起動シーケンスを再生中...");
                    
                    // 段階的に強度を上げるパルスパターン
                    for i in 1..=5 {
                        let intensity = (i as u8) * 20;
                        let event = HapticEvent::new(HapticPattern::Pulse)
                            .with_intensity(intensity)
                            .with_duration(100)
                            .with_category("system".to_string());
                        
                        play_aetheros_haptic_event(event)?;
                        thread::sleep(Duration::from_millis(150));
                    }
                    
                    // 最後に成功通知
                    let event = HapticEvent::new(HapticPattern::Success)
                        .with_intensity(80)
                        .with_duration(300)
                        .with_category("system".to_string());
                    
                    play_aetheros_haptic_event(event)?;
                },
                "2" => {
                    println!("  アラートを再生中...");
                    
                    // 警告的なパターン（短い休止を挟む3つのバースト）
                    for _ in 0..3 {
                        let event = HapticEvent::new(HapticPattern::Warning)
                            .with_intensity(80)
                            .with_duration(150)
                            .with_category("system".to_string());
                        
                        play_aetheros_haptic_event(event)?;
                        thread::sleep(Duration::from_millis(200));
                    }
                },
                "3" => {
                    println!("  成功フィードバックを再生中...");
                    
                    // 短いクリックに続く成功通知
                    let click_event = HapticEvent::new(HapticPattern::Click)
                        .with_intensity(50)
                        .with_duration(30)
                        .with_category("ui".to_string());
                    
                    play_aetheros_haptic_event(click_event)?;
                    thread::sleep(Duration::from_millis(100));
                    
                    let success_event = HapticEvent::new(HapticPattern::Success)
                        .with_intensity(60)
                        .with_duration(200)
                        .with_category("system".to_string());
                    
                    play_aetheros_haptic_event(success_event)?;
                },
                _ => println!("  無効な選択です。"),
            }
        },
        "0" => return Ok(()),
        _ => println!("無効な選択です。"),
    }
    
    Ok(())
} 