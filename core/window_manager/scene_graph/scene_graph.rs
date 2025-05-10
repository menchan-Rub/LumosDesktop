// LumosDesktop シーングラフ
// ウィンドウやUIコンポーネントの階層構造を管理する高速シーングラフ

use std::collections::{HashMap, VecDeque};
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// シーンノードの識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// シーンノードの種類
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Root,
    Window,
    Panel,
    Overlay,
    Widget,
    Container,
    Decoration,
    Background,
    Custom(String),
}

/// 変換情報
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: (f32, f32, f32),
    pub scale: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32), // クォータニオン (x, y, z, w)
    pub origin: (f32, f32, f32),        // 変換の原点
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            rotation: (0.0, 0.0, 0.0, 1.0), // 単位クォータニオン
            origin: (0.0, 0.0, 0.0),
        }
    }
    
    /// 2つの変換を結合
    pub fn combine(&self, other: &Transform) -> Transform {
        // 実際の実装ではより複雑なクォータニオン計算を行う
        // 簡略化のために単純な結合を行う
        Transform {
            position: (
                self.position.0 + other.position.0 * self.scale.0,
                self.position.1 + other.position.1 * self.scale.1,
                self.position.2 + other.position.2 * self.scale.2,
            ),
            scale: (
                self.scale.0 * other.scale.0,
                self.scale.1 * other.scale.1,
                self.scale.2 * other.scale.2,
            ),
            rotation: combine_quaternions(self.rotation, other.rotation),
            origin: (
                self.origin.0 + other.origin.0,
                self.origin.1 + other.origin.1,
                self.origin.2 + other.origin.2,
            ),
        }
    }
    
    /// 行列に変換
    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        // 実際の実装では完全な4x4行列変換を行う
        let mut matrix = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        
        // スケーリング
        matrix[0][0] = self.scale.0;
        matrix[1][1] = self.scale.1;
        matrix[2][2] = self.scale.2;
        
        // TODO: クォータニオンからの回転行列
        
        // 移動
        matrix[3][0] = self.position.0;
        matrix[3][1] = self.position.1;
        matrix[3][2] = self.position.2;
        
        matrix
    }
}

/// クォータニオンの結合
fn combine_quaternions(q1: (f32, f32, f32, f32), q2: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    // q1 * q2の計算
    let x = q1.3 * q2.0 + q1.0 * q2.3 + q1.1 * q2.2 - q1.2 * q2.1;
    let y = q1.3 * q2.1 - q1.0 * q2.2 + q1.1 * q2.3 + q1.2 * q2.0;
    let z = q1.3 * q2.2 + q1.0 * q2.1 - q1.1 * q2.0 + q1.2 * q2.3;
    let w = q1.3 * q2.3 - q1.0 * q2.0 - q1.1 * q2.1 - q1.2 * q2.2;
    
    (x, y, z, w)
}

/// 境界ボックス
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: (f32, f32, f32),
    pub max: (f32, f32, f32),
}

impl BoundingBox {
    pub fn new(min: (f32, f32, f32), max: (f32, f32, f32)) -> Self {
        Self { min, max }
    }
    
    pub fn from_size(width: f32, height: f32, depth: f32) -> Self {
        Self {
            min: (0.0, 0.0, 0.0),
            max: (width, height, depth),
        }
    }
    
    /// 境界ボックスの変換
    pub fn transform(&self, transform: &Transform) -> BoundingBox {
        // 実際の実装ではより正確な変換を行う
        // 簡略化のため、スケールと移動のみ考慮
        let min = (
            self.min.0 * transform.scale.0 + transform.position.0,
            self.min.1 * transform.scale.1 + transform.position.1,
            self.min.2 * transform.scale.2 + transform.position.2,
        );
        
        let max = (
            self.max.0 * transform.scale.0 + transform.position.0,
            self.max.1 * transform.scale.1 + transform.position.1,
            self.max.2 * transform.scale.2 + transform.position.2,
        );
        
        BoundingBox::new(min, max)
    }
    
    /// 境界ボックスの交差判定
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.0 <= other.max.0 && self.max.0 >= other.min.0 &&
        self.min.1 <= other.max.1 && self.max.1 >= other.min.1 &&
        self.min.2 <= other.max.2 && self.max.2 >= other.min.2
    }
    
    /// 境界ボックス内にポイントが含まれるかの判定
    pub fn contains_point(&self, point: (f32, f32, f32)) -> bool {
        point.0 >= self.min.0 && point.0 <= self.max.0 &&
        point.1 >= self.min.1 && point.1 <= self.max.1 &&
        point.2 >= self.min.2 && point.2 <= self.max.2
    }
    
    /// 二つの境界ボックスの和集合
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: (
                self.min.0.min(other.min.0),
                self.min.1.min(other.min.1),
                self.min.2.min(other.min.2),
            ),
            max: (
                self.max.0.max(other.max.0),
                self.max.1.max(other.max.1),
                self.max.2.max(other.max.2),
            ),
        }
    }
}

/// シーンノードのプロパティ
#[derive(Debug, Clone)]
pub struct NodeProperties {
    pub visible: bool,
    pub opacity: f32,
    pub clip_to_bounds: bool,
    pub interactive: bool,
    pub layer: u32,         // レンダリング層
    pub tag: Option<String>, // 任意のタグ
    pub data: Option<Arc<dyn std::any::Any + Send + Sync>>, // カスタムデータ
}

impl Default for NodeProperties {
    fn default() -> Self {
        Self {
            visible: true,
            opacity: 1.0,
            clip_to_bounds: true,
            interactive: true,
            layer: 0,
            tag: None,
            data: None,
        }
    }
}

/// シーンノード
pub struct SceneNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub name: String,
    pub transform: Transform,
    pub bounds: BoundingBox,
    pub properties: NodeProperties,
    pub parent: Option<Weak<RefCell<SceneNode>>>,
    pub children: Vec<Rc<RefCell<SceneNode>>>,
    pub render_data: Option<Arc<dyn std::any::Any + Send + Sync>>,
    pub last_update: Instant,
}

impl SceneNode {
    pub fn new(id: NodeId, node_type: NodeType, name: String) -> Self {
        Self {
            id,
            node_type,
            name,
            transform: Transform::identity(),
            bounds: BoundingBox::from_size(0.0, 0.0, 0.0),
            properties: NodeProperties::default(),
            parent: None,
            children: Vec::new(),
            render_data: None,
            last_update: Instant::now(),
        }
    }
    
    /// 子ノードの追加
    pub fn add_child(&mut self, child: Rc<RefCell<SceneNode>>) {
        // 親の設定
        child.borrow_mut().parent = Some(Rc::downgrade(&Rc::new(RefCell::new(self.clone()))));
        self.children.push(child);
    }
    
    /// 子ノードの削除
    pub fn remove_child(&mut self, id: NodeId) -> Option<Rc<RefCell<SceneNode>>> {
        let index = self.children.iter().position(|child| child.borrow().id == id)?;
        let child = self.children.remove(index);
        child.borrow_mut().parent = None;
        Some(child)
    }
    
    /// 指定された型の最も近い親を探す
    pub fn find_parent_of_type(&self, node_type: &NodeType) -> Option<Rc<RefCell<SceneNode>>> {
        let parent_weak = self.parent.as_ref()?;
        let parent = parent_weak.upgrade()?;
        
        if parent.borrow().node_type == *node_type {
            Some(parent)
        } else {
            parent.borrow().find_parent_of_type(node_type)
        }
    }
    
    /// グローバル変換の計算
    pub fn global_transform(&self) -> Transform {
        match &self.parent {
            Some(parent_weak) => {
                if let Some(parent) = parent_weak.upgrade() {
                    parent.borrow().global_transform().combine(&self.transform)
                } else {
                    self.transform.clone()
                }
            },
            None => self.transform.clone(),
        }
    }
    
    /// グローバル境界ボックスの計算
    pub fn global_bounds(&self) -> BoundingBox {
        self.bounds.transform(&self.global_transform())
    }
    
    /// ノードが表示されているかどうか
    pub fn is_effectively_visible(&self) -> bool {
        if !self.properties.visible || self.properties.opacity <= 0.0 {
            return false;
        }
        
        // 親が非表示なら子も非表示
        match &self.parent {
            Some(parent_weak) => {
                if let Some(parent) = parent_weak.upgrade() {
                    parent.borrow().is_effectively_visible()
                } else {
                    true
                }
            },
            None => true,
        }
    }
    
    /// 実効的な不透明度の計算
    pub fn effective_opacity(&self) -> f32 {
        let self_opacity = self.properties.opacity;
        
        match &self.parent {
            Some(parent_weak) => {
                if let Some(parent) = parent_weak.upgrade() {
                    self_opacity * parent.borrow().effective_opacity()
                } else {
                    self_opacity
                }
            },
            None => self_opacity,
        }
    }
}

impl Clone for SceneNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            node_type: self.node_type.clone(),
            name: self.name.clone(),
            transform: self.transform.clone(),
            bounds: self.bounds,
            properties: self.properties.clone(),
            parent: self.parent.clone(),
            children: Vec::new(), // 子はクローンしない
            render_data: self.render_data.clone(),
            last_update: self.last_update,
        }
    }
}

/// シーングラフ
pub struct SceneGraph {
    root: Rc<RefCell<SceneNode>>,
    nodes: HashMap<NodeId, Weak<RefCell<SceneNode>>>,
    next_id: u64,
    dirty_nodes: Vec<NodeId>,
    cached_transforms: HashMap<NodeId, Transform>,
    cached_bounds: HashMap<NodeId, BoundingBox>,
    spatial_index: SpatialIndex,
    update_time: Instant,
}

/// 空間インデックス - 効率的な空間検索のために
struct SpatialIndex {
    // 簡易的な実装。実際の実装ではQuadtreeやR-treeなどを使用する
    nodes: HashMap<NodeId, BoundingBox>,
}

impl SpatialIndex {
    fn new() -> Self {
        Self { nodes: HashMap::new() }
    }
    
    fn update(&mut self, id: NodeId, bounds: BoundingBox) {
        self.nodes.insert(id, bounds);
    }
    
    fn remove(&mut self, id: &NodeId) {
        self.nodes.remove(id);
    }
    
    fn query_region(&self, region: &BoundingBox) -> Vec<NodeId> {
        self.nodes.iter()
            .filter(|(_, bounds)| bounds.intersects(region))
            .map(|(&id, _)| id)
            .collect()
    }
    
    fn query_point(&self, point: (f32, f32, f32)) -> Vec<NodeId> {
        self.nodes.iter()
            .filter(|(_, bounds)| bounds.contains_point(point))
            .map(|(&id, _)| id)
            .collect()
    }
}

impl SceneGraph {
    pub fn new() -> Self {
        let root_id = NodeId(0);
        let root = Rc::new(RefCell::new(SceneNode::new(
            root_id,
            NodeType::Root,
            "root".to_string(),
        )));
        
        let mut nodes = HashMap::new();
        nodes.insert(root_id, Rc::downgrade(&root));
        
        Self {
            root,
            nodes,
            next_id: 1,
            dirty_nodes: Vec::new(),
            cached_transforms: HashMap::new(),
            cached_bounds: HashMap::new(),
            spatial_index: SpatialIndex::new(),
            update_time: Instant::now(),
        }
    }
    
    /// 新しいノードIDの生成
    pub fn generate_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }
    
    /// ノードの作成と追加
    pub fn create_node(&mut self, parent_id: NodeId, node_type: NodeType, name: String) -> Result<NodeId, String> {
        let id = self.generate_id();
        let node = Rc::new(RefCell::new(SceneNode::new(id, node_type, name)));
        
        // ノードをノードマップに追加
        self.nodes.insert(id, Rc::downgrade(&node));
        
        // 親ノードに子を追加
        if let Some(parent_weak) = self.nodes.get(&parent_id) {
            if let Some(parent) = parent_weak.upgrade() {
                // 親に子を追加
                parent.borrow_mut().add_child(node);
                // ノードが変更されたことをマーク
                self.mark_dirty(id);
                
                Ok(id)
            } else {
                Err("親ノードが既に削除されています".to_string())
            }
        } else {
            Err("指定された親ノードが存在しません".to_string())
        }
    }
    
    /// ルートノードの取得
    pub fn root(&self) -> Rc<RefCell<SceneNode>> {
        self.root.clone()
    }
    
    /// ノードの取得
    pub fn get_node(&self, id: NodeId) -> Option<Rc<RefCell<SceneNode>>> {
        self.nodes.get(&id).and_then(|weak| weak.upgrade())
    }
    
    /// ノードの削除
    pub fn remove_node(&mut self, id: NodeId) -> Result<(), String> {
        if id == self.root.borrow().id {
            return Err("ルートノードは削除できません".to_string());
        }
        
        if let Some(node_weak) = self.nodes.get(&id) {
            if let Some(node) = node_weak.upgrade() {
                // 親から自分を削除
                if let Some(parent_weak) = &node.borrow().parent {
                    if let Some(parent) = parent_weak.upgrade() {
                        parent.borrow_mut().remove_child(id);
                    }
                }
                
                // 子ノードも再帰的に削除
                let children_ids: Vec<NodeId> = node.borrow()
                    .children
                    .iter()
                    .map(|child| child.borrow().id)
                    .collect();
                
                for child_id in children_ids {
                    self.remove_node(child_id)?;
                }
                
                // ノードを削除
                self.nodes.remove(&id);
                self.cached_transforms.remove(&id);
                self.cached_bounds.remove(&id);
                self.spatial_index.remove(&id);
                
                Ok(())
            } else {
                Err("ノードは既に削除されています".to_string())
            }
        } else {
            Err("指定されたノードが存在しません".to_string())
        }
    }
    
    /// ノードを変更済みとしてマーク
    pub fn mark_dirty(&mut self, id: NodeId) {
        if !self.dirty_nodes.contains(&id) {
            self.dirty_nodes.push(id);
            
            // 子ノードも再帰的にマーク
            if let Some(node) = self.get_node(id) {
                for child in &node.borrow().children {
                    self.mark_dirty(child.borrow().id);
                }
            }
        }
    }
    
    /// シーングラフの更新
    pub fn update(&mut self) {
        let now = Instant::now();
        
        // 変更されたノードの変換と境界を再計算
        for &id in &self.dirty_nodes {
            if let Some(node) = self.get_node(id) {
                let global_transform = node.borrow().global_transform();
                let global_bounds = node.borrow().global_bounds();
                
                self.cached_transforms.insert(id, global_transform);
                self.cached_bounds.insert(id, global_bounds);
                self.spatial_index.update(id, global_bounds);
            }
        }
        
        self.dirty_nodes.clear();
        self.update_time = now;
    }
    
    /// 描画順にノードを取得
    pub fn get_nodes_in_draw_order(&self) -> Vec<Rc<RefCell<SceneNode>>> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(self.root.clone());
        
        while let Some(node) = queue.pop_front() {
            let node_ref = node.borrow();
            
            if node_ref.is_effectively_visible() {
                result.push(node.clone());
                
                // 子ノードをレイヤー順にソートして追加
                let mut children: Vec<_> = node_ref.children.clone();
                children.sort_by_key(|child| child.borrow().properties.layer);
                
                for child in children {
                    queue.push_back(child);
                }
            }
        }
        
        result
    }
    
    /// 指定した点を含むノードを検索
    pub fn hit_test(&self, point: (f32, f32, f32)) -> Vec<NodeId> {
        // 空間インデックスを使用して候補を絞り込む
        let candidates = self.spatial_index.query_point(point);
        
        // インタラクティブなノードだけをフィルタリング
        candidates.into_iter()
            .filter(|&id| {
                if let Some(node) = self.get_node(id) {
                    let node_ref = node.borrow();
                    node_ref.properties.interactive && node_ref.is_effectively_visible()
                } else {
                    false
                }
            })
            .collect()
    }
    
    /// 指定した領域に含まれるノードを検索
    pub fn query_region(&self, region: BoundingBox) -> Vec<NodeId> {
        self.spatial_index.query_region(&region)
    }
    
    /// ツリー構造を文字列として取得（デバッグ用）
    pub fn print_tree(&self) -> String {
        let mut result = String::new();
        self.print_node(&self.root, 0, &mut result);
        result
    }
    
    fn print_node(&self, node: &Rc<RefCell<SceneNode>>, depth: usize, result: &mut String) {
        let node_ref = node.borrow();
        let indent = "  ".repeat(depth);
        
        result.push_str(&format!("{}{}: {} ({:?})\n",
            indent,
            node_ref.id.0,
            node_ref.name,
            node_ref.node_type
        ));
        
        for child in &node_ref.children {
            self.print_node(child, depth + 1, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scene_graph_creation() {
        let mut graph = SceneGraph::new();
        let root_id = graph.root().borrow().id;
        
        // パネルの追加
        let panel_id = graph.create_node(
            root_id,
            NodeType::Panel,
            "main_panel".to_string()
        ).unwrap();
        
        // ウィンドウの追加
        let window_id = graph.create_node(
            root_id,
            NodeType::Window,
            "test_window".to_string()
        ).unwrap();
        
        // ウィジェットの追加
        let widget_id = graph.create_node(
            panel_id,
            NodeType::Widget,
            "test_widget".to_string()
        ).unwrap();
        
        // ノードの検証
        assert!(graph.get_node(root_id).is_some());
        assert!(graph.get_node(panel_id).is_some());
        assert!(graph.get_node(window_id).is_some());
        assert!(graph.get_node(widget_id).is_some());
        
        // 親子関係の検証
        let panel = graph.get_node(panel_id).unwrap();
        assert_eq!(panel.borrow().children.len(), 1);
        
        let root = graph.get_node(root_id).unwrap();
        assert_eq!(root.borrow().children.len(), 2);
        
        // ノードの削除
        graph.remove_node(panel_id).unwrap();
        assert!(graph.get_node(panel_id).is_none());
        assert!(graph.get_node(widget_id).is_none()); // 子も削除される
        
        let root_after = graph.get_node(root_id).unwrap();
        assert_eq!(root_after.borrow().children.len(), 1); // パネルが削除されたので1つのみ
    }
    
    #[test]
    fn test_bounding_box() {
        let box1 = BoundingBox::from_size(10.0, 10.0, 0.0);
        let box2 = BoundingBox::new((5.0, 5.0, 0.0), (15.0, 15.0, 0.0));
        
        assert!(box1.intersects(&box2));
        assert!(box1.contains_point((5.0, 5.0, 0.0)));
        assert!(!box1.contains_point((15.0, 15.0, 0.0)));
        
        let transform = Transform {
            position: (10.0, 10.0, 0.0),
            scale: (2.0, 2.0, 1.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            origin: (0.0, 0.0, 0.0),
        };
        
        let transformed = box1.transform(&transform);
        assert_eq!(transformed.min, (10.0, 10.0, 0.0));
        assert_eq!(transformed.max, (30.0, 30.0, 0.0));
    }
} 