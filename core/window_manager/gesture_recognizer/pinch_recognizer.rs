// LumosDesktop ピンチ認識器
// 二本指でのピンチイン・ピンチアウト操作を認識する

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::core::window_manager::scene_graph::NodeId;
use crate::core::window_manager::input_translator::{
    InputEvent, InputEventType, MouseButton, KeyModifier,
};
use crate::core::window_manager::gesture_recognizer::{
    GestureRecognizer, GestureType, GestureState, GestureInfo, SwipeDirection,
};

/// ピンチ操作のパターン
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinchPattern {
    /// ピンチイン（縮小）
    In,
    /// ピンチアウト（拡大）
    Out,
}

/// タッチポイント情報
#[derive(Debug, Clone)]
struct TouchPoint {
    id: u64,
    position: (f64, f64),
    timestamp: u64,
}

/// ピンチ認識器
pub struct PinchRecognizer {
    touch_points: HashMap<u64, TouchPoint>,
    initial_distance: Option<f64>,
    current_distance: Option<f64>,
    center_position: Option<(f64, f64)>,
    target: Option<NodeId>,
    source_device: Option<String>,
    is_active: bool,
    is_recognized: bool,
    start_timestamp: Option<u64>,
    last_timestamp: Option<u64>,
    scale_factor: f64,
    min_distance_threshold: f64,
    min_scale_change_threshold: f64,
    modifiers: HashSet<KeyModifier>,
    start_time: Option<Instant>,
    last_gesture_pattern: Option<PinchPattern>,
}

impl PinchRecognizer {
    pub fn new() -> Self {
        Self {
            touch_points: HashMap::new(),
            initial_distance: None,
            current_distance: None,
            center_position: None,
            target: None,
            source_device: None,
            is_active: false,
            is_recognized: false,
            start_timestamp: None,
            last_timestamp: None,
            scale_factor: 1.0,
            min_distance_threshold: 20.0, // 最小距離（ピクセル）
            min_scale_change_threshold: 0.05, // 最小スケール変更（5%）
            modifiers: HashSet::new(),
            start_time: None,
            last_gesture_pattern: None,
        }
    }
    
    pub fn with_min_distance_threshold(mut self, threshold: f64) -> Self {
        self.min_distance_threshold = threshold;
        self
    }
    
    pub fn with_min_scale_change_threshold(mut self, threshold: f64) -> Self {
        self.min_scale_change_threshold = threshold;
        self
    }
    
    /// 2点間の距離を計算
    fn calculate_distance(&self, p1: &(f64, f64), p2: &(f64, f64)) -> f64 {
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// 2点の中点を計算
    fn calculate_center(&self, p1: &(f64, f64), p2: &(f64, f64)) -> (f64, f64) {
        ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0)
    }
    
    /// ピンチ操作を確認し、認識イベントを生成
    fn check_pinch(&mut self, timestamp: u64) -> Option<GestureInfo> {
        if self.touch_points.len() != 2 {
            return None;
        }
        
        let points: Vec<&TouchPoint> = self.touch_points.values().collect();
        let p1 = &points[0].position;
        let p2 = &points[1].position;
        
        // 現在の距離
        let current_distance = self.calculate_distance(p1, p2);
        self.current_distance = Some(current_distance);
        
        // 中心位置
        let center = self.calculate_center(p1, p2);
        self.center_position = Some(center);
        
        // 最初の測定
        if self.initial_distance.is_none() {
            self.initial_distance = Some(current_distance);
            self.start_timestamp = Some(timestamp);
            return None;
        }
        
        let initial_distance = self.initial_distance.unwrap();
        
        // 距離が短すぎる場合は認識しない
        if initial_distance < self.min_distance_threshold {
            return None;
        }
        
        // スケールファクター
        let new_scale = current_distance / initial_distance;
        let scale_change = (new_scale - self.scale_factor).abs();
        
        // スケール変更が小さすぎる場合はイベントを生成しない
        if scale_change < self.min_scale_change_threshold && self.is_recognized {
            return None;
        }
        
        self.scale_factor = new_scale;
        
        // ピンチパターン
        let pattern = if new_scale < 1.0 {
            PinchPattern::In
        } else {
            PinchPattern::Out
        };
        
        // ジェスチャー状態
        let state = if !self.is_recognized {
            self.is_recognized = true;
            self.last_gesture_pattern = Some(pattern);
            GestureState::Began
        } else if self.last_gesture_pattern != Some(pattern) {
            // パターンが変わった（ピンチインからピンチアウトへなど）
            self.last_gesture_pattern = Some(pattern);
            GestureState::Began
        } else {
            GestureState::Changed
        };
        
        let mut gesture = GestureInfo::new(
            GestureType::Pinch,
            state,
            timestamp,
        )
        .with_position(center)
        .with_scale(new_scale);
        
        // ピンチパターン情報
        if pattern == PinchPattern::In {
            gesture = gesture.with_pinch_in();
        } else {
            gesture = gesture.with_pinch_out();
        }
        
        // 追加情報
        if let Some(target) = self.target {
            gesture = gesture.with_target(target);
        }
        
        if !self.modifiers.is_empty() {
            gesture = gesture.with_modifiers(self.modifiers.clone());
        }
        
        if let Some(source) = &self.source_device {
            gesture = gesture.with_source_device(source.clone());
        }
        
        self.last_timestamp = Some(timestamp);
        
        Some(gesture)
    }
}

impl GestureRecognizer for PinchRecognizer {
    fn name(&self) -> &'static str {
        "Pinch Recognizer"
    }
    
    fn gesture_type(&self) -> GestureType {
        GestureType::Pinch
    }
    
    fn update(&mut self, event: &InputEvent) -> Option<GestureInfo> {
        match &event.event_type {
            InputEventType::TouchBegin {
                id,
                x,
                y,
                pressure: _,
                timestamp,
            } => {
                // 新しいタッチポイントを追加
                let touch_point = TouchPoint {
                    id: *id,
                    position: (*x, *y),
                    timestamp: *timestamp,
                };
                
                self.touch_points.insert(*id, touch_point);
                
                // 2点が揃った時点で、まだアクティブでなければ開始
                if self.touch_points.len() == 2 && !self.is_active {
                    self.is_active = true;
                    self.is_recognized = false;
                    self.scale_factor = 1.0;
                    self.initial_distance = None;
                    self.current_distance = None;
                    self.target = event.target;
                    self.source_device = event.source_device.clone();
                    self.modifiers = HashSet::new();
                    self.start_time = Some(Instant::now());
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
            } => {
                // タッチポイントが存在する場合は更新
                if let Some(touch_point) = self.touch_points.get_mut(id) {
                    touch_point.position = (*x, *y);
                    touch_point.timestamp = *timestamp;
                }
                
                // アクティブな場合はピンチチェック
                if self.is_active && self.touch_points.len() == 2 {
                    self.check_pinch(*timestamp)
                } else {
                    None
                }
            }
            InputEventType::TouchEnd {
                id,
                x: _,
                y: _,
                timestamp,
            } => {
                // タッチポイントを削除
                self.touch_points.remove(id);
                
                // ジェスチャーを終了
                if self.is_active && self.is_recognized {
                    let result = if let Some(center) = self.center_position {
                        let mut gesture = GestureInfo::new(
                            GestureType::Pinch,
                            GestureState::Ended,
                            *timestamp,
                        )
                        .with_position(center);
                        
                        if let Some(scale) = self.current_distance.map(|d| {
                            let initial = self.initial_distance.unwrap_or(1.0);
                            if initial > 0.0 { d / initial } else { 1.0 }
                        }) {
                            gesture = gesture.with_scale(scale);
                        }
                        
                        // ピンチパターン情報
                        if let Some(pattern) = self.last_gesture_pattern {
                            if pattern == PinchPattern::In {
                                gesture = gesture.with_pinch_in();
                            } else {
                                gesture = gesture.with_pinch_out();
                            }
                        }
                        
                        if let Some(target) = self.target {
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
                } else {
                    // 残りのタッチポイントが1つの場合は、まだリセットしない
                    if self.touch_points.is_empty() {
                        self.reset();
                    }
                    None
                }
            }
            InputEventType::MouseWheel {
                delta_x,
                delta_y,
                delta_z: _,
                x,
                y,
                modifiers,
                timestamp,
            } if event.source_device.as_deref() == Some("touchpad") => {
                // タッチパッドからのピンチジェスチャーをシミュレート
                // 通常、マルチタッチトラックパッドはCtrlキーと組み合わせた
                // ホイールイベントとしてピンチジェスチャーを送信します
                
                if modifiers.contains(&KeyModifier::Ctrl) {
                    let position = (*x, *y);
                    
                    // スケールファクターを計算
                    // delta_yを使用（一般的な実装）
                    let delta_scale = if *delta_y != 0.0 {
                        1.0 - (*delta_y * 0.01) // 調整可能
                    } else {
                        1.0
                    };
                    
                    if !self.is_active {
                        // 新しいピンチジェスチャーの開始
                        self.is_active = true;
                        self.is_recognized = true;
                        self.scale_factor = 1.0;
                        self.center_position = Some(position);
                        self.target = event.target;
                        self.source_device = event.source_device.clone();
                        self.modifiers = modifiers.clone();
                        self.start_time = Some(Instant::now());
                        self.start_timestamp = Some(*timestamp);
                        self.last_timestamp = Some(*timestamp);
                        
                        // 初回スケールで方向を決定
                        let pattern = if delta_scale < 1.0 {
                            PinchPattern::In
                        } else {
                            PinchPattern::Out
                        };
                        
                        self.last_gesture_pattern = Some(pattern);
                        
                        let mut gesture = GestureInfo::new(
                            GestureType::Pinch,
                            GestureState::Began,
                            *timestamp,
                        )
                        .with_position(position)
                        .with_scale(delta_scale);
                        
                        // ピンチパターン情報
                        if pattern == PinchPattern::In {
                            gesture = gesture.with_pinch_in();
                        } else {
                            gesture = gesture.with_pinch_out();
                        }
                        
                        if let Some(target) = self.target {
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
                        // 継続中のピンチジェスチャー
                        self.center_position = Some(position);
                        self.scale_factor *= delta_scale;
                        
                        // 新しいパターン
                        let pattern = if delta_scale < 1.0 {
                            PinchPattern::In
                        } else {
                            PinchPattern::Out
                        };
                        
                        // 方向が変わったかどうか
                        let state = if self.last_gesture_pattern != Some(pattern) {
                            self.last_gesture_pattern = Some(pattern);
                            GestureState::Began
                        } else {
                            GestureState::Changed
                        };
                        
                        let mut gesture = GestureInfo::new(
                            GestureType::Pinch,
                            state,
                            *timestamp,
                        )
                        .with_position(position)
                        .with_scale(self.scale_factor);
                        
                        // ピンチパターン情報
                        if pattern == PinchPattern::In {
                            gesture = gesture.with_pinch_in();
                        } else {
                            gesture = gesture.with_pinch_out();
                        }
                        
                        if let Some(target) = self.target {
                            gesture = gesture.with_target(target);
                        }
                        
                        if !self.modifiers.is_empty() {
                            gesture = gesture.with_modifiers(self.modifiers.clone());
                        }
                        
                        if let Some(source) = &self.source_device {
                            gesture = gesture.with_source_device(source.clone());
                        }
                        
                        self.last_timestamp = Some(*timestamp);
                        
                        Some(gesture)
                    }
                } else {
                    // トラックパッドからのピンチジェスチャーが終了した場合
                    if self.is_active && self.is_recognized && 
                       event.source_device.as_deref() == Some("touchpad") {
                        // 一定時間経過後に自動終了
                        let now = Instant::now();
                        if let Some(start) = self.start_time {
                            if now.duration_since(start).as_millis() > 200 {
                                let result = if let Some(center) = self.center_position {
                                    let mut gesture = GestureInfo::new(
                                        GestureType::Pinch,
                                        GestureState::Ended,
                                        *timestamp,
                                    )
                                    .with_position(center)
                                    .with_scale(self.scale_factor);
                                    
                                    // ピンチパターン情報
                                    if let Some(pattern) = self.last_gesture_pattern {
                                        if pattern == PinchPattern::In {
                                            gesture = gesture.with_pinch_in();
                                        } else {
                                            gesture = gesture.with_pinch_out();
                                        }
                                    }
                                    
                                    if let Some(target) = self.target {
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
                                return result;
                            }
                        }
                    }
                    
                    None
                }
            }
            _ => None,
        }
    }
    
    fn reset(&mut self) {
        self.touch_points.clear();
        self.initial_distance = None;
        self.current_distance = None;
        self.center_position = None;
        self.target = None;
        self.source_device = None;
        self.is_active = false;
        self.is_recognized = false;
        self.start_timestamp = None;
        self.last_timestamp = None;
        self.scale_factor = 1.0;
        self.modifiers.clear();
        self.start_time = None;
        self.last_gesture_pattern = None;
    }
    
    fn is_active(&self) -> bool {
        self.is_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pinch_recognizer() {
        let mut recognizer = PinchRecognizer::new()
            .with_min_distance_threshold(10.0);
            
        // 第1タッチポイント
        let timestamp = 1000;
        let event = InputEvent::new(InputEventType::TouchBegin {
            id: 1,
            x: 100.0,
            y: 100.0,
            pressure: 1.0,
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_none());
        assert!(!recognizer.is_active());
        
        // 第2タッチポイント（開始）
        let timestamp = 1010;
        let event = InputEvent::new(InputEventType::TouchBegin {
            id: 2,
            x: 120.0,
            y: 120.0,
            pressure: 1.0,
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_none());
        assert!(recognizer.is_active());
        
        // タッチポイント1の更新（ピンチアウト開始）
        let timestamp = 1020;
        let event = InputEvent::new(InputEventType::TouchUpdate {
            id: 1,
            x: 90.0,
            y: 90.0,
            dx: -10.0,
            dy: -10.0,
            pressure: 1.0,
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Pinch);
            assert_eq!(gesture.state, GestureState::Began);
            assert!(gesture.scale.is_some());
            assert!(gesture.is_pinch_out);
            assert!(!gesture.is_pinch_in);
        }
        
        // タッチポイント2の更新（ピンチアウト継続）
        let timestamp = 1030;
        let event = InputEvent::new(InputEventType::TouchUpdate {
            id: 2,
            x: 130.0,
            y: 130.0,
            dx: 10.0,
            dy: 10.0,
            pressure: 1.0,
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Pinch);
            assert_eq!(gesture.state, GestureState::Changed);
            assert!(gesture.scale.is_some());
            assert!(gesture.is_pinch_out);
            assert!(!gesture.is_pinch_in);
        }
        
        // 方向転換（ピンチイン開始）
        let timestamp = 1040;
        let event = InputEvent::new(InputEventType::TouchUpdate {
            id: 1,
            x: 100.0,
            y: 100.0,
            dx: 10.0,
            dy: 10.0,
            pressure: 1.0,
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Pinch);
            assert_eq!(gesture.state, GestureState::Began); // 方向が変わったので再開始
            assert!(gesture.scale.is_some());
            assert!(!gesture.is_pinch_out);
            assert!(gesture.is_pinch_in);
        }
        
        // タッチポイント1終了
        let timestamp = 1050;
        let event = InputEvent::new(InputEventType::TouchEnd {
            id: 1,
            x: 100.0,
            y: 100.0,
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Pinch);
            assert_eq!(gesture.state, GestureState::Ended);
            assert!(gesture.scale.is_some());
        }
        
        assert!(!recognizer.is_active());
    }
    
    #[test]
    fn test_pinch_recognizer_with_touchpad() {
        let mut recognizer = PinchRecognizer::new();
        
        // トラックパッドからのCtrl+ホイールイベント（ピンチイン）
        let mut modifiers = HashSet::new();
        modifiers.insert(KeyModifier::Ctrl);
        
        let timestamp = 1000;
        let mut event = InputEvent::new(InputEventType::MouseWheel {
            delta_x: 0.0,
            delta_y: 1.0, // 正の値でピンチイン
            delta_z: 0.0,
            x: 200.0,
            y: 200.0,
            modifiers: modifiers.clone(),
            timestamp,
        });
        event.source_device = Some("touchpad".to_string());
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Pinch);
            assert_eq!(gesture.state, GestureState::Began);
            assert!(gesture.scale.is_some());
            assert!(!gesture.is_pinch_out);
            assert!(gesture.is_pinch_in);
        }
        
        // 続けてのイベント（ピンチアウト）
        let timestamp = 1010;
        let mut event = InputEvent::new(InputEventType::MouseWheel {
            delta_x: 0.0,
            delta_y: -1.0, // 負の値でピンチアウト
            delta_z: 0.0,
            x: 200.0,
            y: 200.0,
            modifiers: modifiers.clone(),
            timestamp,
        });
        event.source_device = Some("touchpad".to_string());
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Pinch);
            assert_eq!(gesture.state, GestureState::Began); // 方向が変わったので再開始
            assert!(gesture.scale.is_some());
            assert!(gesture.is_pinch_out);
            assert!(!gesture.is_pinch_in);
        }
        
        // 終了イベント（modifierなし）
        let timestamp = 1020;
        let mut event = InputEvent::new(InputEventType::MouseWheel {
            delta_x: 0.0,
            delta_y: 0.0,
            delta_z: 0.0,
            x: 200.0,
            y: 200.0,
            modifiers: HashSet::new(), // Ctrlなし
            timestamp,
        });
        event.source_device = Some("touchpad".to_string());
        
        // 長時間経過したとみなす
        recognizer.start_time = Some(Instant::now() - std::time::Duration::from_millis(300));
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Pinch);
            assert_eq!(gesture.state, GestureState::Ended);
        }
        
        assert!(!recognizer.is_active());
    }
} 