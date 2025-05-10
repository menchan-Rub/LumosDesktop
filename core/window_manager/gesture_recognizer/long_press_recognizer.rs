// LumosDesktop 長押し認識器
// 一定時間以上のタッチやクリックを長押しとして認識する

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::core::window_manager::scene_graph::NodeId;
use crate::core::window_manager::input_translator::{
    InputEvent, InputEventType, MouseButton, KeyModifier,
};
use crate::core::window_manager::gesture_recognizer::{
    GestureRecognizer, GestureType, GestureState, GestureInfo, SwipeDirection,
};

/// 長押し認識器
pub struct LongPressRecognizer {
    press_position: Option<(f64, f64)>,
    press_timestamp: Option<u64>,
    press_target: Option<NodeId>,
    source_device: Option<String>,
    modifiers: HashSet<KeyModifier>,
    is_active: bool,
    is_recognized: bool,
    movement_threshold: f64,
    long_press_time: Duration,
    feedback_interval: Duration,
    start_time: Option<Instant>,
    last_feedback_time: Option<Instant>,
    touch_id: Option<u64>,
}

impl LongPressRecognizer {
    pub fn new() -> Self {
        Self {
            press_position: None,
            press_timestamp: None,
            press_target: None,
            source_device: None,
            modifiers: HashSet::new(),
            is_active: false,
            is_recognized: false,
            movement_threshold: 15.0, // ピクセル
            long_press_time: Duration::from_millis(500), // 500ミリ秒
            feedback_interval: Duration::from_millis(100), // 100ミリ秒ごとに更新イベント
            start_time: None,
            last_feedback_time: None,
            touch_id: None,
        }
    }
    
    pub fn with_movement_threshold(mut self, threshold: f64) -> Self {
        self.movement_threshold = threshold;
        self
    }
    
    pub fn with_long_press_time(mut self, time: Duration) -> Self {
        self.long_press_time = time;
        self
    }
    
    pub fn with_feedback_interval(mut self, interval: Duration) -> Self {
        self.feedback_interval = interval;
        self
    }
    
    /// 長押し時間を確認し、認識イベントを生成
    fn check_long_press(&mut self, current_position: (f64, f64), timestamp: u64) -> Option<GestureInfo> {
        if let (Some(start_pos), Some(start_time)) = (self.press_position, self.start_time) {
            // 移動距離をチェック
            let dx = current_position.0 - start_pos.0;
            let dy = current_position.1 - start_pos.1;
            let distance = (dx * dx + dy * dy).sqrt();
            
            // 動きが多すぎる場合は長押しをキャンセル
            if distance > self.movement_threshold {
                self.reset();
                return None;
            }
            
            let elapsed = Instant::now().duration_since(start_time);
            
            // 長押し時間に達したかチェック
            if elapsed >= self.long_press_time {
                if !self.is_recognized {
                    // 初回認識
                    self.is_recognized = true;
                    self.last_feedback_time = Some(Instant::now());
                    
                    let mut gesture = GestureInfo::new(
                        GestureType::LongPress,
                        GestureState::Began,
                        timestamp,
                    )
                    .with_position(current_position)
                    .with_start_position(start_pos)
                    .with_long_press_duration(elapsed);
                    
                    if let Some(target) = self.press_target {
                        gesture = gesture.with_target(target);
                    }
                    
                    if !self.modifiers.is_empty() {
                        gesture = gesture.with_modifiers(self.modifiers.clone());
                    }
                    
                    if let Some(source) = &self.source_device {
                        gesture = gesture.with_source_device(source.clone());
                    }
                    
                    return Some(gesture);
                } else if let Some(last_time) = self.last_feedback_time {
                    // 継続中の長押し - 定期的な更新
                    let since_last = Instant::now().duration_since(last_time);
                    
                    if since_last >= self.feedback_interval {
                        self.last_feedback_time = Some(Instant::now());
                        
                        let mut gesture = GestureInfo::new(
                            GestureType::LongPress,
                            GestureState::Changed,
                            timestamp,
                        )
                        .with_position(current_position)
                        .with_start_position(start_pos)
                        .with_long_press_duration(elapsed);
                        
                        if let Some(target) = self.press_target {
                            gesture = gesture.with_target(target);
                        }
                        
                        if !self.modifiers.is_empty() {
                            gesture = gesture.with_modifiers(self.modifiers.clone());
                        }
                        
                        if let Some(source) = &self.source_device {
                            gesture = gesture.with_source_device(source.clone());
                        }
                        
                        return Some(gesture);
                    }
                }
            }
        }
        
        None
    }
}

impl GestureRecognizer for LongPressRecognizer {
    fn name(&self) -> &'static str {
        "Long Press Recognizer"
    }
    
    fn gesture_type(&self) -> GestureType {
        GestureType::LongPress
    }
    
    fn update(&mut self, event: &InputEvent) -> Option<GestureInfo> {
        match &event.event_type {
            InputEventType::MousePress {
                button: MouseButton::Left,
                x,
                y,
                modifiers,
                timestamp,
            } => {
                // 長押しの開始
                self.press_position = Some((*x, *y));
                self.press_timestamp = Some(*timestamp);
                self.press_target = event.target;
                self.modifiers = modifiers.clone();
                self.source_device = event.source_device.clone();
                self.is_active = true;
                self.is_recognized = false;
                self.start_time = Some(Instant::now());
                self.last_feedback_time = None;
                
                None
            }
            InputEventType::MouseMove { x, y, dx: _, dy: _, modifiers: _, timestamp } if self.is_active => {
                // 長押し中の移動をチェック
                self.check_long_press((*x, *y), *timestamp)
            }
            InputEventType::MouseRelease {
                button: MouseButton::Left,
                x,
                y,
                modifiers: _,
                timestamp,
            } if self.is_active => {
                // 長押しの終了
                let result = if self.is_recognized {
                    let mut gesture = GestureInfo::new(
                        GestureType::LongPress,
                        GestureState::Ended,
                        *timestamp,
                    )
                    .with_position((*x, *y));
                    
                    if let Some(start_pos) = self.press_position {
                        gesture = gesture.with_start_position(start_pos);
                    }
                    
                    if let Some(start_time) = self.start_time {
                        let elapsed = Instant::now().duration_since(start_time);
                        gesture = gesture.with_long_press_duration(elapsed);
                    }
                    
                    if let Some(target) = self.press_target {
                        gesture = gesture.with_target(target);
                    }
                    
                    if !self.modifiers.is_empty() {
                        gesture = gesture.with_modifiers(self.modifiers.clone());
                    }
                    
                    if let Some(source) = &self.source_device {
                        gesture = gesture.with_source_device(source.clone());
                    }
                    
                    Some(gesture)
                } else {
                    None
                };
                
                self.reset();
                result
            }
            InputEventType::TouchBegin {
                id,
                x,
                y,
                pressure: _,
                timestamp,
            } => {
                // 長押しの開始
                if !self.is_active {
                    self.press_position = Some((*x, *y));
                    self.press_timestamp = Some(*timestamp);
                    self.press_target = event.target;
                    self.source_device = event.source_device.clone();
                    self.is_active = true;
                    self.is_recognized = false;
                    self.start_time = Some(Instant::now());
                    self.last_feedback_time = None;
                    self.touch_id = Some(*id);
                }
                
                None
            }
            InputEventType::TouchUpdate {
                id,
                x,
                y,
                dx: _,
                dy: _,
                pressure: _,
                timestamp,
            } if self.is_active && self.touch_id == Some(*id) => {
                // 長押し中の移動をチェック
                self.check_long_press((*x, *y), *timestamp)
            }
            InputEventType::TouchEnd {
                id,
                x,
                y,
                timestamp,
            } if self.is_active && self.touch_id == Some(*id) => {
                // 長押しの終了
                let result = if self.is_recognized {
                    let mut gesture = GestureInfo::new(
                        GestureType::LongPress,
                        GestureState::Ended,
                        *timestamp,
                    )
                    .with_position((*x, *y));
                    
                    if let Some(start_pos) = self.press_position {
                        gesture = gesture.with_start_position(start_pos);
                    }
                    
                    if let Some(start_time) = self.start_time {
                        let elapsed = Instant::now().duration_since(start_time);
                        gesture = gesture.with_long_press_duration(elapsed);
                    }
                    
                    if let Some(target) = self.press_target {
                        gesture = gesture.with_target(target);
                    }
                    
                    if let Some(source) = &self.source_device {
                        gesture = gesture.with_source_device(source.clone());
                    }
                    
                    Some(gesture)
                } else {
                    None
                };
                
                self.reset();
                result
            }
            // 長押し中にタイマーイベントを発生させるため、空イベントも処理
            InputEventType::Idle { timestamp } if self.is_active => {
                if let Some(pos) = self.press_position {
                    self.check_long_press(pos, *timestamp)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    fn reset(&mut self) {
        self.press_position = None;
        self.press_timestamp = None;
        self.press_target = None;
        self.source_device = None;
        self.modifiers.clear();
        self.is_active = false;
        self.is_recognized = false;
        self.start_time = None;
        self.last_feedback_time = None;
        self.touch_id = None;
    }
    
    fn is_active(&self) -> bool {
        self.is_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_long_press_recognizer() {
        let mut recognizer = LongPressRecognizer::new()
            .with_long_press_time(Duration::from_millis(100)); // テスト用に短い時間
            
        // プレス開始
        let timestamp = 1000;
        let event = InputEvent::new(InputEventType::MousePress {
            button: MouseButton::Left,
            x: 100.0,
            y: 100.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_none());
        assert!(recognizer.is_active());
        
        // 待機（長押し時間）
        thread::sleep(Duration::from_millis(150));
        
        // 移動イベント（長押し認識トリガー）
        let timestamp = 1150;
        let event = InputEvent::new(InputEventType::MouseMove {
            x: 105.0, // 少しだけ移動（閾値内）
            y: 105.0,
            dx: 5.0,
            dy: 5.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::LongPress);
            assert_eq!(gesture.state, GestureState::Began);
            assert!(gesture.long_press_duration.is_some());
        }
        
        // さらに少し動かす（更新イベント）
        thread::sleep(Duration::from_millis(150));
        
        let timestamp = 1300;
        let event = InputEvent::new(InputEventType::MouseMove {
            x: 108.0,
            y: 108.0,
            dx: 3.0,
            dy: 3.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::LongPress);
            assert_eq!(gesture.state, GestureState::Changed);
        }
        
        // リリース（終了イベント）
        let timestamp = 1500;
        let event = InputEvent::new(InputEventType::MouseRelease {
            button: MouseButton::Left,
            x: 110.0,
            y: 110.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::LongPress);
            assert_eq!(gesture.state, GestureState::Ended);
            assert!(gesture.long_press_duration.is_some());
        }
        
        assert!(!recognizer.is_active());
    }
    
    #[test]
    fn test_long_press_cancel_on_move() {
        let mut recognizer = LongPressRecognizer::new()
            .with_long_press_time(Duration::from_millis(200))
            .with_movement_threshold(10.0);
            
        // プレス開始
        let timestamp = 1000;
        let event = InputEvent::new(InputEventType::MousePress {
            button: MouseButton::Left,
            x: 100.0,
            y: 100.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        recognizer.update(&event);
        
        // 大きく移動（閾値超え）
        let timestamp = 1050;
        let event = InputEvent::new(InputEventType::MouseMove {
            x: 120.0,  // 20px移動（閾値超過）
            y: 120.0,
            dx: 20.0,
            dy: 20.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_none());
        
        // 長押し時間が過ぎても認識されないことを確認
        thread::sleep(Duration::from_millis(250));
        
        let timestamp = 1300;
        let event = InputEvent::new(InputEventType::MouseMove {
            x: 121.0,
            y: 121.0,
            dx: 1.0,
            dy: 1.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_none());
        
        // リリース
        let timestamp = 1500;
        let event = InputEvent::new(InputEventType::MouseRelease {
            button: MouseButton::Left,
            x: 122.0,
            y: 122.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_none());
    }
} 