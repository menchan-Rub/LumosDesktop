// LumosDesktop レイアウトマネージャー
// ウィンドウとワークスペースの自動配置を管理する

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;

use crate::core::window_manager::scene_graph::{NodeId, Transform, BoundingBox};

/// レイアウトタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutType {
    Floating,         // 自由配置
    Tiling,           // タイル配置
    Stacking,         // 重ね配置
    Tabbed,           // タブ配置
    Grid,             // グリッド配置
    Maximized,        // 最大化
    HorizontalSplit,  // 水平分割
    VerticalSplit,    // 垂直分割
    Cascade,          // カスケード配置
    Custom,           // カスタム配置
}

/// レイアウト方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

/// スプリットレイアウトの分割比率
#[derive(Debug, Clone)]
pub struct SplitRatio {
    pub values: Vec<f32>,  // 各分割位置の比率（0.0〜1.0）
}

impl SplitRatio {
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }
    
    pub fn even(count: usize) -> Self {
        if count <= 1 {
            return Self { values: vec![] };
        }
        
        let step = 1.0 / count as f32;
        let mut values = Vec::with_capacity(count - 1);
        
        for i in 1..count {
            values.push(step * i as f32);
        }
        
        Self { values }
    }
    
    pub fn get_section_size(&self, workspace_size: f32, section_index: usize) -> f32 {
        if self.values.is_empty() {
            return workspace_size;
        }
        
        let start_ratio = if section_index == 0 {
            0.0
        } else {
            self.values[section_index - 1]
        };
        
        let end_ratio = if section_index >= self.values.len() {
            1.0
        } else {
            self.values[section_index]
        };
        
        workspace_size * (end_ratio - start_ratio)
    }
}

/// ワークスペース領域
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl WorkspaceRect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width as i32 &&
        y >= self.y && y < self.y + self.height as i32
    }
    
    pub fn intersects(&self, other: &WorkspaceRect) -> bool {
        self.x < other.x + other.width as i32 &&
        self.x + self.width as i32 > other.x &&
        self.y < other.y + other.height as i32 &&
        self.y + self.height as i32 > other.y
    }
    
    pub fn center(&self) -> (i32, i32) {
        (
            self.x + (self.width / 2) as i32,
            self.y + (self.height / 2) as i32,
        )
    }
    
    pub fn to_bounding_box(&self) -> BoundingBox {
        BoundingBox::new(
            (self.x as f32, self.y as f32, 0.0),
            ((self.x + self.width as i32) as f32, (self.y + self.height as i32) as f32, 0.0),
        )
    }
}

/// ウィンドウ情報
#[derive(Debug, Clone)]
pub struct LayoutWindow {
    pub id: NodeId,
    pub rect: WorkspaceRect,
    pub min_size: (u32, u32),
    pub max_size: Option<(u32, u32)>,
    pub floating: bool,
    pub fullscreen: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub z_order: i32,
    pub snap_edges: HashSet<SnapEdge>,
    pub tags: HashSet<String>,
}

/// スナップエッジ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SnapEdge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// ワークスペース情報
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: usize,
    pub name: String,
    pub rect: WorkspaceRect,
    pub layout_type: LayoutType,
    pub windows: Vec<NodeId>,
    pub active_window: Option<NodeId>,
    pub split_ratios: HashMap<LayoutDirection, SplitRatio>,
    pub tags: HashSet<String>,
    pub custom_data: HashMap<String, String>,
}

impl Workspace {
    pub fn new(id: usize, name: String, rect: WorkspaceRect) -> Self {
        Self {
            id,
            name,
            rect,
            layout_type: LayoutType::Floating,
            windows: Vec::new(),
            active_window: None,
            split_ratios: HashMap::new(),
            tags: HashSet::new(),
            custom_data: HashMap::new(),
        }
    }
    
    pub fn add_window(&mut self, window_id: NodeId) {
        if !self.windows.contains(&window_id) {
            self.windows.push(window_id);
        }
    }
    
    pub fn remove_window(&mut self, window_id: NodeId) -> bool {
        let pos = self.windows.iter().position(|id| *id == window_id);
        if let Some(index) = pos {
            self.windows.remove(index);
            
            // アクティブウィンドウを更新
            if let Some(active_id) = self.active_window {
                if active_id == window_id {
                    self.active_window = self.windows.last().copied();
                }
            }
            
            true
        } else {
            false
        }
    }
    
    pub fn set_active_window(&mut self, window_id: NodeId) -> bool {
        if self.windows.contains(&window_id) {
            self.active_window = Some(window_id);
            true
        } else {
            false
        }
    }
}

/// レイアウトイベント
#[derive(Debug, Clone)]
pub enum LayoutEvent {
    WindowAdded(NodeId, usize),           // ウィンドウID, ワークスペースID
    WindowRemoved(NodeId, usize),         // ウィンドウID, ワークスペースID
    WindowMoved(NodeId, WorkspaceRect),   // ウィンドウID, 新しい位置
    WindowResized(NodeId, WorkspaceRect), // ウィンドウID, 新しいサイズ
    WindowStateChanged(NodeId),           // ウィンドウID (最大化、最小化、フルスクリーン等)
    WorkspaceAdded(usize),                // ワークスペースID
    WorkspaceRemoved(usize),              // ワークスペースID
    WorkspaceResized(usize, WorkspaceRect), // ワークスペースID, 新しいサイズ
    LayoutTypeChanged(usize, LayoutType), // ワークスペースID, 新しいレイアウトタイプ
    ActiveWindowChanged(usize, Option<NodeId>), // ワークスペースID, アクティブウィンドウID
}

/// レイアウトマネージャー - ウィンドウとワークスペースの配置を管理
pub struct LayoutManager {
    windows: HashMap<NodeId, LayoutWindow>,
    workspaces: HashMap<usize, Workspace>,
    current_workspace: usize,
    layout_engines: HashMap<LayoutType, Box<dyn LayoutEngine>>,
    event_listeners: Vec<Box<dyn Fn(&LayoutEvent) -> bool>>,
    last_update: Instant,
}

/// レイアウトエンジントレイト - 各レイアウトタイプの実装
pub trait LayoutEngine {
    fn name(&self) -> &'static str;
    fn layout_type(&self) -> LayoutType;
    fn arrange_windows(
        &self,
        workspace: &Workspace,
        windows: &[LayoutWindow],
    ) -> Vec<(NodeId, WorkspaceRect)>;
}

impl LayoutManager {
    pub fn new() -> Self {
        let mut manager = Self {
            windows: HashMap::new(),
            workspaces: HashMap::new(),
            current_workspace: 0,
            layout_engines: HashMap::new(),
            event_listeners: Vec::new(),
            last_update: Instant::now(),
        };
        
        // デフォルトのワークスペースを作成
        let default_workspace = Workspace::new(
            0,
            "Default".to_string(),
            WorkspaceRect::new(0, 0, 1920, 1080),
        );
        manager.workspaces.insert(0, default_workspace);
        
        // レイアウトエンジンを登録
        manager.register_layout_engine(Box::new(FloatingLayoutEngine::new()));
        manager.register_layout_engine(Box::new(TilingLayoutEngine::new()));
        manager.register_layout_engine(Box::new(GridLayoutEngine::new()));
        
        manager
    }
    
    /// レイアウトエンジンの登録
    pub fn register_layout_engine(&mut self, engine: Box<dyn LayoutEngine>) {
        let layout_type = engine.layout_type();
        self.layout_engines.insert(layout_type, engine);
    }
    
    /// ウィンドウの追加
    pub fn add_window(&mut self, window: LayoutWindow, workspace_id: usize) -> Result<(), String> {
        let window_id = window.id;
        
        // ワークスペースの存在確認
        if !self.workspaces.contains_key(&workspace_id) {
            return Err(format!("ワークスペースが存在しません: {}", workspace_id));
        }
        
        // ウィンドウの登録
        self.windows.insert(window_id, window);
        
        // ワークスペースにウィンドウを追加
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.add_window(window_id);
            
            // アクティブウィンドウがなければ設定
            if workspace.active_window.is_none() {
                workspace.active_window = Some(window_id);
            }
        }
        
        // イベント発火
        self.emit_event(LayoutEvent::WindowAdded(window_id, workspace_id));
        
        // レイアウトの更新
        self.update_layout(workspace_id);
        
        Ok(())
    }
    
    /// ウィンドウの削除
    pub fn remove_window(&mut self, window_id: NodeId) -> Result<(), String> {
        // ウィンドウの存在確認
        if !self.windows.contains_key(&window_id) {
            return Err(format!("ウィンドウが存在しません: {:?}", window_id));
        }
        
        // すべてのワークスペースから削除
        for (workspace_id, workspace) in &mut self.workspaces {
            if workspace.remove_window(window_id) {
                // イベント発火
                self.emit_event(LayoutEvent::WindowRemoved(window_id, *workspace_id));
                
                // アクティブウィンドウの更新
                if let Some(active_id) = workspace.active_window {
                    if active_id == window_id {
                        workspace.active_window = workspace.windows.last().copied();
                        
                        // イベント発火
                        self.emit_event(LayoutEvent::ActiveWindowChanged(
                            *workspace_id,
                            workspace.active_window,
                        ));
                    }
                }
                
                // レイアウトの更新
                self.update_layout(*workspace_id);
            }
        }
        
        // ウィンドウの削除
        self.windows.remove(&window_id);
        
        Ok(())
    }
    
    /// ワークスペースの追加
    pub fn add_workspace(&mut self, workspace: Workspace) -> Result<(), String> {
        let workspace_id = workspace.id;
        
        // 既に存在するかチェック
        if self.workspaces.contains_key(&workspace_id) {
            return Err(format!("ワークスペースが既に存在します: {}", workspace_id));
        }
        
        // ワークスペースの登録
        self.workspaces.insert(workspace_id, workspace);
        
        // イベント発火
        self.emit_event(LayoutEvent::WorkspaceAdded(workspace_id));
        
        Ok(())
    }
    
    /// ワークスペースの削除
    pub fn remove_workspace(&mut self, workspace_id: usize) -> Result<(), String> {
        // デフォルトワークスペースは削除不可
        if workspace_id == 0 {
            return Err("デフォルトワークスペースは削除できません".to_string());
        }
        
        // ワークスペースの存在確認
        if !self.workspaces.contains_key(&workspace_id) {
            return Err(format!("ワークスペースが存在しません: {}", workspace_id));
        }
        
        // 現在のワークスペースなら別のワークスペースに切り替え
        if self.current_workspace == workspace_id {
            self.switch_workspace(0)?;
        }
        
        // ワークスペース内のウィンドウを処理
        if let Some(workspace) = self.workspaces.get(&workspace_id) {
            let windows: Vec<NodeId> = workspace.windows.clone();
            
            // ウィンドウをデフォルトワークスペースに移動
            for window_id in windows {
                self.move_window_to_workspace(window_id, 0)?;
            }
        }
        
        // ワークスペースの削除
        self.workspaces.remove(&workspace_id);
        
        // イベント発火
        self.emit_event(LayoutEvent::WorkspaceRemoved(workspace_id));
        
        Ok(())
    }
    
    /// ワークスペースの切り替え
    pub fn switch_workspace(&mut self, workspace_id: usize) -> Result<(), String> {
        // ワークスペースの存在確認
        if !self.workspaces.contains_key(&workspace_id) {
            return Err(format!("ワークスペースが存在しません: {}", workspace_id));
        }
        
        self.current_workspace = workspace_id;
        
        // レイアウトの更新
        self.update_layout(workspace_id);
        
        Ok(())
    }
    
    /// 現在のワークスペースのID
    pub fn current_workspace_id(&self) -> usize {
        self.current_workspace
    }
    
    /// 現在のワークスペース
    pub fn current_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(&self.current_workspace)
    }
    
    /// 現在のワークスペース（可変）
    pub fn current_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.workspaces.get_mut(&self.current_workspace)
    }
    
    /// ウィンドウをワークスペース間で移動
    pub fn move_window_to_workspace(
        &mut self,
        window_id: NodeId,
        workspace_id: usize,
    ) -> Result<(), String> {
        // ウィンドウの存在確認
        if !self.windows.contains_key(&window_id) {
            return Err(format!("ウィンドウが存在しません: {:?}", window_id));
        }
        
        // ワークスペースの存在確認
        if !self.workspaces.contains_key(&workspace_id) {
            return Err(format!("ワークスペースが存在しません: {}", workspace_id));
        }
        
        // 現在のワークスペースを探す
        let mut current_workspace_id = None;
        for (id, workspace) in &self.workspaces {
            if workspace.windows.contains(&window_id) {
                current_workspace_id = Some(*id);
                break;
            }
        }
        
        // 同じワークスペースなら何もしない
        if let Some(current_id) = current_workspace_id {
            if current_id == workspace_id {
                return Ok(());
            }
            
            // 現在のワークスペースから削除
            if let Some(workspace) = self.workspaces.get_mut(&current_id) {
                workspace.remove_window(window_id);
                
                // イベント発火
                self.emit_event(LayoutEvent::WindowRemoved(window_id, current_id));
                
                // レイアウトの更新
                self.update_layout(current_id);
            }
        }
        
        // 新しいワークスペースに追加
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.add_window(window_id);
            
            // アクティブウィンドウが設定されていなければ設定
            if workspace.active_window.is_none() {
                workspace.active_window = Some(window_id);
                
                // イベント発火
                self.emit_event(LayoutEvent::ActiveWindowChanged(
                    workspace_id,
                    workspace.active_window,
                ));
            }
            
            // イベント発火
            self.emit_event(LayoutEvent::WindowAdded(window_id, workspace_id));
            
            // レイアウトの更新
            self.update_layout(workspace_id);
        }
        
        Ok(())
    }
    
    /// レイアウトタイプの変更
    pub fn set_layout_type(
        &mut self,
        workspace_id: usize,
        layout_type: LayoutType,
    ) -> Result<(), String> {
        // ワークスペースの存在確認
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            // レイアウトエンジンの存在確認
            if !self.layout_engines.contains_key(&layout_type) {
                return Err(format!("レイアウトエンジンが登録されていません: {:?}", layout_type));
            }
            
            // レイアウトタイプの変更
            workspace.layout_type = layout_type;
            
            // イベント発火
            self.emit_event(LayoutEvent::LayoutTypeChanged(workspace_id, layout_type));
            
            // レイアウトの更新
            self.update_layout(workspace_id);
            
            Ok(())
        } else {
            Err(format!("ワークスペースが存在しません: {}", workspace_id))
        }
    }
    
    /// ワークスペースのサイズ変更
    pub fn resize_workspace(
        &mut self,
        workspace_id: usize,
        rect: WorkspaceRect,
    ) -> Result<(), String> {
        // ワークスペースの存在確認
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            // ワークスペースのサイズ変更
            workspace.rect = rect;
            
            // イベント発火
            self.emit_event(LayoutEvent::WorkspaceResized(workspace_id, rect));
            
            // レイアウトの更新
            self.update_layout(workspace_id);
            
            Ok(())
        } else {
            Err(format!("ワークスペースが存在しません: {}", workspace_id))
        }
    }
    
    /// アクティブウィンドウの設定
    pub fn set_active_window(
        &mut self,
        workspace_id: usize,
        window_id: NodeId,
    ) -> Result<(), String> {
        // ワークスペースの存在確認
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            // ウィンドウの存在確認
            if !workspace.windows.contains(&window_id) {
                return Err(format!(
                    "ウィンドウがワークスペースに存在しません: {:?}",
                    window_id
                ));
            }
            
            // アクティブウィンドウの変更
            workspace.active_window = Some(window_id);
            
            // イベント発火
            self.emit_event(LayoutEvent::ActiveWindowChanged(
                workspace_id,
                workspace.active_window,
            ));
            
            Ok(())
        } else {
            Err(format!("ワークスペースが存在しません: {}", workspace_id))
        }
    }
    
    /// レイアウトの更新
    pub fn update_layout(&mut self, workspace_id: usize) {
        if let Some(workspace) = self.workspaces.get(&workspace_id) {
            let layout_type = workspace.layout_type;
            
            // レイアウトエンジンの取得
            if let Some(engine) = self.layout_engines.get(&layout_type) {
                // ワークスペース内のウィンドウ情報を収集
                let window_ids = workspace.windows.clone();
                let mut windows = Vec::with_capacity(window_ids.len());
                
                for id in window_ids {
                    if let Some(window) = self.windows.get(&id) {
                        windows.push(window.clone());
                    }
                }
                
                // ウィンドウの配置計算
                let arrangements = engine.arrange_windows(workspace, &windows);
                
                // ウィンドウの位置とサイズを更新
                for (window_id, rect) in arrangements {
                    if let Some(window) = self.windows.get_mut(&window_id) {
                        let old_rect = window.rect;
                        window.rect = rect;
                        
                        // 位置またはサイズが変更された場合はイベント発火
                        if old_rect.x != rect.x || old_rect.y != rect.y {
                            self.emit_event(LayoutEvent::WindowMoved(window_id, rect));
                        }
                        
                        if old_rect.width != rect.width || old_rect.height != rect.height {
                            self.emit_event(LayoutEvent::WindowResized(window_id, rect));
                        }
                    }
                }
            }
        }
    }
    
    /// ウィンドウの取得
    pub fn get_window(&self, window_id: NodeId) -> Option<&LayoutWindow> {
        self.windows.get(&window_id)
    }
    
    /// ウィンドウの変更
    pub fn get_window_mut(&mut self, window_id: NodeId) -> Option<&mut LayoutWindow> {
        self.windows.get_mut(&window_id)
    }
    
    /// ワークスペースの取得
    pub fn get_workspace(&self, workspace_id: usize) -> Option<&Workspace> {
        self.workspaces.get(&workspace_id)
    }
    
    /// ワークスペースの変更
    pub fn get_workspace_mut(&mut self, workspace_id: usize) -> Option<&mut Workspace> {
        self.workspaces.get_mut(&workspace_id)
    }
    
    /// イベントリスナーの登録
    pub fn add_event_listener<F>(&mut self, listener: F)
    where
        F: Fn(&LayoutEvent) -> bool + 'static,
    {
        self.event_listeners.push(Box::new(listener));
    }
    
    /// イベントの発火
    fn emit_event(&self, event: LayoutEvent) {
        for listener in &self.event_listeners {
            if !listener(&event) {
                break;
            }
        }
    }
}

//------------------------------------------------------------------------------
// レイアウトエンジンの実装
//------------------------------------------------------------------------------

/// フローティングレイアウト - 自由配置
pub struct FloatingLayoutEngine;

impl FloatingLayoutEngine {
    pub fn new() -> Self {
        Self
    }
}

impl LayoutEngine for FloatingLayoutEngine {
    fn name(&self) -> &'static str {
        "Floating Layout"
    }
    
    fn layout_type(&self) -> LayoutType {
        LayoutType::Floating
    }
    
    fn arrange_windows(
        &self,
        workspace: &Workspace,
        windows: &[LayoutWindow],
    ) -> Vec<(NodeId, WorkspaceRect)> {
        // フローティングレイアウトでは現在の位置を維持
        windows
            .iter()
            .map(|window| (window.id, window.rect))
            .collect()
    }
}

/// タイリングレイアウト - 領域分割配置
pub struct TilingLayoutEngine;

impl TilingLayoutEngine {
    pub fn new() -> Self {
        Self
    }
}

impl LayoutEngine for TilingLayoutEngine {
    fn name(&self) -> &'static str {
        "Tiling Layout"
    }
    
    fn layout_type(&self) -> LayoutType {
        LayoutType::Tiling
    }
    
    fn arrange_windows(
        &self,
        workspace: &Workspace,
        windows: &[LayoutWindow],
    ) -> Vec<(NodeId, WorkspaceRect)> {
        let mut result = Vec::new();
        let workspace_rect = workspace.rect;
        
        // フローティングウィンドウは位置を維持
        let (floating_windows, tiling_windows): (Vec<_>, Vec<_>) = 
            windows.iter().partition(|w| w.floating || w.fullscreen);
        
        // フローティングウィンドウを追加
        for window in &floating_windows {
            result.push((window.id, window.rect));
        }
        
        // タイリングウィンドウがなければ終了
        if tiling_windows.is_empty() {
            return result;
        }
        
        // タイリングウィンドウを配置
        let window_count = tiling_windows.len() as u32;
        
        // 水平分割が効率的な場合（幅が高さより大きい場合）
        if workspace_rect.width >= workspace_rect.height {
            let window_width = workspace_rect.width / window_count;
            let window_height = workspace_rect.height;
            
            for (i, window) in tiling_windows.iter().enumerate() {
                let x = workspace_rect.x + (i as u32 * window_width) as i32;
                let y = workspace_rect.y;
                
                // 最後のウィンドウは残りのスペースをすべて使用
                let width = if i == tiling_windows.len() - 1 {
                    workspace_rect.width - (i as u32 * window_width)
                } else {
                    window_width
                };
                
                result.push((
                    window.id,
                    WorkspaceRect::new(x, y, width, window_height),
                ));
            }
        } else {
            // 垂直分割
            let window_width = workspace_rect.width;
            let window_height = workspace_rect.height / window_count;
            
            for (i, window) in tiling_windows.iter().enumerate() {
                let x = workspace_rect.x;
                let y = workspace_rect.y + (i as u32 * window_height) as i32;
                
                // 最後のウィンドウは残りのスペースをすべて使用
                let height = if i == tiling_windows.len() - 1 {
                    workspace_rect.height - (i as u32 * window_height)
                } else {
                    window_height
                };
                
                result.push((
                    window.id,
                    WorkspaceRect::new(x, y, window_width, height),
                ));
            }
        }
        
        result
    }
}

/// グリッドレイアウト - 格子状配置
pub struct GridLayoutEngine;

impl GridLayoutEngine {
    pub fn new() -> Self {
        Self
    }
}

impl LayoutEngine for GridLayoutEngine {
    fn name(&self) -> &'static str {
        "Grid Layout"
    }
    
    fn layout_type(&self) -> LayoutType {
        LayoutType::Grid
    }
    
    fn arrange_windows(
        &self,
        workspace: &Workspace,
        windows: &[LayoutWindow],
    ) -> Vec<(NodeId, WorkspaceRect)> {
        let mut result = Vec::new();
        let workspace_rect = workspace.rect;
        
        // フローティングウィンドウは位置を維持
        let (floating_windows, tiling_windows): (Vec<_>, Vec<_>) = 
            windows.iter().partition(|w| w.floating || w.fullscreen);
        
        // フローティングウィンドウを追加
        for window in &floating_windows {
            result.push((window.id, window.rect));
        }
        
        // タイリングウィンドウがなければ終了
        if tiling_windows.is_empty() {
            return result;
        }
        
        // グリッドのサイズを計算
        let window_count = tiling_windows.len();
        let cols = (window_count as f64).sqrt().ceil() as u32;
        let rows = ((window_count as f64) / (cols as f64)).ceil() as u32;
        
        let cell_width = workspace_rect.width / cols;
        let cell_height = workspace_rect.height / rows;
        
        for (i, window) in tiling_windows.iter().enumerate() {
            let col = (i as u32) % cols;
            let row = (i as u32) / cols;
            
            let x = workspace_rect.x + (col * cell_width) as i32;
            let y = workspace_rect.y + (row * cell_height) as i32;
            
            // 最後の行と列のセルは残りのスペースをすべて使用
            let width = if col == cols - 1 {
                workspace_rect.width - (col * cell_width)
            } else {
                cell_width
            };
            
            let height = if row == rows - 1 {
                workspace_rect.height - (row * cell_height)
            } else {
                cell_height
            };
            
            result.push((
                window.id,
                WorkspaceRect::new(x, y, width, height),
            ));
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// テスト用のNodeIdを生成
    fn make_node_id(id: u64) -> NodeId {
        NodeId(id)
    }
    
    #[test]
    fn test_layout_manager_creation() {
        let manager = LayoutManager::new();
        
        // デフォルトワークスペースの確認
        assert_eq!(manager.current_workspace_id(), 0);
        assert!(manager.current_workspace().is_some());
    }
    
    #[test]
    fn test_add_window() {
        let mut manager = LayoutManager::new();
        
        // ウィンドウの作成
        let window = LayoutWindow {
            id: make_node_id(1),
            rect: WorkspaceRect::new(0, 0, 100, 100),
            min_size: (50, 50),
            max_size: None,
            floating: false,
            fullscreen: false,
            minimized: false,
            maximized: false,
            z_order: 0,
            snap_edges: HashSet::new(),
            tags: HashSet::new(),
        };
        
        // ウィンドウの追加
        let result = manager.add_window(window, 0);
        assert!(result.is_ok());
        
        // ウィンドウの確認
        assert!(manager.get_window(make_node_id(1)).is_some());
        assert!(manager.current_workspace().unwrap().windows.contains(&make_node_id(1)));
    }
    
    #[test]
    fn test_add_workspace() {
        let mut manager = LayoutManager::new();
        
        // ワークスペースの作成
        let workspace = Workspace::new(
            1,
            "Test".to_string(),
            WorkspaceRect::new(0, 0, 1920, 1080),
        );
        
        // ワークスペースの追加
        let result = manager.add_workspace(workspace);
        assert!(result.is_ok());
        
        // ワークスペースの確認
        assert!(manager.get_workspace(1).is_some());
    }
    
    #[test]
    fn test_layout_types() {
        let mut manager = LayoutManager::new();
        
        // レイアウトタイプの変更
        let result = manager.set_layout_type(0, LayoutType::Tiling);
        assert!(result.is_ok());
        
        // レイアウトタイプの確認
        assert_eq!(manager.current_workspace().unwrap().layout_type, LayoutType::Tiling);
        
        // レイアウトタイプの変更
        let result = manager.set_layout_type(0, LayoutType::Grid);
        assert!(result.is_ok());
        
        // レイアウトタイプの確認
        assert_eq!(manager.current_workspace().unwrap().layout_type, LayoutType::Grid);
    }
    
    #[test]
    fn test_move_window_between_workspaces() {
        let mut manager = LayoutManager::new();
        
        // ワークスペースの追加
        let workspace = Workspace::new(
            1,
            "Test".to_string(),
            WorkspaceRect::new(0, 0, 1920, 1080),
        );
        manager.add_workspace(workspace).unwrap();
        
        // ウィンドウの作成と追加
        let window = LayoutWindow {
            id: make_node_id(1),
            rect: WorkspaceRect::new(0, 0, 100, 100),
            min_size: (50, 50),
            max_size: None,
            floating: false,
            fullscreen: false,
            minimized: false,
            maximized: false,
            z_order: 0,
            snap_edges: HashSet::new(),
            tags: HashSet::new(),
        };
        manager.add_window(window, 0).unwrap();
        
        // ウィンドウの移動
        let result = manager.move_window_to_workspace(make_node_id(1), 1);
        assert!(result.is_ok());
        
        // ウィンドウの確認
        assert!(!manager.get_workspace(0).unwrap().windows.contains(&make_node_id(1)));
        assert!(manager.get_workspace(1).unwrap().windows.contains(&make_node_id(1)));
    }
} 