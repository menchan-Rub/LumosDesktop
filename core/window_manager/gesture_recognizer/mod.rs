// LumosDesktop ジェスチャー認識モジュール
// タッチやマウス入力からの様々なジェスチャーを検出する機能を提供します

//! ジェスチャー認識モジュール
//!
//! このモジュールは入力イベントからジェスチャーを認識する機能を提供します。
//! 複数の認識器が並行して動作し、それぞれが特定のジェスチャーパターンを検出します。
//! 検出されたジェスチャーはウィンドウマネージャを通じてアプリケーションに通知されます。

pub mod gesture_recognizer;
pub mod tap_recognizer;
pub mod double_tap_recognizer;
pub mod long_press_recognizer;
pub mod swipe_recognizer;
pub mod pinch_recognizer;
pub mod rotate_recognizer;
pub mod edge_swipe_recognizer;

// 主要な型の再エクスポート
pub use gesture_recognizer::{
    GestureRecognizer, GestureType, GestureState, GestureInfo,
    SwipeDirection, TouchPoint
};
pub use rotate_recognizer::RotationDirection;

use std::collections::HashMap;
use std::time::Instant;

use crate::core::window_manager::input_translator::InputEvent;

/// マルチジェスチャー処理を担当するジェスチャーマネージャー
pub struct GestureManager {
    recognizers: HashMap<GestureType, Box<dyn GestureRecognizer + Send + Sync>>,
    last_update: Instant,
    active_recognizers: Vec<GestureType>,
    gesture_callbacks: Vec<Box<dyn Fn(&GestureInfo) -> bool + Send + Sync>>,
}

impl GestureManager {
    /// 新しいジェスチャーマネージャーを作成
    pub fn new() -> Self {
        Self {
            recognizers: HashMap::new(),
            last_update: Instant::now(),
            active_recognizers: Vec::new(),
            gesture_callbacks: Vec::new(),
        }
    }
    
    /// 認識器を登録
    pub fn register_recognizer(&mut self, recognizer: Box<dyn GestureRecognizer + Send + Sync>) {
        let gesture_type = recognizer.gesture_type();
        self.recognizers.insert(gesture_type, recognizer);
    }
    
    /// デフォルトの認識器をすべて登録
    pub fn register_default_recognizers(&mut self) {
        self.register_recognizer(Box::new(tap_recognizer::TapRecognizer::new()));
        self.register_recognizer(Box::new(double_tap_recognizer::DoubleTapRecognizer::new()));
        self.register_recognizer(Box::new(long_press_recognizer::LongPressRecognizer::new()));
        self.register_recognizer(Box::new(swipe_recognizer::SwipeRecognizer::new()));
        self.register_recognizer(Box::new(pinch_recognizer::PinchRecognizer::new()));
        self.register_recognizer(Box::new(rotate_recognizer::RotateRecognizer::new()));
        
        // 一部の環境では追加のジェスチャーも登録可能
        if cfg!(feature = "advanced_gestures") {
            self.register_recognizer(Box::new(edge_swipe_recognizer::EdgeSwipeRecognizer::new()));
        }
    }
    
    /// ジェスチャーコールバックを登録
    pub fn add_gesture_callback<F>(&mut self, callback: F)
    where
        F: Fn(&GestureInfo) -> bool + Send + Sync + 'static,
    {
        self.gesture_callbacks.push(Box::new(callback));
    }
    
    /// 入力イベントを処理してジェスチャーを検出
    pub fn process_event(&mut self, event: &InputEvent) -> Vec<GestureInfo> {
        let mut detected_gestures = Vec::new();
        
        // アクティブでない認識器を更新
        for (gesture_type, recognizer) in self.recognizers.iter_mut() {
            if !self.active_recognizers.contains(gesture_type) {
                if let Some(gesture) = recognizer.update(event) {
                    // ジェスチャーの開始
                    if gesture.state == GestureState::Began {
                        self.active_recognizers.push(*gesture_type);
                    }
                    
                    detected_gestures.push(gesture.clone());
                    
                    // コールバックの実行
                    for callback in &self.gesture_callbacks {
                        if !callback(&gesture) {
                            break;
                        }
                    }
                }
            }
        }
        
        // アクティブな認識器を優先的に更新
        let mut completed_gestures = Vec::new();
        
        for gesture_type in &self.active_recognizers {
            if let Some(recognizer) = self.recognizers.get_mut(gesture_type) {
                if let Some(gesture) = recognizer.update(event) {
                    detected_gestures.push(gesture.clone());
                    
                    // コールバックの実行
                    for callback in &self.gesture_callbacks {
                        if !callback(&gesture) {
                            break;
                        }
                    }
                    
                    // ジェスチャーの終了
                    if gesture.state == GestureState::Ended || 
                       gesture.state == GestureState::Cancelled ||
                       gesture.state == GestureState::Failed {
                        completed_gestures.push(*gesture_type);
                    }
                }
            }
        }
        
        // 完了したジェスチャーをアクティブリストから削除
        for gesture_type in completed_gestures {
            if let Some(pos) = self.active_recognizers.iter().position(|&gt| gt == gesture_type) {
                self.active_recognizers.remove(pos);
            }
        }
        
        self.last_update = Instant::now();
        detected_gestures
    }
    
    /// すべての認識器をリセット
    pub fn reset_all(&mut self) {
        for (_, recognizer) in self.recognizers.iter_mut() {
            recognizer.reset();
        }
        self.active_recognizers.clear();
    }
    
    /// 特定のジェスチャー認識器を取得
    pub fn get_recognizer(&self, gesture_type: GestureType) -> Option<&dyn GestureRecognizer> {
        self.recognizers.get(&gesture_type).map(|r| r.as_ref())
    }
    
    /// 特定のジェスチャー認識器を取得（可変参照）
    pub fn get_recognizer_mut(&mut self, gesture_type: GestureType) -> Option<&mut dyn GestureRecognizer> {
        self.recognizers.get_mut(&gesture_type).map(|r| r.as_mut())
    }
    
    /// アクティブなジェスチャー認識器があるかどうか
    pub fn has_active_recognizers(&self) -> bool {
        !self.active_recognizers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::window_manager::input_translator::{InputEventType, MouseButton};
    use std::collections::HashSet;
    
    #[test]
    fn test_gesture_manager() {
        let mut manager = GestureManager::new();
        manager.register_default_recognizers();
        
        // タップイベントの生成
        let event = InputEvent::new(InputEventType::MousePress {
            button: MouseButton::Left,
            x: 100.0,
            y: 100.0,
            modifiers: HashSet::new(),
            timestamp: 1000,
        });
        
        // イベントの処理
        let gestures = manager.process_event(&event);
        assert!(gestures.is_empty()); // タップはまだ完了していない
        
        // リリースイベント
        let event = InputEvent::new(InputEventType::MouseRelease {
            button: MouseButton::Left,
            x: 100.0,
            y: 100.0,
            modifiers: HashSet::new(),
            timestamp: 1050,
        });
        
        // イベントの処理
        let gestures = manager.process_event(&event);
        assert_eq!(gestures.len(), 1);
        assert_eq!(gestures[0].gesture_type, GestureType::Tap);
    }
} 