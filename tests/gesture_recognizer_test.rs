// LumosDesktop ジェスチャー認識器テスト
// 実装した長押しとピンチの認識器が正しく動作するかテストするためのプログラム

use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::thread;

// プロジェクトのクレートを使用
use lumos_desktop::core::window_manager::gesture_recognizer::{
    GestureManager, GestureType, GestureState, GestureInfo
};
use lumos_desktop::core::window_manager::input_translator::{
    InputEvent, InputEventType, MouseButton, KeyModifier
};

fn main() {
    println!("LumosDesktop ジェスチャー認識器テスト開始");
    
    // ジェスチャーマネージャーを作成し、デフォルト認識器を登録
    let mut gesture_manager = GestureManager::new();
    gesture_manager.register_default_recognizers();
    
    // コールバックを登録
    gesture_manager.add_gesture_callback(|gesture| {
        println!("ジェスチャー検出: {:?}, 状態: {:?}", gesture.gesture_type, gesture.state);
        match gesture.gesture_type {
            GestureType::LongPress => {
                println!("  長押し: 位置 = ({:.1}, {:.1})", 
                    gesture.position.0, gesture.position.1);
                if let Some(duration) = gesture.long_press_duration {
                    println!("  長押し時間: {}ms", duration.as_millis());
                }
            },
            GestureType::Pinch => {
                println!("  ピンチ: 中心位置 = ({:.1}, {:.1}), スケール = {:.2}", 
                    gesture.position.0, gesture.position.1, gesture.scale);
            },
            GestureType::Rotate => {
                println!("  回転: 中心位置 = ({:.1}, {:.1}), 角度 = {:.2}ラジアン（約{:.1}度）", 
                    gesture.position.0, gesture.position.1, gesture.rotation, gesture.rotation * 180.0 / std::f64::consts::PI);
            },
            _ => {}
        }
        true
    });
    
    println!("\n=== 長押し認識器テスト ===");
    test_long_press(&mut gesture_manager);
    
    println!("\n=== ピンチ認識器テスト ===");
    test_pinch(&mut gesture_manager);
    
    println!("\n=== 回転認識器テスト ===");
    test_rotate(&mut gesture_manager);
    
    println!("\nLumosDesktop ジェスチャー認識器テスト完了");
}

/// 長押し認識器のテスト
fn test_long_press(gesture_manager: &mut GestureManager) {
    // テスト1: 正常な長押し
    println!("テスト1: 正常な長押し");
    
    // マウスプレスイベントを生成
    let press_event = InputEvent::new(InputEventType::MousePress {
        button: MouseButton::Left,
        x: 100.0,
        y: 100.0,
        modifiers: HashSet::new(),
        timestamp: 0,
    });
    
    // イベントを処理
    gesture_manager.process_event(&press_event);
    
    // 長押し時間を待機（500ms以上）
    let wait_time = Duration::from_millis(600);
    thread::sleep(wait_time);
    
    // マウス移動イベントを生成（わずかに動かす）
    let move_event = InputEvent::new(InputEventType::MouseMove {
        x: 102.0,
        y: 101.0,
        dx: 2.0,
        dy: 1.0,
        modifiers: HashSet::new(),
        timestamp: 600,
    });
    
    // イベントを処理
    let gestures = gesture_manager.process_event(&move_event);
    println!("  検出されたジェスチャー数: {}", gestures.len());
    
    // マウスリリースイベントを生成
    let release_event = InputEvent::new(InputEventType::MouseRelease {
        button: MouseButton::Left,
        x: 102.0,
        y: 101.0,
        modifiers: HashSet::new(),
        timestamp: 700,
    });
    
    // イベントを処理
    gesture_manager.process_event(&release_event);
    
    // テスト2: キャンセルされる長押し（移動しすぎ）
    println!("\nテスト2: キャンセルされる長押し（移動しすぎ）");
    
    // 状態をリセット
    gesture_manager.reset_all();
    
    // マウスプレスイベントを生成
    let press_event = InputEvent::new(InputEventType::MousePress {
        button: MouseButton::Left,
        x: 200.0,
        y: 200.0,
        modifiers: HashSet::new(),
        timestamp: 1000,
    });
    
    // イベントを処理
    gesture_manager.process_event(&press_event);
    
    // 長押し時間を待機（500ms以上）
    thread::sleep(wait_time);
    
    // マウス移動イベントを生成（大きく動かす - キャンセルされるはず）
    let move_event = InputEvent::new(InputEventType::MouseMove {
        x: 230.0,
        y: 220.0,
        dx: 30.0,
        dy: 20.0,
        modifiers: HashSet::new(),
        timestamp: 1600,
    });
    
    // イベントを処理
    let gestures = gesture_manager.process_event(&move_event);
    println!("  検出されたジェスチャー数: {}", gestures.len());
    
    // マウスリリースイベントを生成
    let release_event = InputEvent::new(InputEventType::MouseRelease {
        button: MouseButton::Left,
        x: 230.0,
        y: 220.0,
        modifiers: HashSet::new(),
        timestamp: 1700,
    });
    
    // イベントを処理
    gesture_manager.process_event(&release_event);
}

/// ピンチ認識器のテスト
fn test_pinch(gesture_manager: &mut GestureManager) {
    // テスト1: ピンチアウト
    println!("テスト1: ピンチアウト");
    
    // 状態をリセット
    gesture_manager.reset_all();
    
    // 最初のタッチ開始イベント
    let touch1_begin = InputEvent::new(InputEventType::TouchBegin {
        id: 1,
        x: 100.0,
        y: 100.0,
        pressure: 1.0,
        timestamp: 2000,
    });
    
    // 2番目のタッチ開始イベント
    let touch2_begin = InputEvent::new(InputEventType::TouchBegin {
        id: 2,
        x: 120.0,
        y: 100.0,
        pressure: 1.0,
        timestamp: 2020,
    });
    
    // イベントを処理
    gesture_manager.process_event(&touch1_begin);
    gesture_manager.process_event(&touch2_begin);
    
    // 指を広げる一連の動き
    for i in 1..=5 {
        let dist = 5.0 * i as f64;
        
        // 第1タッチポイントを左に移動
        let touch1_update = InputEvent::new(InputEventType::TouchUpdate {
            id: 1,
            x: 100.0 - dist,
            y: 100.0,
            dx: -5.0,
            dy: 0.0,
            pressure: 1.0,
            timestamp: 2050 + (i as u64 * 30),
        });
        
        // 第2タッチポイントを右に移動
        let touch2_update = InputEvent::new(InputEventType::TouchUpdate {
            id: 2,
            x: 120.0 + dist,
            y: 100.0,
            dx: 5.0,
            dy: 0.0,
            pressure: 1.0,
            timestamp: 2050 + (i as u64 * 30) + 10,
        });
        
        // イベントを処理
        let gestures1 = gesture_manager.process_event(&touch1_update);
        let gestures2 = gesture_manager.process_event(&touch2_update);
        
        println!("  ステップ {}: 検出されたジェスチャー数: {} + {}", 
            i, gestures1.len(), gestures2.len());
    }
    
    // タッチ終了イベント
    let touch1_end = InputEvent::new(InputEventType::TouchEnd {
        id: 1,
        x: 75.0,
        y: 100.0,
        timestamp: 2250,
    });
    
    let touch2_end = InputEvent::new(InputEventType::TouchEnd {
        id: 2,
        x: 145.0,
        y: 100.0,
        timestamp: 2260,
    });
    
    // イベントを処理
    gesture_manager.process_event(&touch1_end);
    gesture_manager.process_event(&touch2_end);
    
    // テスト2: ピンチイン
    println!("\nテスト2: ピンチイン");
    
    // 状態をリセット
    gesture_manager.reset_all();
    
    // 最初のタッチ開始イベント（最初から離れた位置）
    let touch1_begin = InputEvent::new(InputEventType::TouchBegin {
        id: 1,
        x: 50.0,
        y: 200.0,
        pressure: 1.0,
        timestamp: 3000,
    });
    
    // 2番目のタッチ開始イベント
    let touch2_begin = InputEvent::new(InputEventType::TouchBegin {
        id: 2,
        x: 150.0,
        y: 200.0,
        pressure: 1.0,
        timestamp: 3020,
    });
    
    // イベントを処理
    gesture_manager.process_event(&touch1_begin);
    gesture_manager.process_event(&touch2_begin);
    
    // 指を狭める一連の動き
    for i in 1..=5 {
        let dist = 10.0 * i as f64;
        
        // 第1タッチポイントを右に移動
        let touch1_update = InputEvent::new(InputEventType::TouchUpdate {
            id: 1,
            x: 50.0 + dist,
            y: 200.0,
            dx: 10.0,
            dy: 0.0,
            pressure: 1.0,
            timestamp: 3050 + (i as u64 * 30),
        });
        
        // 第2タッチポイントを左に移動
        let touch2_update = InputEvent::new(InputEventType::TouchUpdate {
            id: 2,
            x: 150.0 - dist,
            y: 200.0,
            dx: -10.0,
            dy: 0.0,
            pressure: 1.0,
            timestamp: 3050 + (i as u64 * 30) + 10,
        });
        
        // イベントを処理
        let gestures1 = gesture_manager.process_event(&touch1_update);
        let gestures2 = gesture_manager.process_event(&touch2_update);
        
        println!("  ステップ {}: 検出されたジェスチャー数: {} + {}", 
            i, gestures1.len(), gestures2.len());
    }
    
    // タッチ終了イベント
    let touch1_end = InputEvent::new(InputEventType::TouchEnd {
        id: 1,
        x: 100.0,
        y: 200.0,
        timestamp: 3250,
    });
    
    let touch2_end = InputEvent::new(InputEventType::TouchEnd {
        id: 2,
        x: 100.0,
        y: 200.0,
        timestamp: 3260,
    });
    
    // イベントを処理
    gesture_manager.process_event(&touch1_end);
    gesture_manager.process_event(&touch2_end);
}

/// 回転認識器のテスト
fn test_rotate(gesture_manager: &mut GestureManager) {
    // テスト1: 時計回り回転
    println!("テスト1: 時計回り回転");
    
    // 状態をリセット
    gesture_manager.reset_all();
    
    // 最初のタッチ開始イベント
    let touch1_begin = InputEvent::new(InputEventType::TouchBegin {
        id: 1,
        x: 100.0,
        y: 100.0,
        pressure: 1.0,
        timestamp: 4000,
    });
    
    // 2番目のタッチ開始イベント
    let touch2_begin = InputEvent::new(InputEventType::TouchBegin {
        id: 2,
        x: 200.0,
        y: 100.0,
        pressure: 1.0,
        timestamp: 4020,
    });
    
    // イベントを処理
    gesture_manager.process_event(&touch1_begin);
    gesture_manager.process_event(&touch2_begin);
    
    // 時計回りに回転する一連の動き
    for i in 1..=5 {
        let angle = i as f64 * 0.2; // 約11度ずつ回転
        
        // 第1タッチポイントを時計回りに移動
        let touch1_update = InputEvent::new(InputEventType::TouchUpdate {
            id: 1,
            x: 100.0,
            y: 100.0 + 20.0 * angle,
            dx: 0.0,
            dy: 20.0 * angle,
            pressure: 1.0,
            timestamp: 4050 + (i as u64 * 30),
        });
        
        // 第2タッチポイントを時計回りに移動
        let touch2_update = InputEvent::new(InputEventType::TouchUpdate {
            id: 2,
            x: 200.0,
            y: 100.0 - 20.0 * angle,
            dx: 0.0,
            dy: -20.0 * angle,
            pressure: 1.0,
            timestamp: 4050 + (i as u64 * 30) + 10,
        });
        
        // イベントを処理
        let gestures1 = gesture_manager.process_event(&touch1_update);
        let gestures2 = gesture_manager.process_event(&touch2_update);
        
        println!("  ステップ {}: 検出されたジェスチャー数: {} + {}", 
            i, gestures1.len(), gestures2.len());
    }
    
    // タッチ終了イベント
    let touch1_end = InputEvent::new(InputEventType::TouchEnd {
        id: 1,
        x: 100.0,
        y: 120.0,
        timestamp: 4250,
    });
    
    let touch2_end = InputEvent::new(InputEventType::TouchEnd {
        id: 2,
        x: 200.0,
        y: 80.0,
        timestamp: 4260,
    });
    
    // イベントを処理
    gesture_manager.process_event(&touch1_end);
    gesture_manager.process_event(&touch2_end);
    
    // テスト2: 反時計回り回転
    println!("\nテスト2: 反時計回り回転");
    
    // 状態をリセット
    gesture_manager.reset_all();
    
    // 最初のタッチ開始イベント
    let touch1_begin = InputEvent::new(InputEventType::TouchBegin {
        id: 1,
        x: 100.0,
        y: 300.0,
        pressure: 1.0,
        timestamp: 5000,
    });
    
    // 2番目のタッチ開始イベント
    let touch2_begin = InputEvent::new(InputEventType::TouchBegin {
        id: 2,
        x: 200.0,
        y: 300.0,
        pressure: 1.0,
        timestamp: 5020,
    });
    
    // イベントを処理
    gesture_manager.process_event(&touch1_begin);
    gesture_manager.process_event(&touch2_begin);
    
    // 反時計回りに回転する一連の動き
    for i in 1..=5 {
        let angle = i as f64 * 0.2; // 約11度ずつ回転
        
        // 第1タッチポイントを反時計回りに移動
        let touch1_update = InputEvent::new(InputEventType::TouchUpdate {
            id: 1,
            x: 100.0,
            y: 300.0 - 20.0 * angle,
            dx: 0.0,
            dy: -20.0 * angle,
            pressure: 1.0,
            timestamp: 5050 + (i as u64 * 30),
        });
        
        // 第2タッチポイントを反時計回りに移動
        let touch2_update = InputEvent::new(InputEventType::TouchUpdate {
            id: 2,
            x: 200.0,
            y: 300.0 + 20.0 * angle,
            dx: 0.0,
            dy: 20.0 * angle,
            pressure: 1.0,
            timestamp: 5050 + (i as u64 * 30) + 10,
        });
        
        // イベントを処理
        let gestures1 = gesture_manager.process_event(&touch1_update);
        let gestures2 = gesture_manager.process_event(&touch2_update);
        
        println!("  ステップ {}: 検出されたジェスチャー数: {} + {}", 
            i, gestures1.len(), gestures2.len());
    }
    
    // タッチ終了イベント
    let touch1_end = InputEvent::new(InputEventType::TouchEnd {
        id: 1,
        x: 100.0,
        y: 280.0,
        timestamp: 5250,
    });
    
    let touch2_end = InputEvent::new(InputEventType::TouchEnd {
        id: 2,
        x: 200.0,
        y: 320.0,
        timestamp: 5260,
    });
    
    // イベントを処理
    gesture_manager.process_event(&touch1_end);
    gesture_manager.process_event(&touch2_end);
    
    // テスト3: マウスを使用した回転シミュレーション (Ctrl+右クリック)
    println!("\nテスト3: マウスを使用した回転シミュレーション (Ctrl+右クリック)");
    
    // 状態をリセット
    gesture_manager.reset_all();
    
    // Ctrl修飾キーの準備
    let mut modifiers = HashSet::new();
    modifiers.insert(KeyModifier::Ctrl);
    
    // マウス右クリック (Ctrl押下)
    let mouse_press = InputEvent::new(InputEventType::MousePress {
        button: MouseButton::Right,
        x: 300.0,
        y: 300.0,
        modifiers: modifiers.clone(),
        timestamp: 6000,
    });
    
    // イベントを処理
    gesture_manager.process_event(&mouse_press);
    
    // マウスを移動して回転をシミュレーション
    for i in 1..=5 {
        let mouse_move = InputEvent::new(InputEventType::MouseMove {
            x: 300.0 + 10.0 * i as f64,
            y: 300.0 - 15.0 * i as f64,
            dx: 10.0,
            dy: -15.0,
            modifiers: modifiers.clone(),
            timestamp: 6050 + (i as u64 * 30),
        });
        
        // イベントを処理
        let gestures = gesture_manager.process_event(&mouse_move);
        
        println!("  ステップ {}: 検出されたジェスチャー数: {}", i, gestures.len());
    }
    
    // マウスリリース
    let mouse_release = InputEvent::new(InputEventType::MouseRelease {
        button: MouseButton::Right,
        x: 350.0,
        y: 225.0,
        modifiers: modifiers.clone(),
        timestamp: 6250,
    });
    
    // イベントを処理
    gesture_manager.process_event(&mouse_release);
} 