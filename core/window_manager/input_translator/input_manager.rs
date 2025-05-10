// LumosDesktop 入力マネージャー
// ユーザー入力の処理と配布を担当するシステム

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

use crate::core::window_manager::scene_graph::NodeId;

/// キーボードのモディファイア
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyModifier {
    Shift,
    Ctrl,
    Alt,
    Super,
    Hyper,
    Meta,
    CapsLock,
    NumLock,
}

/// キーボードのキーコード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCode(pub u32);

/// キーボードのキーシンボル
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeySym(pub String);

/// マウスボタン
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Extra(u8),
}

/// 入力イベントの種類
#[derive(Debug, Clone)]
pub enum InputEventType {
    KeyPress {
        key_code: KeyCode,
        key_sym: KeySym,
        modifiers: HashSet<KeyModifier>,
        timestamp: u64,
        repeat: bool,
    },
    KeyRelease {
        key_code: KeyCode,
        key_sym: KeySym,
        modifiers: HashSet<KeyModifier>,
        timestamp: u64,
    },
    MousePress {
        button: MouseButton,
        x: f64,
        y: f64,
        modifiers: HashSet<KeyModifier>,
        timestamp: u64,
    },
    MouseRelease {
        button: MouseButton,
        x: f64,
        y: f64,
        modifiers: HashSet<KeyModifier>,
        timestamp: u64,
    },
    MouseMove {
        x: f64,
        y: f64,
        dx: f64,
        dy: f64,
        modifiers: HashSet<KeyModifier>,
        timestamp: u64,
    },
    MouseScroll {
        x: f64,
        y: f64,
        dx: f64,
        dy: f64,
        modifiers: HashSet<KeyModifier>,
        timestamp: u64,
    },
    TouchBegin {
        id: u64,
        x: f64,
        y: f64,
        pressure: f64,
        timestamp: u64,
    },
    TouchUpdate {
        id: u64,
        x: f64,
        y: f64,
        dx: f64,
        dy: f64,
        pressure: f64,
        timestamp: u64,
    },
    TouchEnd {
        id: u64,
        x: f64,
        y: f64,
        timestamp: u64,
    },
    TabletToolProximity {
        x: f64,
        y: f64,
        pressure: f64,
        tilt_x: f64,
        tilt_y: f64,
        rotation: f64,
        timestamp: u64,
    },
    TabletToolTip {
        x: f64,
        y: f64,
        pressure: f64,
        tilt_x: f64,
        tilt_y: f64,
        rotation: f64,
        pressed: bool,
        timestamp: u64,
    },
    TabletToolButton {
        button: u32,
        pressed: bool,
        timestamp: u64,
    },
    FocusIn {
        timestamp: u64,
    },
    FocusOut {
        timestamp: u64,
    },
}

/// 入力イベント
#[derive(Debug, Clone)]
pub struct InputEvent {
    pub target: Option<NodeId>,
    pub event_type: InputEventType,
    pub handled: bool,
    pub propagate: bool,
    pub source_device: Option<String>,
}

impl InputEvent {
    pub fn new(event_type: InputEventType) -> Self {
        Self {
            target: None,
            event_type,
            handled: false,
            propagate: true,
            source_device: None,
        }
    }
    
    pub fn with_target(mut self, target: NodeId) -> Self {
        self.target = Some(target);
        self
    }
    
    pub fn with_source(mut self, source: String) -> Self {
        self.source_device = Some(source);
        self
    }
    
    pub fn mark_handled(&mut self) {
        self.handled = true;
    }
    
    pub fn stop_propagation(&mut self) {
        self.propagate = false;
    }
    
    pub fn timestamp(&self) -> u64 {
        match &self.event_type {
            InputEventType::KeyPress { timestamp, .. } => *timestamp,
            InputEventType::KeyRelease { timestamp, .. } => *timestamp,
            InputEventType::MousePress { timestamp, .. } => *timestamp,
            InputEventType::MouseRelease { timestamp, .. } => *timestamp,
            InputEventType::MouseMove { timestamp, .. } => *timestamp,
            InputEventType::MouseScroll { timestamp, .. } => *timestamp,
            InputEventType::TouchBegin { timestamp, .. } => *timestamp,
            InputEventType::TouchUpdate { timestamp, .. } => *timestamp,
            InputEventType::TouchEnd { timestamp, .. } => *timestamp,
            InputEventType::TabletToolProximity { timestamp, .. } => *timestamp,
            InputEventType::TabletToolTip { timestamp, .. } => *timestamp,
            InputEventType::TabletToolButton { timestamp, .. } => *timestamp,
            InputEventType::FocusIn { timestamp } => *timestamp,
            InputEventType::FocusOut { timestamp } => *timestamp,
        }
    }
    
    pub fn is_key_event(&self) -> bool {
        matches!(
            self.event_type,
            InputEventType::KeyPress { .. } | InputEventType::KeyRelease { .. }
        )
    }
    
    pub fn is_mouse_event(&self) -> bool {
        matches!(
            self.event_type,
            InputEventType::MousePress { .. }
                | InputEventType::MouseRelease { .. }
                | InputEventType::MouseMove { .. }
                | InputEventType::MouseScroll { .. }
        )
    }
    
    pub fn is_touch_event(&self) -> bool {
        matches!(
            self.event_type,
            InputEventType::TouchBegin { .. }
                | InputEventType::TouchUpdate { .. }
                | InputEventType::TouchEnd { .. }
        )
    }
    
    pub fn is_tablet_event(&self) -> bool {
        matches!(
            self.event_type,
            InputEventType::TabletToolProximity { .. }
                | InputEventType::TabletToolTip { .. }
                | InputEventType::TabletToolButton { .. }
        )
    }
    
    pub fn is_focus_event(&self) -> bool {
        matches!(
            self.event_type,
            InputEventType::FocusIn { .. } | InputEventType::FocusOut { .. }
        )
    }
}

/// ショートカットの定義
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortcutDefinition {
    pub key_sym: KeySym,
    pub modifiers: HashSet<KeyModifier>,
    pub description: String,
}

impl ShortcutDefinition {
    pub fn new(
        key_sym: KeySym,
        modifiers: HashSet<KeyModifier>,
        description: String,
    ) -> Self {
        Self {
            key_sym,
            modifiers,
            description,
        }
    }
    
    pub fn matches(&self, key_sym: &KeySym, modifiers: &HashSet<KeyModifier>) -> bool {
        &self.key_sym == key_sym && &self.modifiers == modifiers
    }
}

/// ショートカットアクション
pub type ShortcutAction = Box<dyn Fn() -> bool + Send + Sync>;

/// 入力ハンドラ - 入力イベントを処理する関数
pub type InputHandler = Box<dyn Fn(&InputEvent) -> bool + Send + Sync>;

/// 入力マネージャー - 入力イベントの管理と配送を担当
pub struct InputManager {
    // イベントキュー
    event_queue: VecDeque<InputEvent>,
    
    // 入力ハンドラ
    global_handlers: Vec<InputHandler>,
    node_handlers: HashMap<NodeId, Vec<InputHandler>>,
    
    // キーボード状態
    pressed_keys: HashSet<KeyCode>,
    pressed_modifiers: HashSet<KeyModifier>,
    repeat_info: HashMap<KeyCode, (Instant, Duration)>,
    
    // マウス状態
    mouse_position: (f64, f64),
    pressed_buttons: HashSet<MouseButton>,
    dragging: Option<(MouseButton, NodeId, (f64, f64))>,
    
    // ショートカット
    shortcuts: HashMap<ShortcutDefinition, ShortcutAction>,
    
    // 入力フォーカス
    keyboard_focus: Option<NodeId>,
    mouse_focus: Option<NodeId>,
    
    // タッチ状態
    active_touches: HashMap<u64, (f64, f64)>,
    
    // 入力設定
    key_repeat_delay: Duration,
    key_repeat_interval: Duration,
    double_click_timeout: Duration,
    drag_threshold: f64,
    
    // タイムスタンプ生成用
    start_time: Instant,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            global_handlers: Vec::new(),
            node_handlers: HashMap::new(),
            pressed_keys: HashSet::new(),
            pressed_modifiers: HashSet::new(),
            repeat_info: HashMap::new(),
            mouse_position: (0.0, 0.0),
            pressed_buttons: HashSet::new(),
            dragging: None,
            shortcuts: HashMap::new(),
            keyboard_focus: None,
            mouse_focus: None,
            active_touches: HashMap::new(),
            key_repeat_delay: Duration::from_millis(500),
            key_repeat_interval: Duration::from_millis(50),
            double_click_timeout: Duration::from_millis(500),
            drag_threshold: 5.0,
            start_time: Instant::now(),
        }
    }
    
    /// グローバル入力ハンドラの登録
    pub fn register_global_handler<F>(&mut self, handler: F)
    where
        F: Fn(&InputEvent) -> bool + Send + Sync + 'static,
    {
        self.global_handlers.push(Box::new(handler));
    }
    
    /// ノード固有の入力ハンドラの登録
    pub fn register_node_handler<F>(&mut self, node_id: NodeId, handler: F)
    where
        F: Fn(&InputEvent) -> bool + Send + Sync + 'static,
    {
        self.node_handlers
            .entry(node_id)
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }
    
    /// ノード固有のハンドラを削除
    pub fn remove_node_handlers(&mut self, node_id: NodeId) {
        self.node_handlers.remove(&node_id);
    }
    
    /// ショートカットの登録
    pub fn register_shortcut<F>(
        &mut self,
        definition: ShortcutDefinition,
        action: F,
    ) where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.shortcuts.insert(definition, Box::new(action));
    }
    
    /// ショートカットの削除
    pub fn remove_shortcut(&mut self, definition: &ShortcutDefinition) {
        self.shortcuts.remove(definition);
    }
    
    /// 入力イベントの追加
    pub fn push_event(&mut self, event: InputEvent) {
        self.event_queue.push_back(event);
    }
    
    /// 全ての入力イベントを処理
    pub fn process_events(&mut self) {
        let now = Instant::now();
        
        // キーリピートの処理
        self.process_key_repeats(now);
        
        // イベント処理
        while let Some(event) = self.event_queue.pop_front() {
            self.process_event(event);
        }
    }
    
    /// 単一の入力イベントを処理
    fn process_event(&mut self, mut event: InputEvent) {
        // イベントタイプに基づいて状態を更新
        match &event.event_type {
            InputEventType::KeyPress {
                key_code,
                key_sym,
                modifiers,
                timestamp,
                repeat,
            } => {
                // キー状態の更新
                self.pressed_keys.insert(*key_code);
                self.pressed_modifiers = modifiers.clone();
                
                // リピート情報の更新（リピートイベントでない場合のみ）
                if !repeat {
                    self.repeat_info.insert(
                        *key_code,
                        (
                            Instant::now(),
                            self.key_repeat_delay,
                        ),
                    );
                }
                
                // ショートカットの処理
                if self.process_shortcut(key_sym, modifiers) {
                    event.mark_handled();
                    event.stop_propagation();
                }
                
                // ターゲットが未設定の場合はキーボードフォーカスを設定
                if event.target.is_none() {
                    event.target = self.keyboard_focus;
                }
            }
            InputEventType::KeyRelease {
                key_code,
                key_sym: _,
                modifiers,
                timestamp: _,
            } => {
                // キー状態の更新
                self.pressed_keys.remove(key_code);
                self.pressed_modifiers = modifiers.clone();
                
                // リピート情報の削除
                self.repeat_info.remove(key_code);
                
                // ターゲットが未設定の場合はキーボードフォーカスを設定
                if event.target.is_none() {
                    event.target = self.keyboard_focus;
                }
            }
            InputEventType::MousePress {
                button,
                x,
                y,
                modifiers,
                timestamp: _,
            } => {
                // マウス状態の更新
                self.mouse_position = (*x, *y);
                self.pressed_buttons.insert(*button);
                self.pressed_modifiers = modifiers.clone();
                
                // ドラッグ操作の開始準備
                if let Some(node_id) = self.mouse_focus {
                    self.dragging = Some((*button, node_id, (*x, *y)));
                }
                
                // ターゲットが未設定の場合はマウスフォーカスを設定
                if event.target.is_none() {
                    event.target = self.mouse_focus;
                }
            }
            InputEventType::MouseRelease {
                button,
                x,
                y,
                modifiers,
                timestamp: _,
            } => {
                // マウス状態の更新
                self.mouse_position = (*x, *y);
                self.pressed_buttons.remove(button);
                self.pressed_modifiers = modifiers.clone();
                
                // ドラッグ操作の終了
                if let Some((drag_button, _, _)) = self.dragging {
                    if drag_button == *button {
                        self.dragging = None;
                    }
                }
                
                // ターゲットが未設定の場合はマウスフォーカスを設定
                if event.target.is_none() {
                    event.target = self.mouse_focus;
                }
            }
            InputEventType::MouseMove {
                x,
                y,
                dx: _,
                dy: _,
                modifiers,
                timestamp: _,
            } => {
                // マウス状態の更新
                self.mouse_position = (*x, *y);
                self.pressed_modifiers = modifiers.clone();
                
                // ターゲットが未設定の場合はマウスフォーカスを設定
                if event.target.is_none() {
                    event.target = self.mouse_focus;
                }
            }
            InputEventType::MouseScroll {
                x,
                y,
                dx: _,
                dy: _,
                modifiers,
                timestamp: _,
            } => {
                // マウス状態の更新
                self.mouse_position = (*x, *y);
                self.pressed_modifiers = modifiers.clone();
                
                // ターゲットが未設定の場合はマウスフォーカスを設定
                if event.target.is_none() {
                    event.target = self.mouse_focus;
                }
            }
            InputEventType::TouchBegin {
                id,
                x,
                y,
                pressure: _,
                timestamp: _,
            } => {
                // タッチ状態の更新
                self.active_touches.insert(*id, (*x, *y));
            }
            InputEventType::TouchUpdate {
                id,
                x,
                y,
                dx: _,
                dy: _,
                pressure: _,
                timestamp: _,
            } => {
                // タッチ状態の更新
                self.active_touches.insert(*id, (*x, *y));
            }
            InputEventType::TouchEnd {
                id,
                x: _,
                y: _,
                timestamp: _,
            } => {
                // タッチ状態の更新
                self.active_touches.remove(id);
            }
            InputEventType::FocusIn { timestamp: _ } => {
                // ターゲットをキーボードフォーカスとして設定
                if let Some(target) = event.target {
                    self.keyboard_focus = Some(target);
                }
            }
            InputEventType::FocusOut { timestamp: _ } => {
                // ターゲットがキーボードフォーカスと一致する場合はフォーカスを削除
                if let Some(target) = event.target {
                    if self.keyboard_focus == Some(target) {
                        self.keyboard_focus = None;
                    }
                }
            }
            _ => {}
        }
        
        // イベントハンドラを実行
        self.dispatch_event(&mut event);
    }
    
    /// キーリピートの処理
    fn process_key_repeats(&mut self, now: Instant) {
        let mut repeat_events = Vec::new();
        
        // リピート情報を確認
        for (&key_code, (start_time, delay)) in &mut self.repeat_info {
            let elapsed = now.duration_since(*start_time);
            
            if elapsed >= *delay {
                // リピートイベントの作成
                if let Some(key_sym) = self.key_code_to_sym(&key_code) {
                    let timestamp = self.generate_timestamp();
                    let repeat_event = InputEvent::new(InputEventType::KeyPress {
                        key_code,
                        key_sym: key_sym.clone(),
                        modifiers: self.pressed_modifiers.clone(),
                        timestamp,
                        repeat: true,
                    });
                    
                    repeat_events.push(repeat_event);
                    
                    // 次のリピートのための更新
                    *start_time = now;
                    *delay = self.key_repeat_interval;
                }
            }
        }
        
        // リピートイベントをキューに追加
        for event in repeat_events {
            self.event_queue.push_back(event);
        }
    }
    
    /// ショートカットの処理
    fn process_shortcut(
        &self,
        key_sym: &KeySym,
        modifiers: &HashSet<KeyModifier>,
    ) -> bool {
        for (definition, action) in &self.shortcuts {
            if definition.matches(key_sym, modifiers) {
                return action();
            }
        }
        
        false
    }
    
    /// イベントを適切なハンドラに配送
    fn dispatch_event(&self, event: &mut InputEvent) {
        // グローバルハンドラで処理
        for handler in &self.global_handlers {
            if !handler(event) || !event.propagate {
                return;
            }
        }
        
        // ターゲットが指定されている場合はノード固有のハンドラで処理
        if let Some(target) = event.target {
            if let Some(handlers) = self.node_handlers.get(&target) {
                for handler in handlers {
                    if !handler(event) || !event.propagate {
                        return;
                    }
                }
            }
        }
    }
    
    /// キーボードフォーカスを設定
    pub fn set_keyboard_focus(&mut self, node_id: Option<NodeId>) {
        // 現在のフォーカスと異なる場合のみ処理
        if self.keyboard_focus != node_id {
            // フォーカスアウトイベントの作成
            if let Some(old_focus) = self.keyboard_focus {
                let timestamp = self.generate_timestamp();
                let focus_out_event = InputEvent::new(InputEventType::FocusOut { timestamp })
                    .with_target(old_focus);
                
                self.event_queue.push_back(focus_out_event);
            }
            
            // フォーカスインイベントの作成
            if let Some(new_focus) = node_id {
                let timestamp = self.generate_timestamp();
                let focus_in_event = InputEvent::new(InputEventType::FocusIn { timestamp })
                    .with_target(new_focus);
                
                self.event_queue.push_back(focus_in_event);
            }
            
            self.keyboard_focus = node_id;
        }
    }
    
    /// マウスフォーカスを設定
    pub fn set_mouse_focus(&mut self, node_id: Option<NodeId>) {
        self.mouse_focus = node_id;
    }
    
    /// タイムスタンプを生成
    fn generate_timestamp(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
    
    /// キーコードからキーシンボルを取得（実際の実装では適切なマッピング）
    fn key_code_to_sym(&self, key_code: &KeyCode) -> Option<KeySym> {
        // 実際の実装ではXKBなどを使用してマッピング
        // ここでは簡略化のため単純な変換
        Some(KeySym(format!("KEY_{}", key_code.0)))
    }
    
    /// 現在押されているキーの取得
    pub fn get_pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys
    }
    
    /// 現在押されているモディファイアの取得
    pub fn get_pressed_modifiers(&self) -> &HashSet<KeyModifier> {
        &self.pressed_modifiers
    }
    
    /// 現在のマウス位置の取得
    pub fn get_mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }
    
    /// 現在押されているマウスボタンの取得
    pub fn get_pressed_buttons(&self) -> &HashSet<MouseButton> {
        &self.pressed_buttons
    }
    
    /// 現在のキーボードフォーカスの取得
    pub fn get_keyboard_focus(&self) -> Option<NodeId> {
        self.keyboard_focus
    }
    
    /// 現在のマウスフォーカスの取得
    pub fn get_mouse_focus(&self) -> Option<NodeId> {
        self.mouse_focus
    }
    
    /// アクティブなタッチの取得
    pub fn get_active_touches(&self) -> &HashMap<u64, (f64, f64)> {
        &self.active_touches
    }
    
    /// ドラッグ操作の取得
    pub fn get_dragging(&self) -> Option<(MouseButton, NodeId, (f64, f64))> {
        self.dragging
    }
    
    /// キーリピート遅延の設定
    pub fn set_key_repeat_delay(&mut self, delay: Duration) {
        self.key_repeat_delay = delay;
    }
    
    /// キーリピート間隔の設定
    pub fn set_key_repeat_interval(&mut self, interval: Duration) {
        self.key_repeat_interval = interval;
    }
    
    /// ダブルクリックタイムアウトの設定
    pub fn set_double_click_timeout(&mut self, timeout: Duration) {
        self.double_click_timeout = timeout;
    }
    
    /// ドラッグしきい値の設定
    pub fn set_drag_threshold(&mut self, threshold: f64) {
        self.drag_threshold = threshold;
    }
}

/// キー情報ヘルパー - わかりやすいキー名
pub struct KeyInfo;

impl KeyInfo {
    pub fn key_code(name: &str) -> Option<KeyCode> {
        // 実際の実装ではより包括的なマッピング
        match name.to_uppercase().as_str() {
            "A" => Some(KeyCode(0x41)),
            "B" => Some(KeyCode(0x42)),
            "C" => Some(KeyCode(0x43)),
            "D" => Some(KeyCode(0x44)),
            "E" => Some(KeyCode(0x45)),
            "F" => Some(KeyCode(0x46)),
            "G" => Some(KeyCode(0x47)),
            "H" => Some(KeyCode(0x48)),
            "I" => Some(KeyCode(0x49)),
            "J" => Some(KeyCode(0x4A)),
            "K" => Some(KeyCode(0x4B)),
            "L" => Some(KeyCode(0x4C)),
            "M" => Some(KeyCode(0x4D)),
            "N" => Some(KeyCode(0x4E)),
            "O" => Some(KeyCode(0x4F)),
            "P" => Some(KeyCode(0x50)),
            "Q" => Some(KeyCode(0x51)),
            "R" => Some(KeyCode(0x52)),
            "S" => Some(KeyCode(0x53)),
            "T" => Some(KeyCode(0x54)),
            "U" => Some(KeyCode(0x55)),
            "V" => Some(KeyCode(0x56)),
            "W" => Some(KeyCode(0x57)),
            "X" => Some(KeyCode(0x58)),
            "Y" => Some(KeyCode(0x59)),
            "Z" => Some(KeyCode(0x5A)),
            "0" => Some(KeyCode(0x30)),
            "1" => Some(KeyCode(0x31)),
            "2" => Some(KeyCode(0x32)),
            "3" => Some(KeyCode(0x33)),
            "4" => Some(KeyCode(0x34)),
            "5" => Some(KeyCode(0x35)),
            "6" => Some(KeyCode(0x36)),
            "7" => Some(KeyCode(0x37)),
            "8" => Some(KeyCode(0x38)),
            "9" => Some(KeyCode(0x39)),
            "SPACE" => Some(KeyCode(0x20)),
            "ENTER" => Some(KeyCode(0x0D)),
            "ESCAPE" => Some(KeyCode(0x1B)),
            "TAB" => Some(KeyCode(0x09)),
            "BACKSPACE" => Some(KeyCode(0x08)),
            "LEFT" => Some(KeyCode(0x25)),
            "RIGHT" => Some(KeyCode(0x27)),
            "UP" => Some(KeyCode(0x26)),
            "DOWN" => Some(KeyCode(0x28)),
            _ => None,
        }
    }
    
    pub fn key_sym(name: &str) -> Option<KeySym> {
        // 実際の実装ではより包括的なマッピング
        match name.to_uppercase().as_str() {
            "A" => Some(KeySym("a".to_string())),
            "B" => Some(KeySym("b".to_string())),
            "C" => Some(KeySym("c".to_string())),
            "D" => Some(KeySym("d".to_string())),
            "E" => Some(KeySym("e".to_string())),
            "F" => Some(KeySym("f".to_string())),
            "G" => Some(KeySym("g".to_string())),
            "H" => Some(KeySym("h".to_string())),
            "I" => Some(KeySym("i".to_string())),
            "J" => Some(KeySym("j".to_string())),
            "K" => Some(KeySym("k".to_string())),
            "L" => Some(KeySym("l".to_string())),
            "M" => Some(KeySym("m".to_string())),
            "N" => Some(KeySym("n".to_string())),
            "O" => Some(KeySym("o".to_string())),
            "P" => Some(KeySym("p".to_string())),
            "Q" => Some(KeySym("q".to_string())),
            "R" => Some(KeySym("r".to_string())),
            "S" => Some(KeySym("s".to_string())),
            "T" => Some(KeySym("t".to_string())),
            "U" => Some(KeySym("u".to_string())),
            "V" => Some(KeySym("v".to_string())),
            "W" => Some(KeySym("w".to_string())),
            "X" => Some(KeySym("x".to_string())),
            "Y" => Some(KeySym("y".to_string())),
            "Z" => Some(KeySym("z".to_string())),
            "0" => Some(KeySym("0".to_string())),
            "1" => Some(KeySym("1".to_string())),
            "2" => Some(KeySym("2".to_string())),
            "3" => Some(KeySym("3".to_string())),
            "4" => Some(KeySym("4".to_string())),
            "5" => Some(KeySym("5".to_string())),
            "6" => Some(KeySym("6".to_string())),
            "7" => Some(KeySym("7".to_string())),
            "8" => Some(KeySym("8".to_string())),
            "9" => Some(KeySym("9".to_string())),
            "SPACE" => Some(KeySym("space".to_string())),
            "ENTER" => Some(KeySym("Return".to_string())),
            "ESCAPE" => Some(KeySym("Escape".to_string())),
            "TAB" => Some(KeySym("Tab".to_string())),
            "BACKSPACE" => Some(KeySym("BackSpace".to_string())),
            "LEFT" => Some(KeySym("Left".to_string())),
            "RIGHT" => Some(KeySym("Right".to_string())),
            "UP" => Some(KeySym("Up".to_string())),
            "DOWN" => Some(KeySym("Down".to_string())),
            _ => None,
        }
    }
    
    pub fn modifier(name: &str) -> Option<KeyModifier> {
        match name.to_uppercase().as_str() {
            "SHIFT" => Some(KeyModifier::Shift),
            "CTRL" | "CONTROL" => Some(KeyModifier::Ctrl),
            "ALT" => Some(KeyModifier::Alt),
            "SUPER" | "WIN" | "WINDOWS" | "CMD" | "COMMAND" => Some(KeyModifier::Super),
            "HYPER" => Some(KeyModifier::Hyper),
            "META" => Some(KeyModifier::Meta),
            "CAPSLOCK" | "CAPS" => Some(KeyModifier::CapsLock),
            "NUMLOCK" | "NUM" => Some(KeyModifier::NumLock),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_manager_creation() {
        let manager = InputManager::new();
        assert!(manager.get_pressed_keys().is_empty());
        assert!(manager.get_pressed_modifiers().is_empty());
        assert_eq!(manager.get_mouse_position(), (0.0, 0.0));
    }
    
    #[test]
    fn test_keyboard_focus() {
        let mut manager = InputManager::new();
        let node_id = NodeId(1);
        
        assert_eq!(manager.get_keyboard_focus(), None);
        
        manager.set_keyboard_focus(Some(node_id));
        assert_eq!(manager.get_keyboard_focus(), Some(node_id));
        
        manager.set_keyboard_focus(None);
        assert_eq!(manager.get_keyboard_focus(), None);
    }
    
    #[test]
    fn test_key_press_and_release() {
        let mut manager = InputManager::new();
        let key_code = KeyCode(0x41); // 'A' キー
        let key_sym = KeySym("a".to_string());
        let modifiers = HashSet::new();
        
        // キー押下イベント
        let timestamp = 1000;
        let key_press = InputEvent::new(InputEventType::KeyPress {
            key_code,
            key_sym: key_sym.clone(),
            modifiers: modifiers.clone(),
            timestamp,
            repeat: false,
        });
        
        manager.process_event(key_press);
        
        // キー状態の確認
        assert!(manager.get_pressed_keys().contains(&key_code));
        
        // キー解放イベント
        let timestamp = 1100;
        let key_release = InputEvent::new(InputEventType::KeyRelease {
            key_code,
            key_sym,
            modifiers,
            timestamp,
        });
        
        manager.process_event(key_release);
        
        // キー状態の確認
        assert!(!manager.get_pressed_keys().contains(&key_code));
    }
    
    #[test]
    fn test_mouse_movement() {
        let mut manager = InputManager::new();
        
        // マウス移動イベント
        let timestamp = 1000;
        let mouse_move = InputEvent::new(InputEventType::MouseMove {
            x: 100.0,
            y: 200.0,
            dx: 10.0,
            dy: 20.0,
            modifiers: HashSet::new(),
            timestamp,
        });
        
        manager.process_event(mouse_move);
        
        // マウス位置の確認
        assert_eq!(manager.get_mouse_position(), (100.0, 200.0));
    }
    
    #[test]
    fn test_shortcut_registration() {
        let mut manager = InputManager::new();
        
        // ショートカットの定義
        let mut modifiers = HashSet::new();
        modifiers.insert(KeyModifier::Ctrl);
        
        let shortcut = ShortcutDefinition::new(
            KeySym("c".to_string()),
            modifiers.clone(),
            "Copy".to_string(),
        );
        
        // アクションの作成とショートカットの登録
        let mut action_called = false;
        {
            let action = move || {
                action_called = true;
                true
            };
            
            manager.register_shortcut(shortcut.clone(), action);
        }
        
        // ショートカットイベントの作成と処理
        let key_code = KeyCode(0x43); // 'C' キー
        let key_sym = KeySym("c".to_string());
        
        let timestamp = 1000;
        let key_press = InputEvent::new(InputEventType::KeyPress {
            key_code,
            key_sym,
            modifiers,
            timestamp,
            repeat: false,
        });
        
        manager.process_event(key_press);
        
        // アクションが呼ばれたかの確認
        assert!(action_called);
    }
} 