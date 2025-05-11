// LumosDesktop リソースマネージャー
// グラフィックスリソース（テクスチャ、メッシュ、バッファなど）の管理

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::{GraphicsApi, GraphicsError};
use super::renderer::{Renderer, TextureFormat, BufferTarget, BufferUsage};

/// リソースハンドル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle(pub u64);

impl ResourceHandle {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn invalid() -> Self {
        Self(0)
    }
    
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// リソース読み込みエラー
#[derive(Debug)]
pub enum ResourceLoadError {
    IoError(io::Error),
    DecodingFailed(String),
    UnsupportedFormat(String),
    InvalidResource(String),
    OutOfMemory(String),
    Other(String),
}

impl fmt::Display for ResourceLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceLoadError::IoError(err) => write!(f, "I/Oエラー: {}", err),
            ResourceLoadError::DecodingFailed(msg) => write!(f, "デコード失敗: {}", msg),
            ResourceLoadError::UnsupportedFormat(msg) => write!(f, "サポートされていないフォーマット: {}", msg),
            ResourceLoadError::InvalidResource(msg) => write!(f, "無効なリソース: {}", msg),
            ResourceLoadError::OutOfMemory(msg) => write!(f, "メモリ不足: {}", msg),
            ResourceLoadError::Other(msg) => write!(f, "その他のエラー: {}", msg),
        }
    }
}

impl From<io::Error> for ResourceLoadError {
    fn from(error: io::Error) -> Self {
        ResourceLoadError::IoError(error)
    }
}

impl From<ResourceLoadError> for GraphicsError {
    fn from(error: ResourceLoadError) -> Self {
        GraphicsError::Resource(format!("{}", error))
    }
}

/// リソース状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    Unloaded,
    Loading,
    Loaded,
    Failed,
}

/// リソース基本情報
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub name: String,
    pub path: Option<PathBuf>,
    pub state: ResourceState,
    pub size: usize,
    pub last_modified: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// テクスチャリソース
#[derive(Debug, Clone)]
pub struct TextureResource {
    pub info: ResourceInfo,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub data: Option<Vec<u8>>,
}

impl TextureResource {
    pub fn new(name: &str, width: u32, height: u32, format: TextureFormat) -> Self {
        Self {
            info: ResourceInfo {
                name: name.to_string(),
                path: None,
                state: ResourceState::Unloaded,
                size: 0,
                last_modified: SystemTime::now(),
                metadata: HashMap::new(),
            },
            width,
            height,
            depth: 1,
            format,
            mip_levels: 1,
            array_layers: 1,
            data: None,
        }
    }
    
    pub fn with_data(name: &str, width: u32, height: u32, format: TextureFormat, data: Vec<u8>) -> Self {
        let size = data.len();
        Self {
            info: ResourceInfo {
                name: name.to_string(),
                path: None,
                state: ResourceState::Loaded,
                size,
                last_modified: SystemTime::now(),
                metadata: HashMap::new(),
            },
            width,
            height,
            depth: 1,
            format,
            mip_levels: 1,
            array_layers: 1,
            data: Some(data),
        }
    }
}

/// バッファリソース
#[derive(Debug, Clone)]
pub struct BufferResource {
    pub info: ResourceInfo,
    pub target: BufferTarget,
    pub usage: BufferUsage,
    pub size: usize,
    pub stride: usize,
    pub data: Option<Vec<u8>>,
}

impl BufferResource {
    pub fn new(name: &str, target: BufferTarget, usage: BufferUsage, size: usize, stride: usize) -> Self {
        Self {
            info: ResourceInfo {
                name: name.to_string(),
                path: None,
                state: ResourceState::Unloaded,
                size,
                last_modified: SystemTime::now(),
                metadata: HashMap::new(),
            },
            target,
            usage,
            size,
            stride,
            data: None,
        }
    }
    
    pub fn with_data(name: &str, target: BufferTarget, usage: BufferUsage, data: Vec<u8>, stride: usize) -> Self {
        let size = data.len();
        Self {
            info: ResourceInfo {
                name: name.to_string(),
                path: None,
                state: ResourceState::Loaded,
                size,
                last_modified: SystemTime::now(),
                metadata: HashMap::new(),
            },
            target,
            usage,
            size,
            stride,
            data: Some(data),
        }
    }
}

/// 頂点データ属性
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub format: String,
}

/// メッシュリソース
#[derive(Debug, Clone)]
pub struct MeshResource {
    pub info: ResourceInfo,
    pub vertex_buffer: Option<ResourceHandle>,
    pub index_buffer: Option<ResourceHandle>,
    pub vertex_count: u32,
    pub index_count: u32,
    pub vertex_stride: usize,
    pub index_stride: usize,
    pub attributes: Vec<VertexAttribute>,
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
}

impl MeshResource {
    pub fn new(name: &str) -> Self {
        Self {
            info: ResourceInfo {
                name: name.to_string(),
                path: None,
                state: ResourceState::Unloaded,
                size: 0,
                last_modified: SystemTime::now(),
                metadata: HashMap::new(),
            },
            vertex_buffer: None,
            index_buffer: None,
            vertex_count: 0,
            index_count: 0,
            vertex_stride: 0,
            index_stride: 0,
            attributes: Vec::new(),
            aabb_min: [0.0, 0.0, 0.0],
            aabb_max: [0.0, 0.0, 0.0],
        }
    }
}

// プロテクトメソッド用のプライベート構造体
struct ResourceManagerInner {
    next_handle: u64,
    textures: HashMap<String, TextureResource>,
    buffers: HashMap<String, BufferResource>,
    meshes: HashMap<String, MeshResource>,
    handles: HashMap<ResourceHandle, String>,
}

impl ResourceManagerInner {
    fn new() -> Self {
        Self {
            next_handle: 1, // 0は無効なハンドル
            textures: HashMap::new(),
            buffers: HashMap::new(),
            meshes: HashMap::new(),
            handles: HashMap::new(),
        }
    }
    
    fn generate_handle(&mut self, name: &str) -> ResourceHandle {
        let handle = ResourceHandle(self.next_handle);
        self.next_handle += 1;
        self.handles.insert(handle, name.to_string());
        handle
    }
}

/// リソースマネージャー
pub struct ResourceManager {
    inner: ResourceManagerInner,
    renderer: Arc<Mutex<dyn Renderer>>,
    current_api: GraphicsApi,
    search_paths: Vec<PathBuf>,
    max_texture_size: u32,
    texture_compression: bool,
}

impl fmt::Debug for ResourceManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceManager")
            .field("textures", &self.inner.textures.len())
            .field("buffers", &self.inner.buffers.len())
            .field("meshes", &self.inner.meshes.len())
            .field("current_api", &self.current_api)
            .field("max_texture_size", &self.max_texture_size)
            .field("texture_compression", &self.texture_compression)
            .finish()
    }
}

impl ResourceManager {
    /// 新しいリソースマネージャーを作成
    pub fn new(
        api: GraphicsApi,
        renderer: Arc<Mutex<dyn Renderer>>,
        max_texture_size: u32,
        texture_compression: bool,
    ) -> Result<Self, GraphicsError> {
        Ok(Self {
            inner: ResourceManagerInner::new(),
            renderer,
            current_api: api,
            search_paths: vec![PathBuf::from("assets")],
            max_texture_size,
            texture_compression,
        })
    }
    
    /// 検索パスを追加
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(PathBuf::from(path.as_ref()));
    }
    
    /// テクスチャをファイルから読み込む
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P, name: Option<&str>) -> Result<ResourceHandle, GraphicsError> {
        let path = path.as_ref();
        let texture_name = name.map(|s| s.to_string()).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unnamed".to_string())
        });
        
        // ファイルの拡張子を取得
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| GraphicsError::Resource("テクスチャファイルに拡張子がありません".to_string()))?
            .to_lowercase();
        
        // ファイルを読み込む
        let file_data = fs::read(path)
            .map_err(|e| GraphicsError::Resource(format!("テクスチャファイルの読み込みに失敗: {}", e)))?;
        
        // 画像をデコードし、メモリ内にロード
        let (format, width, height, data) = match extension.as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "tga" => {
                // 実際のアプリケーションでは画像デコードライブラリを使用
                // ここではダミーデータを返す
                (TextureFormat::R8G8B8A8, 256, 256, file_data)
            }
            "ktx" | "dds" => {
                // 圧縮テクスチャフォーマット
                // 実際にはKTXやDDSパーサーを使用
                (TextureFormat::BC3, 256, 256, file_data)
            }
            _ => {
                return Err(GraphicsError::Resource(format!(
                    "サポートされていないテクスチャフォーマット: {}", extension
                )));
            }
        };
        
        // サイズが最大テクスチャサイズを超えていないか確認
        if width > self.max_texture_size || height > self.max_texture_size {
            return Err(GraphicsError::Resource(format!(
                "テクスチャサイズが最大許容サイズを超えています: {}x{} (最大: {}x{})",
                width, height, self.max_texture_size, self.max_texture_size
            )));
        }
        
        // テクスチャリソースを作成
        let texture = TextureResource {
            info: ResourceInfo {
                name: texture_name.clone(),
                path: Some(path.to_path_buf()),
                state: ResourceState::Loaded,
                size: data.len(),
                last_modified: SystemTime::now(),
                metadata: HashMap::new(),
            },
            width,
            height,
            depth: 1,
            format,
            mip_levels: 1,
            array_layers: 1,
            data: Some(data),
        };
        
        // レンダラーにテクスチャを登録
        {
            let mut renderer = self.renderer.lock()
                .map_err(|_| GraphicsError::Resource("レンダラーのロックに失敗".to_string()))?;
            
            renderer.create_texture(
                &texture_name,
                width,
                height,
                format,
                texture.data.as_deref(),
            )?;
        }
        
        // リソースマネージャーに登録
        let handle = self.inner.generate_handle(&texture_name);
        self.inner.textures.insert(texture_name, texture);
        
        Ok(handle)
    }
    
    /// メモリからテクスチャを作成
    pub fn create_texture(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        format: TextureFormat,
        data: Option<&[u8]>,
    ) -> Result<ResourceHandle, GraphicsError> {
        // すでに同名のテクスチャがある場合は上書き
        let texture = TextureResource {
            info: ResourceInfo {
                name: name.to_string(),
                path: None,
                state: ResourceState::Loaded,
                size: data.map_or(0, |d| d.len()),
                last_modified: SystemTime::now(),
                metadata: HashMap::new(),
            },
            width,
            height,
            depth: 1,
            format,
            mip_levels: 1,
            array_layers: 1,
            data: data.map(|d| d.to_vec()),
        };
        
        // レンダラーにテクスチャを登録
        {
            let mut renderer = self.renderer.lock()
                .map_err(|_| GraphicsError::Resource("レンダラーのロックに失敗".to_string()))?;
            
            renderer.create_texture(name, width, height, format, data)?;
        }
        
        // リソースマネージャーに登録
        let handle = self.inner.generate_handle(name);
        self.inner.textures.insert(name.to_string(), texture);
        
        Ok(handle)
    }
    
    /// バッファを作成
    pub fn create_buffer(
        &mut self,
        name: &str,
        target: BufferTarget,
        usage: BufferUsage,
        data: Option<&[u8]>,
        stride: usize,
    ) -> Result<ResourceHandle, GraphicsError> {
        let size = data.map_or(0, |d| d.len());
        
        // バッファリソースを作成
        let buffer = BufferResource {
            info: ResourceInfo {
                name: name.to_string(),
                path: None,
                state: ResourceState::Loaded,
                size,
                last_modified: SystemTime::now(),
                metadata: HashMap::new(),
            },
            target,
            usage,
            size,
            stride,
            data: data.map(|d| d.to_vec()),
        };
        
        // レンダラーにバッファを登録
        {
            let mut renderer = self.renderer.lock()
                .map_err(|_| GraphicsError::Resource("レンダラーのロックに失敗".to_string()))?;
            
            renderer.create_buffer(name, target, usage, data, size)?;
        }
        
        // リソースマネージャーに登録
        let handle = self.inner.generate_handle(name);
        self.inner.buffers.insert(name.to_string(), buffer);
        
        Ok(handle)
    }
    
    /// メッシュを作成
    pub fn create_mesh(
        &mut self,
        name: &str,
        vertices: &[u8],
        indices: Option<&[u8]>,
        vertex_stride: usize,
        index_stride: usize,
        attributes: Vec<VertexAttribute>,
    ) -> Result<ResourceHandle, GraphicsError> {
        let vertex_count = vertices.len() / vertex_stride;
        let index_count = indices.map_or(0, |i| i.len() / index_stride);
        
        // 頂点バッファを作成
        let vertex_buffer_name = format!("{}_vb", name);
        let vertex_buffer = self.create_buffer(
            &vertex_buffer_name,
            BufferTarget::Vertex,
            BufferUsage::Static,
            Some(vertices),
            vertex_stride,
        )?;
        
        // インデックスバッファを作成（存在する場合）
        let index_buffer = if let Some(indices) = indices {
            let index_buffer_name = format!("{}_ib", name);
            Some(self.create_buffer(
                &index_buffer_name,
                BufferTarget::Index,
                BufferUsage::Static,
                Some(indices),
                index_stride,
            )?)
        } else {
            None
        };
        
        // メッシュリソースを作成
        let mesh = MeshResource {
            info: ResourceInfo {
                name: name.to_string(),
                path: None,
                state: ResourceState::Loaded,
                size: vertices.len() + indices.map_or(0, |i| i.len()),
                last_modified: SystemTime::now(),
                metadata: HashMap::new(),
            },
            vertex_buffer: Some(vertex_buffer),
            index_buffer,
            vertex_count: vertex_count as u32,
            index_count: index_count as u32,
            vertex_stride,
            index_stride,
            attributes,
            aabb_min: [0.0, 0.0, 0.0], // ダミー値（実際には計算する）
            aabb_max: [0.0, 0.0, 0.0], // ダミー値（実際には計算する）
        };
        
        // リソースマネージャーに登録
        let handle = self.inner.generate_handle(name);
        self.inner.meshes.insert(name.to_string(), mesh);
        
        Ok(handle)
    }
    
    /// テクスチャを取得
    pub fn get_texture(&self, name: &str) -> Option<&TextureResource> {
        self.inner.textures.get(name)
    }
    
    /// バッファを取得
    pub fn get_buffer(&self, name: &str) -> Option<&BufferResource> {
        self.inner.buffers.get(name)
    }
    
    /// メッシュを取得
    pub fn get_mesh(&self, name: &str) -> Option<&MeshResource> {
        self.inner.meshes.get(name)
    }
    
    /// ハンドルからリソース名を取得
    pub fn get_resource_name(&self, handle: ResourceHandle) -> Option<&str> {
        self.inner.handles.get(&handle).map(|s| s.as_str())
    }
    
    /// ハンドルからテクスチャを取得
    pub fn get_texture_by_handle(&self, handle: ResourceHandle) -> Option<&TextureResource> {
        self.get_resource_name(handle).and_then(|name| self.get_texture(name))
    }
    
    /// ハンドルからバッファを取得
    pub fn get_buffer_by_handle(&self, handle: ResourceHandle) -> Option<&BufferResource> {
        self.get_resource_name(handle).and_then(|name| self.get_buffer(name))
    }
    
    /// ハンドルからメッシュを取得
    pub fn get_mesh_by_handle(&self, handle: ResourceHandle) -> Option<&MeshResource> {
        self.get_resource_name(handle).and_then(|name| self.get_mesh(name))
    }
    
    /// テクスチャを削除
    pub fn remove_texture(&mut self, name: &str) -> Result<(), GraphicsError> {
        if self.inner.textures.remove(name).is_some() {
            // ハンドルも削除
            let handles_to_remove: Vec<_> = self.inner.handles.iter()
                .filter(|(_, n)| n == &name)
                .map(|(h, _)| *h)
                .collect();
            
            for handle in handles_to_remove {
                self.inner.handles.remove(&handle);
            }
            
            Ok(())
        } else {
            Err(GraphicsError::Resource(format!("テクスチャが見つかりません: {}", name)))
        }
    }
    
    /// バッファを削除
    pub fn remove_buffer(&mut self, name: &str) -> Result<(), GraphicsError> {
        if self.inner.buffers.remove(name).is_some() {
            // ハンドルも削除
            let handles_to_remove: Vec<_> = self.inner.handles.iter()
                .filter(|(_, n)| n == &name)
                .map(|(h, _)| *h)
                .collect();
            
            for handle in handles_to_remove {
                self.inner.handles.remove(&handle);
            }
            
            Ok(())
        } else {
            Err(GraphicsError::Resource(format!("バッファが見つかりません: {}", name)))
        }
    }
    
    /// メッシュを削除
    pub fn remove_mesh(&mut self, name: &str) -> Result<(), GraphicsError> {
        if let Some(mesh) = self.inner.meshes.remove(name) {
            // ハンドルも削除
            let handles_to_remove: Vec<_> = self.inner.handles.iter()
                .filter(|(_, n)| n == &name)
                .map(|(h, _)| *h)
                .collect();
            
            for handle in handles_to_remove {
                self.inner.handles.remove(&handle);
            }
            
            // 関連するバッファも削除（オプション）
            if let Some(vb) = mesh.vertex_buffer {
                if let Some(vb_name) = self.get_resource_name(vb) {
                    let _ = self.remove_buffer(vb_name);
                }
            }
            
            if let Some(ib) = mesh.index_buffer {
                if let Some(ib_name) = self.get_resource_name(ib) {
                    let _ = self.remove_buffer(ib_name);
                }
            }
            
            Ok(())
        } else {
            Err(GraphicsError::Resource(format!("メッシュが見つかりません: {}", name)))
        }
    }
    
    /// すべてのリソースをクリア
    pub fn clear(&mut self) {
        self.inner.textures.clear();
        self.inner.buffers.clear();
        self.inner.meshes.clear();
        self.inner.handles.clear();
    }
    
    /// クリーンアップ処理
    pub fn cleanup(&mut self) -> Result<(), GraphicsError> {
        // すべてのリソースをクリア
        self.clear();
        Ok(())
    }
    
    /// テクスチャのリストを取得
    pub fn get_texture_names(&self) -> Vec<String> {
        self.inner.textures.keys().cloned().collect()
    }
    
    /// バッファのリストを取得
    pub fn get_buffer_names(&self) -> Vec<String> {
        self.inner.buffers.keys().cloned().collect()
    }
    
    /// メッシュのリストを取得
    pub fn get_mesh_names(&self) -> Vec<String> {
        self.inner.meshes.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // モックレンダラーを作成
    struct MockRenderer;
    
    impl fmt::Debug for MockRenderer {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "MockRenderer")
        }
    }
    
    impl Renderer for MockRenderer {
        fn name(&self) -> &str {
            "MockRenderer"
        }
        
        fn initialize(&mut self, _config: &super::super::GraphicsConfig) -> Result<(), GraphicsError> {
            Ok(())
        }
        
        fn shutdown(&mut self) -> Result<(), GraphicsError> {
            Ok(())
        }
        
        fn update_config(&mut self, _config: &super::super::GraphicsConfig) -> Result<(), GraphicsError> {
            Ok(())
        }
        
        fn create_context(&mut self) -> Result<Box<dyn super::super::renderer::RenderContext>, GraphicsError> {
            unimplemented!()
        }
        
        fn create_texture(&mut self, _name: &str, _width: u32, _height: u32, _format: TextureFormat, _data: Option<&[u8]>) -> Result<(), GraphicsError> {
            Ok(())
        }
        
        fn create_buffer(&mut self, _name: &str, _target: BufferTarget, _usage: BufferUsage, _data: Option<&[u8]>, _size: usize) -> Result<(), GraphicsError> {
            Ok(())
        }
        
        fn get_device_info(&self) -> HashMap<String, String> {
            HashMap::new()
        }
        
        fn get_capabilities(&self) -> HashMap<String, String> {
            HashMap::new()
        }
    }
    
    #[test]
    fn test_resource_handle() {
        let handle = ResourceHandle::new(42);
        assert_eq!(handle.0, 42);
        assert!(handle.is_valid());
        
        let invalid = ResourceHandle::invalid();
        assert_eq!(invalid.0, 0);
        assert!(!invalid.is_valid());
    }
    
    #[test]
    fn test_resource_manager_creation() {
        let renderer = Arc::new(Mutex::new(MockRenderer));
        let manager = ResourceManager::new(GraphicsApi::Vulkan, renderer, 4096, true).unwrap();
        
        assert_eq!(manager.inner.textures.len(), 0);
        assert_eq!(manager.inner.buffers.len(), 0);
        assert_eq!(manager.inner.meshes.len(), 0);
        assert_eq!(manager.max_texture_size, 4096);
        assert_eq!(manager.texture_compression, true);
    }
} 