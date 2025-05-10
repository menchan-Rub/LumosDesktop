// LumosDesktop ウィンドウマネージャの実装
// AetherOS 用の高性能ウィンドウマネージャシステム

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use super::compositor::wayland_compositor::{LumosCompositor, Window, Rectangle, CompositorEvent};
use super::scene_graph::scene_graph::{SceneGraph, NodeId, NodeType, Transform, BoundingBox};
use super::layout_engine::layout_manager::{LayoutManager, LayoutWindow, Workspace, LayoutType, LayoutEvent, WorkspaceRect};
use super::input_translator::input_manager::{InputManager, InputEvent, KeyModifier, ShortcutDefinition};
use super::gesture_recognizer::{
    GestureManager, GestureRecognizer, GestureType, GestureState, GestureInfo, SwipeDirection
};
use super::effects_pipeline::effects_manager::{EffectsManager, EffectType, TransitionEffect};

/// ウィンドウマネージャの構成設定
#[derive(Debug, Clone)]
pub struct WindowManagerConfig {
    /// パフォーマンスモード
    pub performance_mode: bool,
    
    /// 自動ワークスペース管理
    pub auto_workspace_management: bool,
    
    /// 視覚効果の有効化
    pub enable_effects: bool,
    
    /// マルチタッチジェスチャーの有効化
    pub enable_gestures: bool,
    
    /// ウィンドウスナップの有効化
    pub enable_window_snapping: bool,
    
    /// デフォルトレイアウトタイプ
    pub default_layout: LayoutType,
    
    /// 更新レート (Hz)
    pub update_rate: u32,
    
    /// 垂直同期の有効化
    pub enable_vsync: bool,
    
    /// トリプルバッファリングの有効化
    pub enable_triple_buffering: bool,
    
    /// 省電力モードの有効化
    pub enable_power_saving: bool,
    
    /// アニメーション期間 (ミリ秒)
    pub animation_duration_ms: u32,
}

impl Default for WindowManagerConfig {
    fn default() -> Self {
        Self {
            performance_mode: false,
            auto_workspace_management: true,
            enable_effects: true,
            enable_gestures: true,
            enable_window_snapping: true,
            default_layout: LayoutType::Tiling,
            update_rate: 60,
            enable_vsync: true,
            enable_triple_buffering: true,
            enable_power_saving: true,
            animation_duration_ms: 250,
        }
    }
}

/// ウィンドウマネージャイベント
#[derive(Debug, Clone)]
pub enum WindowManagerEvent {
    /// ウィンドウ関連イベント
    WindowCreated(NodeId),
    WindowDestroyed(NodeId),
    WindowFocused(NodeId),
    WindowMoved(NodeId),
    WindowResized(NodeId),
    WindowStateChanged(NodeId),
    
    /// ワークスペース関連イベント
    WorkspaceCreated(usize),
    WorkspaceRemoved(usize),
    WorkspaceSwitched(usize),
    
    /// ジェスチャー関連イベント
    GestureRecognized(GestureInfo),
    
    /// システム関連イベント
    DisplayConfigChanged,
    PowerModeChanged(bool), // true=省電力モード
    SystemStateChanged(SystemState),
}

/// システム状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemState {
    Starting,
    Running,
    Suspending,
    Resuming,
    ShuttingDown,
}

/// ウィンドウマネージャ - コアコンポーネント
pub struct WindowManager {
    // コアコンポーネント
    compositor: Arc<Mutex<LumosCompositor>>,
    scene_graph: Arc<RwLock<SceneGraph>>,
    layout_manager: Arc<Mutex<LayoutManager>>,
    input_manager: Arc<Mutex<InputManager>>,
    gesture_manager: Arc<Mutex<GestureManager>>,
    effects_manager: Arc<Mutex<EffectsManager>>,
    
    // イベントハンドラ
    event_handlers: Vec<Box<dyn Fn(&WindowManagerEvent) -> bool + Send + Sync>>,
    
    // 設定
    config: WindowManagerConfig,
    
    // 状態管理
    running: bool,
    current_state: SystemState,
    last_update: Instant,
    
    // ノードマッピング
    window_to_node: HashMap<u64, NodeId>,
    node_to_window: HashMap<NodeId, u64>,
    
    // ホットキー設定
    key_bindings: HashMap<ShortcutDefinition, Box<dyn Fn() -> bool + Send + Sync>>,
}

impl WindowManager {
    /// 新しいウィンドウマネージャを作成
    pub fn new(config: Option<WindowManagerConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        let compositor = Arc::new(Mutex::new(LumosCompositor::new()));
        let scene_graph = Arc::new(RwLock::new(SceneGraph::new()));
        let layout_manager = Arc::new(Mutex::new(LayoutManager::new()));
        let input_manager = Arc::new(Mutex::new(InputManager::new()));
        let gesture_manager = Arc::new(Mutex::new(GestureManager::new()));
        let effects_manager = Arc::new(Mutex::new(EffectsManager::new()));
        
        let mut wm = Self {
            compositor,
            scene_graph,
            layout_manager,
            input_manager,
            gesture_manager,
            effects_manager,
            event_handlers: Vec::new(),
            config,
            running: false,
            current_state: SystemState::Starting,
            last_update: Instant::now(),
            window_to_node: HashMap::new(),
            node_to_window: HashMap::new(),
            key_bindings: HashMap::new(),
        };
        
        // 各コンポーネント間の連携を設定
        wm.setup_inter_component_communication();
        
        // ジェスチャー認識器を登録
        if wm.config.enable_gestures {
            wm.setup_gesture_recognizers();
        }
        
        // デフォルトホットキーの設定
        wm.setup_default_key_bindings();
        
        wm
    }
    
    /// ウィンドウマネージャを初期化
    pub fn initialize(&mut self) -> Result<(), String> {
        // コンポジターの初期化
        if let Ok(mut compositor) = self.compositor.lock() {
            compositor.initialize()?;
        } else {
            return Err("コンポジターのロックに失敗しました".to_string());
        }
        
        // 初期ワークスペースの設定
        self.setup_initial_workspace();
        
        // 状態の更新
        self.current_state = SystemState::Running;
        
        Ok(())
    }
    
    /// コンポーネント間の連携を設定
    fn setup_inter_component_communication(&mut self) {
        // コンポジターイベントのハンドラ
        let wm_event_handlers = self.event_handlers.clone();
        let scene_graph = self.scene_graph.clone();
        let layout_manager = self.layout_manager.clone();
        let window_to_node = Arc::new(Mutex::new(self.window_to_node.clone()));
        let node_to_window = Arc::new(Mutex::new(self.node_to_window.clone()));
        
        let compositor_handler = move |event: &CompositorEvent| {
            match event {
                CompositorEvent::WindowCreated(window_id) => {
                    // シーングラフにノードを作成
                    if let Ok(mut sg) = scene_graph.write() {
                        let root_id = sg.root().borrow().id;
                        if let Ok(node_id) = sg.create_node(
                            root_id,
                            NodeType::Window,
                            format!("window_{}", window_id),
                        ) {
                            // マッピングの更新
                            if let Ok(mut w2n) = window_to_node.lock() {
                                w2n.insert(*window_id, node_id);
                            }
                            if let Ok(mut n2w) = node_to_window.lock() {
                                n2w.insert(node_id, *window_id);
                            }
                            
                            // WindowManagerイベントを発行
                            for handler in &wm_event_handlers {
                                if !handler(&WindowManagerEvent::WindowCreated(node_id)) {
                                    break;
                                }
                            }
                        }
                    }
                }
                CompositorEvent::WindowDestroyed(window_id) => {
                    // マッピングから取得
                    if let Ok(w2n) = window_to_node.lock() {
                        if let Some(node_id) = w2n.get(window_id) {
                            // シーングラフからノードを削除
                            if let Ok(mut sg) = scene_graph.write() {
                                let _ = sg.remove_node(*node_id);
                            }
                            
                            // WindowManagerイベントを発行
                            for handler in &wm_event_handlers {
                                if !handler(&WindowManagerEvent::WindowDestroyed(*node_id)) {
                                    break;
                                }
                            }
                        }
                    }
                }
                // 他のイベントも同様に処理
                _ => {}
            }
            true
        };
        
        // コンポジターハンドラの登録
        if let Ok(mut compositor) = self.compositor.lock() {
            compositor.add_event_handler(compositor_handler);
        }
        
        // 入力イベントとジェスチャー認識器の連携
        let gesture_manager = self.gesture_manager.clone();
        let wm_event_handlers = self.event_handlers.clone();
        
        let input_handler = move |event: &InputEvent| {
            // ジェスチャー認識処理
            if let Ok(mut gm) = gesture_manager.lock() {
                let gestures = gm.process_event(event);
                
                // 認識されたジェスチャーをイベントとして発行
                for gesture in gestures {
                    for handler in &wm_event_handlers {
                        if !handler(&WindowManagerEvent::GestureRecognized(gesture.clone())) {
                            break;
                        }
                    }
                }
            }
            
            true
        };
        
        // 入力ハンドラの登録
        if let Ok(mut input_mgr) = self.input_manager.lock() {
            input_mgr.register_global_handler(Box::new(input_handler));
        }
    }
    
    /// ジェスチャー認識器の設定
    fn setup_gesture_recognizers(&mut self) {
        if let Ok(mut gm) = self.gesture_manager.lock() {
            gm.register_default_recognizers();
        }
    }
    
    /// デフォルトのホットキー設定
    fn setup_default_key_bindings(&mut self) {
        use super::input_translator::input_manager::KeyInfo;
        
        // Alt+Tab: ウィンドウ切り替え
        let layout_manager = self.layout_manager.clone();
        let alt_tab = ShortcutDefinition::new(
            KeyInfo::key_sym("TAB").unwrap(),
            {
                let mut modifiers = std::collections::HashSet::new();
                modifiers.insert(KeyModifier::Alt);
                modifiers
            },
            "ウィンドウ切り替え".to_string(),
        );
        
        self.key_bindings.insert(alt_tab, Box::new(move || {
            if let Ok(mut lm) = layout_manager.lock() {
                if let Some(workspace) = lm.current_workspace_mut() {
                    let windows = workspace.windows.clone();
                    if windows.len() > 1 {
                        if let Some(active) = workspace.active_window {
                            let pos = windows.iter().position(|&w| w == active).unwrap_or(0);
                            let next_pos = (pos + 1) % windows.len();
                            let next_window = windows[next_pos];
                            workspace.set_active_window(next_window);
                            return true;
                        }
                    }
                }
            }
            false
        }));
        
        // Super+D: 全てのウィンドウを最小化 (Show Desktop)
        let layout_manager = self.layout_manager.clone();
        let super_d = ShortcutDefinition::new(
            KeyInfo::key_sym("D").unwrap(),
            {
                let mut modifiers = std::collections::HashSet::new();
                modifiers.insert(KeyModifier::Super);
                modifiers
            },
            "デスクトップを表示".to_string(),
        );
        
        self.key_bindings.insert(super_d, Box::new(move || {
            if let Ok(mut lm) = layout_manager.lock() {
                if let Some(workspace) = lm.current_workspace() {
                    for window_id in &workspace.windows {
                        if let Some(window) = lm.get_window_mut(*window_id) {
                            window.minimized = true;
                        }
                    }
                    return true;
                }
            }
            false
        }));
        
        // Super+左右矢印: ウィンドウをスナップ
        let layout_manager = self.layout_manager.clone();
        let super_left = ShortcutDefinition::new(
            KeyInfo::key_sym("LEFT").unwrap(),
            {
                let mut modifiers = std::collections::HashSet::new();
                modifiers.insert(KeyModifier::Super);
                modifiers
            },
            "ウィンドウを左にスナップ".to_string(),
        );
        
        self.key_bindings.insert(super_left, Box::new(move || {
            if let Ok(mut lm) = layout_manager.lock() {
                if let Some(workspace) = lm.current_workspace() {
                    if let Some(active_window) = workspace.active_window {
                        if let Some(window) = lm.get_window_mut(active_window) {
                            // 画面の左半分にスナップ
                            let workspace_rect = workspace.rect;
                            let new_rect = WorkspaceRect::new(
                                workspace_rect.x,
                                workspace_rect.y,
                                workspace_rect.width / 2,
                                workspace_rect.height,
                            );
                            window.rect = new_rect;
                            return true;
                        }
                    }
                }
            }
            false
        }));
        
        // ホットキーを入力マネージャに登録
        if let Ok(mut input_mgr) = self.input_manager.lock() {
            for (shortcut, action) in &self.key_bindings {
                let action = action.clone();
                input_mgr.register_shortcut(shortcut.clone(), move || action());
            }
        }
    }
    
    /// 初期ワークスペースの設定
    fn setup_initial_workspace(&self) {
        if let Ok(mut layout_mgr) = self.layout_manager.lock() {
            // デフォルトワークスペースのレイアウトタイプを設定
            if let Some(workspace) = layout_mgr.get_workspace_mut(0) {
                workspace.layout_type = self.config.default_layout;
            }
        }
    }
    
    /// イベントリスナーの登録
    pub fn add_event_listener<F>(&mut self, listener: F)
    where
        F: Fn(&WindowManagerEvent) -> bool + Send + Sync + 'static,
    {
        self.event_handlers.push(Box::new(listener));
    }
    
    /// ウィンドウマネージャのメインループを実行
    pub fn run(&mut self) -> Result<(), String> {
        self.running = true;
        self.current_state = SystemState::Running;
        
        // 更新間隔の計算
        let update_interval = Duration::from_secs_f64(1.0 / self.config.update_rate as f64);
        
        while self.running {
            let loop_start = Instant::now();
            
            // 入力イベントの処理
            if let Ok(mut input_mgr) = self.input_manager.lock() {
                input_mgr.process_events();
            }
            
            // レイアウトの更新
            if let Ok(mut layout_mgr) = self.layout_manager.lock() {
                layout_mgr.update_layout(layout_mgr.current_workspace_id());
            }
            
            // コンポジターの更新
            if let Ok(mut compositor) = self.compositor.lock() {
                compositor.render_frame();
            }
            
            // 次の更新まで待機
            let elapsed = loop_start.elapsed();
            if elapsed < update_interval {
                thread::sleep(update_interval - elapsed);
            }
            
            self.last_update = Instant::now();
        }
        
        Ok(())
    }
    
    /// ウィンドウマネージャを停止
    pub fn shutdown(&mut self) {
        self.running = false;
        self.current_state = SystemState::ShuttingDown;
        
        if let Ok(mut compositor) = self.compositor.lock() {
            compositor.stop();
        }
    }
    
    /// レイアウトのタイプを変更
    pub fn set_layout_type(&mut self, layout_type: LayoutType) -> Result<(), String> {
        if let Ok(mut layout_mgr) = self.layout_manager.lock() {
            let current_workspace = layout_mgr.current_workspace_id();
            layout_mgr.set_layout_type(current_workspace, layout_type)
        } else {
            Err("レイアウトマネージャのロックに失敗しました".to_string())
        }
    }
    
    /// 新しいワークスペースを作成
    pub fn create_workspace(&mut self, name: &str) -> Result<usize, String> {
        if let Ok(mut layout_mgr) = self.layout_manager.lock() {
            let id = layout_mgr.workspaces.len();
            let workspace = Workspace::new(
                id,
                name.to_string(),
                WorkspaceRect::new(0, 0, 1920, 1080), // デフォルトサイズ
            );
            
            layout_mgr.add_workspace(workspace)?;
            
            // イベント発行
            for handler in &self.event_handlers {
                if !handler(&WindowManagerEvent::WorkspaceCreated(id)) {
                    break;
                }
            }
            
            Ok(id)
        } else {
            Err("レイアウトマネージャのロックに失敗しました".to_string())
        }
    }
    
    /// ワークスペースを切り替え
    pub fn switch_workspace(&mut self, workspace_id: usize) -> Result<(), String> {
        if let Ok(mut layout_mgr) = self.layout_manager.lock() {
            layout_mgr.switch_workspace(workspace_id)?;
            
            // イベント発行
            for handler in &self.event_handlers {
                if !handler(&WindowManagerEvent::WorkspaceSwitched(workspace_id)) {
                    break;
                }
            }
            
            Ok(())
        } else {
            Err("レイアウトマネージャのロックに失敗しました".to_string())
        }
    }
    
    /// ウィンドウをワークスペース間で移動
    pub fn move_window_to_workspace(&mut self, window_id: NodeId, workspace_id: usize) -> Result<(), String> {
        if let Ok(mut layout_mgr) = self.layout_manager.lock() {
            layout_mgr.move_window_to_workspace(window_id, workspace_id)
        } else {
            Err("レイアウトマネージャのロックに失敗しました".to_string())
        }
    }
    
    /// エフェクトを適用
    pub fn apply_effect(&mut self, effect_type: EffectType, target: Option<NodeId>, duration_ms: Option<u32>) -> Result<(), String> {
        let duration = duration_ms.unwrap_or(self.config.animation_duration_ms);
        
        if let Ok(mut effects_mgr) = self.effects_manager.lock() {
            let effect = TransitionEffect::new(effect_type, duration);
            effects_mgr.add_effect(effect, target)
        } else {
            Err("エフェクトマネージャのロックに失敗しました".to_string())
        }
    }
    
    /// ノードIDからウィンドウIDを取得
    pub fn get_window_id(&self, node_id: NodeId) -> Option<u64> {
        self.node_to_window.get(&node_id).copied()
    }
    
    /// ウィンドウIDからノードIDを取得
    pub fn get_node_id(&self, window_id: u64) -> Option<NodeId> {
        self.window_to_node.get(&window_id).copied()
    }
    
    /// 現在のFPSを取得
    pub fn get_fps(&self) -> f64 {
        if let Ok(compositor) = self.compositor.lock() {
            compositor.get_fps()
        } else {
            0.0
        }
    }
    
    /// 省電力モードを設定
    pub fn set_power_saving_mode(&mut self, enabled: bool) {
        self.config.enable_power_saving = enabled;
        
        // 省電力モードに応じた設定変更
        if enabled {
            // 更新レートを下げる
            self.config.update_rate = 30;
            
            // エフェクトを無効化
            self.config.enable_effects = false;
        } else {
            // 標準設定に戻す
            self.config.update_rate = 60;
            self.config.enable_effects = true;
        }
        
        // イベント発行
        for handler in &self.event_handlers {
            if !handler(&WindowManagerEvent::PowerModeChanged(enabled)) {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_window_manager_creation() {
        let wm = WindowManager::new(None);
        assert!(!wm.config.performance_mode);
        assert!(wm.config.enable_gestures);
        assert!(wm.config.enable_effects);
        assert_eq!(wm.current_state, SystemState::Starting);
    }
    
    #[test]
    fn test_custom_config() {
        let config = WindowManagerConfig {
            performance_mode: true,
            enable_gestures: false,
            enable_effects: false,
            default_layout: LayoutType::Floating,
            update_rate: 120,
            ..Default::default()
        };
        
        let wm = WindowManager::new(Some(config));
        assert!(wm.config.performance_mode);
        assert!(!wm.config.enable_gestures);
        assert!(!wm.config.enable_effects);
        assert_eq!(wm.config.default_layout, LayoutType::Floating);
        assert_eq!(wm.config.update_rate, 120);
    }
} 