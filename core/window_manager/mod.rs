// LumosDesktop ウィンドウマネージャ
// AetherOS 用の高性能ウィンドウマネージャシステム

//! WindowManagerモジュール
//!
//! このモジュールはLumosDesktopの中核となるウィンドウ管理システムを提供します。
//! 一貫性のあるユーザーエクスペリエンスを提供するために、以下の機能を統合しています：
//! 
//! - コンポジター: ウィンドウの合成と描画
//! - シーングラフ: UI要素の階層構造管理
//! - レイアウトエンジン: ウィンドウの配置とワークスペース管理
//! - 入力処理: キーボード・マウス・タッチイベントの処理
//! - ジェスチャー認識: マルチタッチジェスチャー検出
//! - エフェクトパイプライン: 視覚効果の処理

// 各モジュールの公開
pub mod compositor;
pub mod scene_graph;
pub mod layout_engine;
pub mod input_translator;
pub mod gesture_recognizer;
pub mod effects_pipeline;

// 主要コンポーネントの再エクスポート
pub use compositor::wayland_compositor::{LumosCompositor, Window, Rectangle};
pub use scene_graph::scene_graph::{SceneGraph, NodeId, NodeType, Transform, BoundingBox};
pub use layout_engine::layout_manager::{LayoutManager, Workspace, LayoutType};
pub use input_translator::input_manager::{InputManager, InputEvent, KeyModifier};
pub use gesture_recognizer::{GestureRecognizer, GestureType, GestureInfo, GestureState};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// ウィンドウマネージャのメインクラス
/// 各サブシステムを統合し、一貫した管理インターフェースを提供します
pub struct WindowManager {
    // コアコンポーネント
    pub compositor: Arc<Mutex<LumosCompositor>>,
    pub scene_graph: Arc<Mutex<SceneGraph>>,
    pub layout_manager: Arc<Mutex<LayoutManager>>,
    pub input_manager: Arc<Mutex<InputManager>>,
    
    // ジェスチャー認識器のレジストリ
    gesture_recognizers: HashMap<GestureType, Box<dyn GestureRecognizer + Send + Sync>>,
    
    // イベントリスナー
    event_handlers: Vec<Box<dyn Fn(&WindowManagerEvent) -> bool + Send + Sync>>,
    
    // グローバル設定
    config: WindowManagerConfig,
}

/// ウィンドウマネージャの設定
#[derive(Debug, Clone)]
pub struct WindowManagerConfig {
    /// 高速レンダリングモード (バッテリー寿命より優先)
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
        }
    }
}

/// ウィンドウマネージャのイベント
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
    PowerModeChanged,
    SystemStateChanged,
}

impl WindowManager {
    /// 新しいウィンドウマネージャを作成
    pub fn new(config: Option<WindowManagerConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        let compositor = Arc::new(Mutex::new(LumosCompositor::new()));
        let scene_graph = Arc::new(Mutex::new(SceneGraph::new()));
        let layout_manager = Arc::new(Mutex::new(LayoutManager::new()));
        let input_manager = Arc::new(Mutex::new(InputManager::new()));
        
        let mut wm = Self {
            compositor,
            scene_graph,
            layout_manager,
            input_manager,
            gesture_recognizers: HashMap::new(),
            event_handlers: Vec::new(),
            config,
        };
        
        // デフォルトのジェスチャー認識器を登録
        wm.register_default_gesture_recognizers();
        
        // 各コンポーネント間の連携を設定
        wm.setup_inter_component_communication();
        
        wm
    }
    
    /// ウィンドウマネージャを初期化
    pub fn initialize(&mut self) -> Result<(), String> {
        // 各コンポーネントの初期化
        self.compositor.lock().unwrap().initialize()?;
        
        // 初期ワークスペースの設定
        self.setup_initial_workspace();
        
        Ok(())
    }
    
    /// デフォルトのジェスチャー認識器を登録
    fn register_default_gesture_recognizers(&mut self) {
        use gesture_recognizer::tap_recognizer::TapRecognizer;
        use gesture_recognizer::long_press_recognizer::LongPressRecognizer;
        use gesture_recognizer::swipe_recognizer::SwipeRecognizer;
        use gesture_recognizer::pinch_recognizer::PinchRecognizer;
        
        if self.config.enable_gestures {
            self.register_gesture_recognizer(Box::new(TapRecognizer::new()));
            self.register_gesture_recognizer(Box::new(LongPressRecognizer::new()));
            self.register_gesture_recognizer(Box::new(SwipeRecognizer::new()));
            self.register_gesture_recognizer(Box::new(PinchRecognizer::new()));
        }
    }
    
    /// ジェスチャー認識器を登録
    pub fn register_gesture_recognizer(&mut self, recognizer: Box<dyn GestureRecognizer + Send + Sync>) {
        let gesture_type = recognizer.gesture_type();
        self.gesture_recognizers.insert(gesture_type, recognizer);
    }
    
    /// 各コンポーネント間の連携を設定
    fn setup_inter_component_communication(&self) {
        // InputManagerからのイベントをジェスチャー認識器に渡す
        let gesture_recognizers = self.gesture_recognizers.clone();
        let event_handlers = self.event_handlers.clone();
        
        let input_handler = move |event: &InputEvent| {
            // ジェスチャー認識処理
            for (_, recognizer) in gesture_recognizers.iter() {
                if let Some(gesture) = recognizer.update(event) {
                    // ジェスチャーイベントを発行
                    for handler in &event_handlers {
                        if !handler(&WindowManagerEvent::GestureRecognized(gesture.clone())) {
                            break;
                        }
                    }
                }
            }
            true
        };
        
        // ハンドラの登録
        if let Ok(mut input_mgr) = self.input_manager.lock() {
            input_mgr.register_global_handler(Box::new(input_handler));
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
        if let Ok(mut compositor) = self.compositor.lock() {
            compositor.run()?;
        }
        
        Ok(())
    }
    
    /// ウィンドウマネージャを停止
    pub fn shutdown(&mut self) {
        if let Ok(mut compositor) = self.compositor.lock() {
            compositor.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_window_manager_creation() {
        let wm = WindowManager::new(None);
        assert!(wm.config.enable_gestures);
    }
    
    #[test]
    fn test_custom_config() {
        let config = WindowManagerConfig {
            performance_mode: true,
            enable_gestures: false,
            default_layout: LayoutType::Floating,
            ..Default::default()
        };
        
        let wm = WindowManager::new(Some(config));
        assert!(wm.config.performance_mode);
        assert!(!wm.config.enable_gestures);
        assert_eq!(wm.config.default_layout, LayoutType::Floating);
    }
} 