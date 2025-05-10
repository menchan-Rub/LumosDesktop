// LumosDesktop 回転ジェスチャー認識器
// 二本指を使った回転操作を検出する

use std::collections::{HashMap, HashSet};
use std::time::Instant;
use std::f64::consts::PI;

use crate::core::window_manager::scene_graph::NodeId;
use crate::core::window_manager::input_translator::{
    InputEvent, InputEventType, MouseButton, KeyModifier,
};
use crate::core::window_manager::gesture_recognizer::{
    GestureRecognizer, GestureType, GestureState, GestureInfo, SwipeDirection,
};

/// 回転方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationDirection {
    /// 時計回り
    Clockwise,
    /// 反時計回り
    CounterClockwise,
}

/// タッチポイント情報
#[derive(Debug, Clone)]
struct TouchPoint {
    id: u64,
    position: (f64, f64),
    timestamp: u64,
}

/// 回転認識器
pub struct RotateRecognizer {
    /// タッチポイント
    touch_points: HashMap<u64, TouchPoint>,
    /// 初期角度
    initial_angle: Option<f64>,
    /// 現在の角度
    current_angle: Option<f64>,
    /// 累積回転角度 (ラジアン)
    accumulated_rotation: f64,
    /// 中心位置
    center_position: Option<(f64, f64)>,
    /// 対象ノード
    target: Option<NodeId>,
    /// 入力デバイスソース
    source_device: Option<String>,
    /// アクティブフラグ
    is_active: bool,
    /// 認識済みフラグ
    is_recognized: bool,
    /// 開始タイムスタンプ
    start_timestamp: Option<u64>,
    /// 最終タイムスタンプ
    last_timestamp: Option<u64>,
    /// 最小角度変化閾値 (ラジアン)
    min_angle_threshold: f64,
    /// 修飾キー
    modifiers: HashSet<KeyModifier>,
    /// 開始時刻
    start_time: Option<Instant>,
    /// 最後の回転方向
    last_rotation_direction: Option<RotationDirection>,
}

impl RotateRecognizer {
    /// 新しい回転認識器を作成
    pub fn new() -> Self {
        Self {
            touch_points: HashMap::new(),
            initial_angle: None,
            current_angle: None,
            accumulated_rotation: 0.0,
            center_position: None,
            target: None,
            source_device: None,
            is_active: false,
            is_recognized: false,
            start_timestamp: None,
            last_timestamp: None,
            min_angle_threshold: 0.05, // 約3度
            modifiers: HashSet::new(),
            start_time: None,
            last_rotation_direction: None,
        }
    }
    
    /// 最小角度閾値を設定
    pub fn with_min_angle_threshold(mut self, threshold: f64) -> Self {
        self.min_angle_threshold = threshold;
        self
    }
    
    /// 2点間の角度を計算（ラジアン）
    fn calculate_angle(&self, p1: &(f64, f64), p2: &(f64, f64)) -> f64 {
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        dy.atan2(dx)
    }
    
    /// 2点の中点を計算
    fn calculate_center(&self, p1: &(f64, f64), p2: &(f64, f64)) -> (f64, f64) {
        ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0)
    }
    
    /// 角度の差分を正規化 (-π～π)
    fn normalize_angle_diff(&self, angle_diff: f64) -> f64 {
        let mut result = angle_diff;
        while result > PI {
            result -= 2.0 * PI;
        }
        while result < -PI {
            result += 2.0 * PI;
        }
        result
    }
    
    /// 回転操作を確認し、認識イベントを生成
    fn check_rotation(&mut self, timestamp: u64) -> Option<GestureInfo> {
        if self.touch_points.len() != 2 {
            return None;
        }
        
        let points: Vec<&TouchPoint> = self.touch_points.values().collect();
        let p1 = &points[0].position;
        let p2 = &points[1].position;
        
        // 中心位置
        let center = self.calculate_center(p1, p2);
        self.center_position = Some(center);
        
        // 現在の角度
        let current_angle = self.calculate_angle(p1, p2);
        self.current_angle = Some(current_angle);
        
        // 最初の測定
        if self.initial_angle.is_none() {
            self.initial_angle = Some(current_angle);
            self.start_timestamp = Some(timestamp);
            return None;
        }
        
        let initial_angle = self.initial_angle.unwrap();
        
        // 角度の変化量
        let angle_diff = self.normalize_angle_diff(current_angle - initial_angle);
        
        // 角度変化が小さすぎる場合はイベントを生成しない
        if angle_diff.abs() < self.min_angle_threshold && self.is_recognized {
            return None;
        }
        
        // 累積回転角度を更新
        self.accumulated_rotation += angle_diff;
        
        // 回転方向
        let direction = if angle_diff > 0.0 {
            RotationDirection::CounterClockwise
        } else {
            RotationDirection::Clockwise
        };
        
        // ジェスチャー状態
        let state = if !self.is_recognized {
            self.is_recognized = true;
            self.last_rotation_direction = Some(direction);
            GestureState::Began
        } else if self.last_rotation_direction != Some(direction) {
            // 方向が変わった（時計回りから反時計回りへなど）
            self.last_rotation_direction = Some(direction);
            GestureState::Began
        } else {
            GestureState::Changed
        };
        
        let mut gesture = GestureInfo::new(
            GestureType::Rotate,
            state,
            timestamp,
        )
        .with_position(center)
        .with_rotation(self.accumulated_rotation);
        
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
        
        // 次回用に現在の角度を初期角度として設定
        self.initial_angle = Some(current_angle);
        
        Some(gesture)
    }
}

impl GestureRecognizer for RotateRecognizer {
    fn name(&self) -> &'static str {
        "Rotate Recognizer"
    }
    
    fn gesture_type(&self) -> GestureType {
        GestureType::Rotate
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
                    self.accumulated_rotation = 0.0;
                    self.initial_angle = None;
                    self.current_angle = None;
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
                
                // アクティブな場合は回転チェック
                if self.is_active && self.touch_points.len() == 2 {
                    return self.check_rotation(*timestamp);
                }
                
                None
            }
            InputEventType::TouchEnd {
                id,
                x: _,
                y: _,
                timestamp,
            } => {
                // タッチポイントを削除
                self.touch_points.remove(id);
                
                // すべてのタッチが終了したか、認識に必要な数のタッチが不足した場合
                if self.touch_points.is_empty() || self.touch_points.len() < 2 {
                    // アクティブな状態であれば終了イベントを生成
                    if self.is_active && self.is_recognized {
                        let state = GestureState::Ended;
                        let mut gesture = GestureInfo::new(
                            GestureType::Rotate,
                            state,
                            *timestamp,
                        );
                        
                        if let Some(center) = self.center_position {
                            gesture = gesture.with_position(center);
                        }
                        
                        gesture = gesture.with_rotation(self.accumulated_rotation);
                        
                        if let Some(target) = self.target {
                            gesture = gesture.with_target(target);
                        }
                        
                        if !self.modifiers.is_empty() {
                            gesture = gesture.with_modifiers(self.modifiers.clone());
                        }
                        
                        if let Some(source) = &self.source_device {
                            gesture = gesture.with_source_device(source.clone());
                        }
                        
                        // 認識器をリセット
                        self.reset();
                        
                        return Some(gesture);
                    }
                    
                    // リセット
                    self.reset();
                }
                
                None
            }
            InputEventType::TouchCancel {
                id,
                timestamp,
            } => {
                // タッチポイントを削除
                self.touch_points.remove(id);
                
                // アクティブな状態であればキャンセルイベントを生成
                if self.is_active && self.is_recognized {
                    let state = GestureState::Cancelled;
                    let mut gesture = GestureInfo::new(
                        GestureType::Rotate,
                        state,
                        *timestamp,
                    );
                    
                    if let Some(center) = self.center_position {
                        gesture = gesture.with_position(center);
                    }
                    
                    gesture = gesture.with_rotation(self.accumulated_rotation);
                    
                    if let Some(target) = self.target {
                        gesture = gesture.with_target(target);
                    }
                    
                    if !self.modifiers.is_empty() {
                        gesture = gesture.with_modifiers(self.modifiers.clone());
                    }
                    
                    if let Some(source) = &self.source_device {
                        gesture = gesture.with_source_device(source.clone());
                    }
                    
                    // リセット
                    self.reset();
                    
                    return Some(gesture);
                }
                
                // リセット
                self.reset();
                
                None
            }
            // マウスを使用した回転シミュレーション（Ctrlキー + 右ドラッグ）
            InputEventType::MousePress {
                button,
                x,
                y,
                modifiers,
                timestamp,
            } => {
                if *button == MouseButton::Right && modifiers.contains(&KeyModifier::Ctrl) {
                    // 初期化
                    self.reset();
                    
                    // タッチポイント1を追加（仮想的な固定点）
                    let touch_point1 = TouchPoint {
                        id: 1000, // 仮想ID
                        position: (*x - 50.0, *y),
                        timestamp: *timestamp,
                    };
                    
                    // タッチポイント2を追加（マウスポイント）
                    let touch_point2 = TouchPoint {
                        id: 1001, // 仮想ID
                        position: (*x, *y),
                        timestamp: *timestamp,
                    };
                    
                    self.touch_points.insert(1000, touch_point1);
                    self.touch_points.insert(1001, touch_point2);
                    
                    self.is_active = true;
                    self.is_recognized = false;
                    self.accumulated_rotation = 0.0;
                    self.initial_angle = None;
                    self.current_angle = None;
                    self.target = event.target;
                    self.source_device = event.source_device.clone();
                    self.modifiers = modifiers.clone();
                    self.start_time = Some(Instant::now());
                }
                
                None
            }
            InputEventType::MouseMove {
                x,
                y,
                dx: _,
                dy: _,
                modifiers,
                timestamp,
            } => {
                // マウスでの回転シミュレーション中
                if self.is_active && self.touch_points.contains_key(&1001) {
                    // マウスポイントを更新
                    if let Some(touch_point) = self.touch_points.get_mut(&1001) {
                        touch_point.position = (*x, *y);
                        touch_point.timestamp = *timestamp;
                    }
                    
                    self.modifiers = modifiers.clone();
                    
                    return self.check_rotation(*timestamp);
                }
                
                None
            }
            InputEventType::MouseRelease {
                button,
                x: _,
                y: _,
                modifiers,
                timestamp,
            } => {
                // マウスでの回転シミュレーション終了
                if *button == MouseButton::Right && self.is_active {
                    // アクティブな状態であれば終了イベントを生成
                    if self.is_recognized {
                        let state = GestureState::Ended;
                        let mut gesture = GestureInfo::new(
                            GestureType::Rotate,
                            state,
                            *timestamp,
                        );
                        
                        if let Some(center) = self.center_position {
                            gesture = gesture.with_position(center);
                        }
                        
                        gesture = gesture.with_rotation(self.accumulated_rotation);
                        
                        if let Some(target) = self.target {
                            gesture = gesture.with_target(target);
                        }
                        
                        gesture = gesture.with_modifiers(modifiers.clone());
                        
                        if let Some(source) = &self.source_device {
                            gesture = gesture.with_source_device(source.clone());
                        }
                        
                        // リセット
                        self.reset();
                        
                        return Some(gesture);
                    }
                    
                    // リセット
                    self.reset();
                }
                
                None
            }
            _ => None,
        }
    }
    
    fn reset(&mut self) {
        self.touch_points.clear();
        self.initial_angle = None;
        self.current_angle = None;
        self.accumulated_rotation = 0.0;
        self.center_position = None;
        self.is_active = false;
        self.is_recognized = false;
        self.start_timestamp = None;
        self.last_timestamp = None;
        self.modifiers.clear();
        self.start_time = None;
        self.last_rotation_direction = None;
    }
    
    fn is_active(&self) -> bool {
        self.is_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    
    #[test]
    fn test_rotate_recognizer() {
        let mut recognizer = RotateRecognizer::new();
        
        // 初期状態
        assert!(!recognizer.is_active());
        
        // タッチ開始（2点）
        let touch1_begin = InputEvent::new(InputEventType::TouchBegin {
            id: 1,
            x: 100.0,
            y: 100.0,
            pressure: 1.0,
            timestamp: 1000,
        });
        
        let touch2_begin = InputEvent::new(InputEventType::TouchBegin {
            id: 2,
            x: 200.0,
            y: 100.0,
            pressure: 1.0,
            timestamp: 1010,
        });
        
        recognizer.update(&touch1_begin);
        recognizer.update(&touch2_begin);
        
        assert!(recognizer.is_active());
        
        // 時計回りに回転
        let touch1_update1 = InputEvent::new(InputEventType::TouchUpdate {
            id: 1,
            x: 100.0,
            y: 120.0,
            dx: 0.0,
            dy: 20.0,
            pressure: 1.0,
            timestamp: 1100,
        });
        
        let touch2_update1 = InputEvent::new(InputEventType::TouchUpdate {
            id: 2,
            x: 200.0,
            y: 80.0,
            dx: 0.0,
            dy: -20.0,
            pressure: 1.0,
            timestamp: 1110,
        });
        
        let gesture1 = recognizer.update(&touch1_update1);
        let gesture2 = recognizer.update(&touch2_update1);
        
        // 最初の更新では手が動いただけなので、認識されていない
        assert!(gesture1.is_none());
        
        // 2点目の更新で回転が認識されるはず
        assert!(gesture2.is_some());
        if let Some(g) = gesture2 {
            assert_eq!(g.gesture_type, GestureType::Rotate);
            assert_eq!(g.state, GestureState::Began);
            
            // 回転角度の確認（約0.38ラジアン≒22度）
            let rotation_angle = g.rotation.abs();
            assert!(rotation_angle > 0.35 && rotation_angle < 0.40, 
                   "Expected rotation angle around 0.38 radians, got {}", rotation_angle);
        }
        
        // さらに回転
        let touch1_update2 = InputEvent::new(InputEventType::TouchUpdate {
            id: 1,
            x: 80.0,
            y: 130.0,
            dx: -20.0,
            dy: 10.0,
            pressure: 1.0,
            timestamp: 1200,
        });
        
        let touch2_update2 = InputEvent::new(InputEventType::TouchUpdate {
            id: 2,
            x: 220.0,
            y: 70.0,
            dx: 20.0,
            dy: -10.0,
            pressure: 1.0,
            timestamp: 1210,
        });
        
        let gesture3 = recognizer.update(&touch1_update2);
        let gesture4 = recognizer.update(&touch2_update2);
        
        assert!(gesture3.is_some());
        assert!(gesture4.is_some());
        
        if let Some(g) = gesture4 {
            assert_eq!(g.gesture_type, GestureType::Rotate);
            assert_eq!(g.state, GestureState::Changed);
        }
        
        // タッチ終了
        let touch1_end = InputEvent::new(InputEventType::TouchEnd {
            id: 1,
            x: 80.0,
            y: 130.0,
            timestamp: 1300,
        });
        
        let touch2_end = InputEvent::new(InputEventType::TouchEnd {
            id: 2,
            x: 220.0,
            y: 70.0,
            timestamp: 1310,
        });
        
        let gesture5 = recognizer.update(&touch1_end);
        let gesture6 = recognizer.update(&touch2_end);
        
        // 最初の指が離れた時点では終了していない
        assert!(gesture5.is_none());
        
        // 2本目の指が離れた時点で終了
        assert!(gesture6.is_some());
        if let Some(g) = gesture6 {
            assert_eq!(g.gesture_type, GestureType::Rotate);
            assert_eq!(g.state, GestureState::Ended);
        }
        
        // リセット後はアクティブでない
        assert!(!recognizer.is_active());
    }
    
    #[test]
    fn test_rotate_recognizer_mouse_simulation() {
        let mut recognizer = RotateRecognizer::new();
        
        // 初期状態
        assert!(!recognizer.is_active());
        
        // Ctrl+右クリックでの開始
        let mut modifiers = HashSet::new();
        modifiers.insert(KeyModifier::Ctrl);
        
        let mouse_press = InputEvent::new(InputEventType::MousePress {
            button: MouseButton::Right,
            x: 200.0,
            y: 200.0,
            modifiers: modifiers.clone(),
            timestamp: 2000,
        });
        
        recognizer.update(&mouse_press);
        
        assert!(recognizer.is_active());
        
        // マウス移動（回転シミュレーション）
        let mouse_move1 = InputEvent::new(InputEventType::MouseMove {
            x: 220.0,
            y: 220.0,
            dx: 20.0,
            dy: 20.0,
            modifiers: modifiers.clone(),
            timestamp: 2100,
        });
        
        let gesture1 = recognizer.update(&mouse_move1);
        
        assert!(gesture1.is_some());
        if let Some(g) = gesture1 {
            assert_eq!(g.gesture_type, GestureType::Rotate);
            assert_eq!(g.state, GestureState::Began);
        }
        
        // さらにマウス移動
        let mouse_move2 = InputEvent::new(InputEventType::MouseMove {
            x: 240.0,
            y: 180.0,
            dx: 20.0,
            dy: -40.0,
            modifiers: modifiers.clone(),
            timestamp: 2200,
        });
        
        let gesture2 = recognizer.update(&mouse_move2);
        
        assert!(gesture2.is_some());
        if let Some(g) = gesture2 {
            assert_eq!(g.gesture_type, GestureType::Rotate);
            assert_eq!(g.state, GestureState::Changed);
        }
        
        // マウスリリースで終了
        let mouse_release = InputEvent::new(InputEventType::MouseRelease {
            button: MouseButton::Right,
            x: 240.0,
            y: 180.0,
            modifiers: modifiers.clone(),
            timestamp: 2300,
        });
        
        let gesture3 = recognizer.update(&mouse_release);
        
        assert!(gesture3.is_some());
        if let Some(g) = gesture3 {
            assert_eq!(g.gesture_type, GestureType::Rotate);
            assert_eq!(g.state, GestureState::Ended);
        }
        
        // リセット後はアクティブでない
        assert!(!recognizer.is_active());
    }
    
    #[test]
    fn test_angle_calculation() {
        let recognizer = RotateRecognizer::new();
        
        // 0度
        let angle1 = recognizer.calculate_angle(&(100.0, 100.0), &(200.0, 100.0));
        assert!((angle1 - 0.0).abs() < 0.001);
        
        // 90度
        let angle2 = recognizer.calculate_angle(&(100.0, 100.0), &(100.0, 200.0));
        assert!((angle2 - PI/2.0).abs() < 0.001);
        
        // 180度
        let angle3 = recognizer.calculate_angle(&(100.0, 100.0), &(0.0, 100.0));
        assert!((angle3 - PI).abs() < 0.001);
        
        // -90度
        let angle4 = recognizer.calculate_angle(&(100.0, 100.0), &(100.0, 0.0));
        assert!((angle4 - (-PI/2.0)).abs() < 0.001);
    }
    
    #[test]
    fn test_angle_normalization() {
        let recognizer = RotateRecognizer::new();
        
        // 通常範囲内
        let norm1 = recognizer.normalize_angle_diff(PI/2.0);
        assert!((norm1 - PI/2.0).abs() < 0.001);
        
        // πよりも大きい
        let norm2 = recognizer.normalize_angle_diff(3.0 * PI/2.0);
        assert!((norm2 - (-PI/2.0)).abs() < 0.001);
        
        // -πよりも小さい
        let norm3 = recognizer.normalize_angle_diff(-3.0 * PI/2.0);
        assert!((norm3 - (PI/2.0)).abs() < 0.001);
        
        // ちょうどπ
        let norm4 = recognizer.normalize_angle_diff(PI);
        assert!((norm4 - PI).abs() < 0.001);
        
        // ちょうど-π
        let norm5 = recognizer.normalize_angle_diff(-PI);
        assert!((norm5 - (-PI)).abs() < 0.001);
    }
} 