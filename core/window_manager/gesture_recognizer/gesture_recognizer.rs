// LumosDesktop ジェスチャー認識器
// タッチやマウスのジェスチャーを認識し、アクションに変換するシステム

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

use crate::core::window_manager::scene_graph::NodeId;
use crate::core::window_manager::input_translator::{
    InputEvent, InputEventType, MouseButton, KeyModifier,
};

/// ジェスチャー種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureType {
    Tap,             // タップ（クリック）
    DoubleTap,       // ダブルタップ（ダブルクリック）
    LongPress,       // 長押し
    Swipe,           // スワイプ
    Pinch,           // ピンチ（ズーム）
    Rotate,          // 回転
    Pan,             // パン（ドラッグ）
    Edge,            // 画面端からのスワイプ
    ThreeFingerDrag, // 3本指ドラッグ
    FourFingerSwipe, // 4本指スワイプ
}

/// スワイプ方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

/// ジェスチャー状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureState {
    Began,        // 開始
    Changed,      // 変更
    Ended,        // 終了
    Cancelled,    // キャンセル
    Failed,       // 失敗
    Recognized,   // 認識完了
}

/// タッチポイント情報
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub id: u64,              // タッチID
    pub position: (f64, f64), // 位置
    pub timestamp: u64,       // タイムスタンプ
    pub pressure: f64,        // 圧力 (0.0-1.0)
}

impl TouchPoint {
    pub fn new(id: u64, position: (f64, f64), timestamp: u64, pressure: f64) -> Self {
        Self {
            id,
            position,
            timestamp,
            pressure,
        }
    }
    
    pub fn distance(&self, other: &TouchPoint) -> f64 {
        let dx = self.position.0 - other.position.0;
        let dy = self.position.1 - other.position.1;
        (dx * dx + dy * dy).sqrt()
    }
}

/// ジェスチャー情報
#[derive(Debug, Clone)]
pub struct GestureInfo {
    pub gesture_type: GestureType,
    pub state: GestureState,
    pub timestamp: u64,
    pub target: Option<NodeId>,
    pub position: (f64, f64),
    pub start_position: (f64, f64),
    pub delta: (f64, f64),
    pub velocity: (f64, f64),
    pub scale: f64,             // ピンチ用
    pub rotation: f64,          // 回転用
    pub touch_count: usize,     // タッチ数
    pub swipe_direction: Option<SwipeDirection>,
    pub long_press_duration: Option<Duration>,
    pub source_device: Option<String>,
    pub modifiers: HashSet<KeyModifier>,
}

impl GestureInfo {
    pub fn new(gesture_type: GestureType, state: GestureState, timestamp: u64) -> Self {
        Self {
            gesture_type,
            state,
            timestamp,
            target: None,
            position: (0.0, 0.0),
            start_position: (0.0, 0.0),
            delta: (0.0, 0.0),
            velocity: (0.0, 0.0),
            scale: 1.0,
            rotation: 0.0,
            touch_count: 0,
            swipe_direction: None,
            long_press_duration: None,
            source_device: None,
            modifiers: HashSet::new(),
        }
    }
    
    pub fn with_target(mut self, target: NodeId) -> Self {
        self.target = Some(target);
        self
    }
    
    pub fn with_position(mut self, position: (f64, f64)) -> Self {
        self.position = position;
        self
    }
    
    pub fn with_start_position(mut self, start_position: (f64, f64)) -> Self {
        self.start_position = start_position;
        self
    }
    
    pub fn with_delta(mut self, delta: (f64, f64)) -> Self {
        self.delta = delta;
        self
    }
    
    pub fn with_velocity(mut self, velocity: (f64, f64)) -> Self {
        self.velocity = velocity;
        self
    }
    
    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }
    
    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }
    
    pub fn with_touch_count(mut self, touch_count: usize) -> Self {
        self.touch_count = touch_count;
        self
    }
    
    pub fn with_swipe_direction(mut self, direction: SwipeDirection) -> Self {
        self.swipe_direction = Some(direction);
        self
    }
    
    pub fn with_long_press_duration(mut self, duration: Duration) -> Self {
        self.long_press_duration = Some(duration);
        self
    }
    
    pub fn with_source_device(mut self, source: String) -> Self {
        self.source_device = Some(source);
        self
    }
    
    pub fn with_modifiers(mut self, modifiers: HashSet<KeyModifier>) -> Self {
        self.modifiers = modifiers;
        self
    }
    
    // ピンチイン情報を追加
    pub fn with_pinch_in(mut self) -> Self {
        if self.gesture_type == GestureType::Pinch {
            self.swipe_direction = Some(SwipeDirection::Down); // ピンチインを下方向と関連付け
        }
        self
    }
    
    // ピンチアウト情報を追加
    pub fn with_pinch_out(mut self) -> Self {
        if self.gesture_type == GestureType::Pinch {
            self.swipe_direction = Some(SwipeDirection::Up); // ピンチアウトを上方向と関連付け
        }
        self
    }
    
    pub fn distance_from_start(&self) -> f64 {
        let dx = self.position.0 - self.start_position.0;
        let dy = self.position.1 - self.start_position.1;
        (dx * dx + dy * dy).sqrt()
    }
    
    pub fn is_horizontal_movement(&self) -> bool {
        self.delta.0.abs() > self.delta.1.abs()
    }
    
    pub fn is_vertical_movement(&self) -> bool {
        self.delta.1.abs() > self.delta.0.abs()
    }
}

/// ジェスチャーコールバック
pub type GestureCallback = Box<dyn Fn(&GestureInfo) -> bool + Send + Sync>;

/// タッチトラッカー - タッチの履歴を追跡
struct TouchTracker {
    active_touches: HashMap<u64, Vec<TouchPoint>>,
    start_time: HashMap<u64, u64>,
    last_update: Instant,
}

impl TouchTracker {
    fn new() -> Self {
        Self {
            active_touches: HashMap::new(),
            start_time: HashMap::new(),
            last_update: Instant::now(),
        }
    }
    
    fn add_touch(&mut self, touch: TouchPoint) {
        if !self.active_touches.contains_key(&touch.id) {
            self.start_time.insert(touch.id, touch.timestamp);
        }
        
        self.active_touches
            .entry(touch.id)
            .or_insert_with(Vec::new)
            .push(touch);
    }
    
    fn update_touch(&mut self, touch: TouchPoint) {
        if let Some(touches) = self.active_touches.get_mut(&touch.id) {
            touches.push(touch);
            
            // 履歴が長すぎる場合は古いものを削除
            const MAX_HISTORY: usize = 20;
            if touches.len() > MAX_HISTORY {
                touches.remove(0);
            }
        }
    }
    
    fn remove_touch(&mut self, touch_id: u64) {
        self.active_touches.remove(&touch_id);
        self.start_time.remove(&touch_id);
    }
    
    fn get_touch(&self, touch_id: u64) -> Option<&TouchPoint> {
        self.active_touches
            .get(&touch_id)
            .and_then(|touches| touches.last())
    }
    
    fn get_touch_history(&self, touch_id: u64) -> Option<&Vec<TouchPoint>> {
        self.active_touches.get(&touch_id)
    }
    
    fn get_active_touch_count(&self) -> usize {
        self.active_touches.len()
    }
    
    fn get_all_active_touches(&self) -> Vec<&TouchPoint> {
        self.active_touches
            .values()
            .filter_map(|touches| touches.last())
            .collect()
    }
    
    fn get_touch_duration(&self, touch_id: u64, current_timestamp: u64) -> Option<u64> {
        self.start_time
            .get(&touch_id)
            .map(|start| current_timestamp.saturating_sub(*start))
    }
    
    fn clear(&mut self) {
        self.active_touches.clear();
        self.start_time.clear();
    }
    
    fn calculate_velocity(&self, touch_id: u64) -> Option<(f64, f64)> {
        if let Some(touches) = self.active_touches.get(&touch_id) {
            if touches.len() < 2 {
                return Some((0.0, 0.0));
            }
            
            let latest = touches.last().unwrap();
            let previous = touches.get(touches.len() - 2).unwrap();
            
            let dt = (latest.timestamp - previous.timestamp) as f64 / 1000.0; // ミリ秒→秒
            if dt <= 0.0 {
                return Some((0.0, 0.0));
            }
            
            let dx = latest.position.0 - previous.position.0;
            let dy = latest.position.1 - previous.position.1;
            
            return Some((dx / dt, dy / dt));
        }
        
        None
    }
}

/// ジェスチャー認識器ベース - すべての認識器の基底トレイト
pub trait GestureRecognizer: Send + Sync {
    fn name(&self) -> &'static str;
    fn gesture_type(&self) -> GestureType;
    fn update(&mut self, event: &InputEvent) -> Option<GestureInfo>;
    fn reset(&mut self);
    fn is_active(&self) -> bool;
}

/// タップ認識器
pub struct TapRecognizer {
    tap_position: Option<(f64, f64)>,
    tap_timestamp: Option<u64>,
    tap_target: Option<NodeId>,
    source_device: Option<String>,
    modifiers: HashSet<KeyModifier>,
    is_active: bool,
    tap_threshold: f64,
    timeout: Duration,
    start_time: Option<Instant>,
}

impl TapRecognizer {
    pub fn new() -> Self {
        Self {
            tap_position: None,
            tap_timestamp: None,
            tap_target: None,
            source_device: None,
            modifiers: HashSet::new(),
            is_active: false,
            tap_threshold: 10.0, // ピクセル
            timeout: Duration::from_millis(300), // 300ミリ秒
            start_time: None,
        }
    }
    
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.tap_threshold = threshold;
        self
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl GestureRecognizer for TapRecognizer {
    fn name(&self) -> &'static str {
        "Tap Recognizer"
    }
    
    fn gesture_type(&self) -> GestureType {
        GestureType::Tap
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
                // タップの開始
                self.tap_position = Some((*x, *y));
                self.tap_timestamp = Some(*timestamp);
                self.tap_target = event.target;
                self.modifiers = modifiers.clone();
                self.source_device = event.source_device.clone();
                self.is_active = true;
                self.start_time = Some(Instant::now());
                
                None
            }
            InputEventType::MouseRelease {
                button: MouseButton::Left,
                x,
                y,
                modifiers,
                timestamp,
            } if self.is_active => {
                // タップの終了
                if let (Some(start_pos), Some(start_time)) = (self.tap_position, self.start_time) {
                    // 距離とタイムアウトをチェック
                    let dx = *x - start_pos.0;
                    let dy = *y - start_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let elapsed = Instant::now().duration_since(start_time);
                    
                    if distance <= self.tap_threshold && elapsed <= self.timeout {
                        // タップとして認識
                        let gesture = GestureInfo::new(
                            GestureType::Tap,
                            GestureState::Recognized,
                            *timestamp,
                        )
                        .with_position((*x, *y))
                        .with_start_position(start_pos)
                        .with_modifiers(modifiers.clone());
                        
                        if let Some(target) = self.tap_target {
                            gesture.with_target(target);
                        }
                        
                        if let Some(source) = &self.source_device {
                            gesture.with_source_device(source.clone());
                        }
                        
                        self.reset();
                        return Some(gesture);
                    }
                }
                
                self.reset();
                None
            }
            InputEventType::MouseMove { x, y, .. } if self.is_active => {
                // タップ中の移動をチェック
                if let (Some(start_pos), Some(start_time)) = (self.tap_position, self.start_time) {
                    // 距離とタイムアウトをチェック
                    let dx = *x - start_pos.0;
                    let dy = *y - start_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let elapsed = Instant::now().duration_since(start_time);
                    
                    if distance > self.tap_threshold || elapsed > self.timeout {
                        // タップ認識失敗
                        self.reset();
                    }
                }
                
                None
            }
            InputEventType::TouchBegin {
                id,
                x,
                y,
                pressure: _,
                timestamp,
            } => {
                // シングルタッチの場合のみタップとして扱う
                if !self.is_active {
                    self.tap_position = Some((*x, *y));
                    self.tap_timestamp = Some(*timestamp);
                    self.tap_target = event.target;
                    self.source_device = event.source_device.clone();
                    self.is_active = true;
                    self.start_time = Some(Instant::now());
                }
                
                None
            }
            InputEventType::TouchEnd {
                id: _,
                x,
                y,
                timestamp,
            } if self.is_active => {
                // タップの終了
                if let (Some(start_pos), Some(start_time)) = (self.tap_position, self.start_time) {
                    // 距離とタイムアウトをチェック
                    let dx = *x - start_pos.0;
                    let dy = *y - start_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let elapsed = Instant::now().duration_since(start_time);
                    
                    if distance <= self.tap_threshold && elapsed <= self.timeout {
                        // タップとして認識
                        let gesture = GestureInfo::new(
                            GestureType::Tap,
                            GestureState::Recognized,
                            *timestamp,
                        )
                        .with_position((*x, *y))
                        .with_start_position(start_pos);
                        
                        if let Some(target) = self.tap_target {
                            gesture.with_target(target);
                        }
                        
                        if let Some(source) = &self.source_device {
                            gesture.with_source_device(source.clone());
                        }
                        
                        self.reset();
                        return Some(gesture);
                    }
                }
                
                self.reset();
                None
            }
            InputEventType::TouchUpdate {
                id: _,
                x,
                y,
                dx: _,
                dy: _,
                pressure: _,
                timestamp: _,
            } if self.is_active => {
                // タップ中の移動をチェック
                if let (Some(start_pos), Some(start_time)) = (self.tap_position, self.start_time) {
                    // 距離とタイムアウトをチェック
                    let dx = *x - start_pos.0;
                    let dy = *y - start_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let elapsed = Instant::now().duration_since(start_time);
                    
                    if distance > self.tap_threshold || elapsed > self.timeout {
                        // タップ認識失敗
                        self.reset();
                    }
                }
                
                None
            }
            _ => None,
        }
    }
    
    fn reset(&mut self) {
        self.tap_position = None;
        self.tap_timestamp = None;
        self.tap_target = None;
        self.source_device = None;
        self.modifiers.clear();
        self.is_active = false;
        self.start_time = None;
    }
    
    fn is_active(&self) -> bool {
        self.is_active
    }
}

/// スワイプ認識器
pub struct SwipeRecognizer {
    start_position: Option<(f64, f64)>,
    current_position: Option<(f64, f64)>,
    start_timestamp: Option<u64>,
    target: Option<NodeId>,
    source_device: Option<String>,
    modifiers: HashSet<KeyModifier>,
    is_active: bool,
    min_distance: f64,
    max_time: Duration,
    start_time: Option<Instant>,
    touch_id: Option<u64>,
}

impl SwipeRecognizer {
    pub fn new() -> Self {
        Self {
            start_position: None,
            current_position: None,
            start_timestamp: None,
            target: None,
            source_device: None,
            modifiers: HashSet::new(),
            is_active: false,
            min_distance: 50.0, // ピクセル
            max_time: Duration::from_millis(500), // 500ミリ秒
            start_time: None,
            touch_id: None,
        }
    }
    
    pub fn with_min_distance(mut self, distance: f64) -> Self {
        self.min_distance = distance;
        self
    }
    
    pub fn with_max_time(mut self, time: Duration) -> Self {
        self.max_time = time;
        self
    }
    
    fn detect_direction(&self) -> Option<SwipeDirection> {
        if let (Some(start), Some(current)) = (self.start_position, self.current_position) {
            let dx = current.0 - start.0;
            let dy = current.1 - start.1;
            
            // 水平または垂直の判定
            let abs_dx = dx.abs();
            let abs_dy = dy.abs();
            
            if abs_dx > abs_dy {
                // 水平方向
                if dx > 0.0 {
                    Some(SwipeDirection::Right)
                } else {
                    Some(SwipeDirection::Left)
                }
            } else {
                // 垂直方向
                if dy > 0.0 {
                    Some(SwipeDirection::Down)
                } else {
                    Some(SwipeDirection::Up)
                }
            }
        } else {
            None
        }
    }
}

impl GestureRecognizer for SwipeRecognizer {
    fn name(&self) -> &'static str {
        "Swipe Recognizer"
    }
    
    fn gesture_type(&self) -> GestureType {
        GestureType::Swipe
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
                // スワイプの開始
                self.start_position = Some((*x, *y));
                self.current_position = Some((*x, *y));
                self.start_timestamp = Some(*timestamp);
                self.target = event.target;
                self.modifiers = modifiers.clone();
                self.source_device = event.source_device.clone();
                self.is_active = true;
                self.start_time = Some(Instant::now());
                
                None
            }
            InputEventType::MouseMove { x, y, dx, dy, .. } if self.is_active => {
                // スワイプ中の移動
                self.current_position = Some((*x, *y));
                
                if let (Some(start_pos), Some(start_time)) = (self.start_position, self.start_time) {
                    // 距離をチェック
                    let dx = *x - start_pos.0;
                    let dy = *y - start_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance >= self.min_distance {
                        // スワイプとして認識
                        let elapsed = Instant::now().duration_since(start_time);
                        if elapsed <= self.max_time {
                            let direction = self.detect_direction();
                            
                            let gesture = GestureInfo::new(
                                GestureType::Swipe,
                                GestureState::Changed,
                                self.start_timestamp.unwrap_or(0),
                            )
                            .with_position((*x, *y))
                            .with_start_position(start_pos)
                            .with_delta((dx, dy))
                            .with_modifiers(self.modifiers.clone());
                            
                            if let Some(dir) = direction {
                                gesture.with_swipe_direction(dir);
                            }
                            
                            if let Some(target) = self.target {
                                gesture.with_target(target);
                            }
                            
                            if let Some(source) = &self.source_device {
                                gesture.with_source_device(source.clone());
                            }
                            
                            return Some(gesture);
                        }
                    }
                }
                
                None
            }
            InputEventType::MouseRelease {
                button: MouseButton::Left,
                x,
                y,
                timestamp,
                ..
            } if self.is_active => {
                // スワイプの終了
                let result = if let (Some(start_pos), Some(start_time)) = (self.start_position, self.start_time) {
                    // 距離と時間をチェック
                    let dx = *x - start_pos.0;
                    let dy = *y - start_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let elapsed = Instant::now().duration_since(start_time);
                    
                    if distance >= self.min_distance && elapsed <= self.max_time {
                        // スワイプとして認識
                        let direction = self.detect_direction();
                        
                        let gesture = GestureInfo::new(
                            GestureType::Swipe,
                            GestureState::Ended,
                            *timestamp,
                        )
                        .with_position((*x, *y))
                        .with_start_position(start_pos)
                        .with_delta((dx, dy))
                        .with_modifiers(self.modifiers.clone());
                        
                        if let Some(dir) = direction {
                            gesture.with_swipe_direction(dir);
                        }
                        
                        if let Some(target) = self.target {
                            gesture.with_target(target);
                        }
                        
                        if let Some(source) = &self.source_device {
                            gesture.with_source_device(source.clone());
                        }
                        
                        Some(gesture)
                    } else {
                        None
                    }
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
                // スワイプの開始
                if !self.is_active {
                    self.start_position = Some((*x, *y));
                    self.current_position = Some((*x, *y));
                    self.start_timestamp = Some(*timestamp);
                    self.target = event.target;
                    self.source_device = event.source_device.clone();
                    self.is_active = true;
                    self.start_time = Some(Instant::now());
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
                // スワイプ中の移動
                self.current_position = Some((*x, *y));
                
                if let (Some(start_pos), Some(start_time)) = (self.start_position, self.start_time) {
                    // 距離をチェック
                    let dx = *x - start_pos.0;
                    let dy = *y - start_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance >= self.min_distance {
                        // スワイプとして認識
                        let elapsed = Instant::now().duration_since(start_time);
                        if elapsed <= self.max_time {
                            let direction = self.detect_direction();
                            
                            let gesture = GestureInfo::new(
                                GestureType::Swipe,
                                GestureState::Changed,
                                *timestamp,
                            )
                            .with_position((*x, *y))
                            .with_start_position(start_pos)
                            .with_delta((dx, dy));
                            
                            if let Some(dir) = direction {
                                gesture.with_swipe_direction(dir);
                            }
                            
                            if let Some(target) = self.target {
                                gesture.with_target(target);
                            }
                            
                            if let Some(source) = &self.source_device {
                                gesture.with_source_device(source.clone());
                            }
                            
                            return Some(gesture);
                        }
                    }
                }
                
                None
            }
            InputEventType::TouchEnd {
                id,
                x,
                y,
                timestamp,
            } if self.is_active && self.touch_id == Some(*id) => {
                // スワイプの終了
                let result = if let (Some(start_pos), Some(start_time)) = (self.start_position, self.start_time) {
                    // 距離と時間をチェック
                    let dx = *x - start_pos.0;
                    let dy = *y - start_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let elapsed = Instant::now().duration_since(start_time);
                    
                    if distance >= self.min_distance && elapsed <= self.max_time {
                        // スワイプとして認識
                        let direction = self.detect_direction();
                        
                        let gesture = GestureInfo::new(
                            GestureType::Swipe,
                            GestureState::Ended,
                            *timestamp,
                        )
                        .with_position((*x, *y))
                        .with_start_position(start_pos)
                        .with_delta((dx, dy));
                        
                        if let Some(dir) = direction {
                            gesture.with_swipe_direction(dir);
                        }
                        
                        if let Some(target) = self.target {
                            gesture.with_target(target);
                        }
                        
                        if let Some(source) = &self.source_device {
                            gesture.with_source_device(source.clone());
                        }
                        
                        Some(gesture)
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                self.reset();
                result
            }
            _ => None,
        }
    }
    
    fn reset(&mut self) {
        self.start_position = None;
        self.current_position = None;
        self.start_timestamp = None;
        self.target = None;
        self.source_device = None;
        self.modifiers.clear();
        self.is_active = false;
        self.start_time = None;
        self.touch_id = None;
    }
    
    fn is_active(&self) -> bool {
        self.is_active
    }
}

// GestureManagerは別のファイルに実装します

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_tap_recognizer() {
        let mut recognizer = TapRecognizer::new();
        
        // タップ開始
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
        
        // タップ終了（認識成功）
        let timestamp = 1100;
        let event = InputEvent::new(InputEventType::MouseRelease {
            button: MouseButton::Left,
            x: 105.0,
            y: 105.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Tap);
            assert_eq!(gesture.state, GestureState::Recognized);
            assert_eq!(gesture.position, (105.0, 105.0));
        }
        
        assert!(!recognizer.is_active());
    }
    
    #[test]
    fn test_swipe_recognizer() {
        let mut recognizer = SwipeRecognizer::new();
        
        // スワイプ開始
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
        
        // スワイプ中（距離不足）
        let timestamp = 1050;
        let event = InputEvent::new(InputEventType::MouseMove {
            x: 110.0,
            y: 110.0,
            dx: 10.0,
            dy: 10.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_none());
        
        // スワイプ中（認識開始）
        let timestamp = 1100;
        let event = InputEvent::new(InputEventType::MouseMove {
            x: 160.0,
            y: 160.0,
            dx: 50.0,
            dy: 50.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Swipe);
            assert_eq!(gesture.state, GestureState::Changed);
            assert_eq!(gesture.position, (160.0, 160.0));
            assert_eq!(gesture.start_position, (100.0, 100.0));
            assert_eq!(gesture.delta, (60.0, 60.0));
        }
        
        // スワイプ終了
        let timestamp = 1200;
        let event = InputEvent::new(InputEventType::MouseRelease {
            button: MouseButton::Left,
            x: 180.0,
            y: 180.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        let result = recognizer.update(&event);
        assert!(result.is_some());
        
        if let Some(gesture) = result {
            assert_eq!(gesture.gesture_type, GestureType::Swipe);
            assert_eq!(gesture.state, GestureState::Ended);
            assert_eq!(gesture.position, (180.0, 180.0));
            assert_eq!(gesture.start_position, (100.0, 100.0));
            assert_eq!(gesture.delta, (80.0, 80.0));
            assert_eq!(gesture.swipe_direction, Some(SwipeDirection::DownRight));
        }
        
        assert!(!recognizer.is_active());
    }
} 