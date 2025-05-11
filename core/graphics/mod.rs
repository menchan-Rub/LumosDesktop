// LumosDesktop グラフィックスモジュール
// グラフィックス処理の抽象化レイヤーを提供します

//! # グラフィックスモジュール
//!
//! このモジュールはLumosDesktopのグラフィックス処理機能を提供します。
//! レンダラー、シェーダー管理、リソース管理などのコンポーネントが含まれています。
//!
//! グラフィックスモジュールは、様々なバックエンド（Vulkan、Metal、DirectX）への
//! 抽象化レイヤーとして機能し、プラットフォーム間での一貫した描画APIを提供します。

pub mod renderer;
pub mod vulkan_backend;
pub mod metal_backend;
pub mod dx_backend;
pub mod shader_manager;
pub mod resource_manager;

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

// 主要なモジュールの公開型をre-export
pub use renderer::{
    Renderer,
    RenderContext,
    RenderPass,
    RenderTarget,
    RenderCommandBuffer,
    PipelineState
};

pub use shader_manager::{
    ShaderManager,
    Shader,
    ShaderStage,
    ShaderCompilationError
};

pub use resource_manager::{
    ResourceManager,
    TextureResource,
    BufferResource,
    MeshResource,
    ResourceHandle,
    ResourceLoadError
};

/// グラフィックスAPIタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsApi {
    /// Vulkan
    Vulkan,
    /// Metal (macOS/iOS向け)
    Metal,
    /// DirectX (Windows向け)
    DirectX,
    /// OpenGL
    OpenGL,
    /// ソフトウェアレンダリング (フォールバック)
    Software,
}

impl fmt::Display for GraphicsApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphicsApi::Vulkan => write!(f, "Vulkan"),
            GraphicsApi::Metal => write!(f, "Metal"),
            GraphicsApi::DirectX => write!(f, "DirectX"),
            GraphicsApi::OpenGL => write!(f, "OpenGL"),
            GraphicsApi::Software => write!(f, "Software"),
        }
    }
}

/// グラフィックスモジュールのエラー型
#[derive(Debug)]
pub enum GraphicsError {
    /// 初期化エラー
    Initialization(String),
    /// レンダリングエラー
    Rendering(String),
    /// リソースエラー
    Resource(String),
    /// シェーダーエラー
    Shader(String),
    /// バックエンドエラー
    Backend(String),
    /// サポートされていないAPIエラー
    UnsupportedApi(String),
    /// その他のエラー
    Other(String),
}

impl fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphicsError::Initialization(msg) => write!(f, "グラフィックス初期化エラー: {}", msg),
            GraphicsError::Rendering(msg) => write!(f, "レンダリングエラー: {}", msg),
            GraphicsError::Resource(msg) => write!(f, "リソースエラー: {}", msg),
            GraphicsError::Shader(msg) => write!(f, "シェーダーエラー: {}", msg),
            GraphicsError::Backend(msg) => write!(f, "バックエンドエラー: {}", msg),
            GraphicsError::UnsupportedApi(msg) => write!(f, "サポートされていないAPIエラー: {}", msg),
            GraphicsError::Other(msg) => write!(f, "グラフィックスエラー: {}", msg),
        }
    }
}

impl Error for GraphicsError {}

/// グラフィックスモジュールの設定
#[derive(Debug, Clone)]
pub struct GraphicsConfig {
    /// 優先するグラフィックスAPI
    pub preferred_api: GraphicsApi,
    /// フォールバックグラフィックスAPI
    pub fallback_apis: Vec<GraphicsApi>,
    /// ハードウェアアクセラレーション有効フラグ
    pub hardware_acceleration: bool,
    /// 垂直同期有効フラグ
    pub vsync: bool,
    /// マルチサンプリングレベル（0=無効）
    pub msaa_samples: u8,
    /// 最大テクスチャサイズ
    pub max_texture_size: u32,
    /// テクスチャ圧縮有効フラグ
    pub texture_compression: bool,
    /// 最大アニソトロピックフィルタリングレベル
    pub max_anisotropy: f32,
    /// GPU省電力モード有効フラグ
    pub power_saving_mode: bool,
    /// カスタム設定
    pub custom_settings: HashMap<String, String>,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            preferred_api: GraphicsApi::Vulkan,
            fallback_apis: vec![GraphicsApi::OpenGL, GraphicsApi::Software],
            hardware_acceleration: true,
            vsync: true,
            msaa_samples: 4,
            max_texture_size: 4096,
            texture_compression: true,
            max_anisotropy: 16.0,
            power_saving_mode: false,
            custom_settings: HashMap::new(),
        }
    }
}

/// グラフィックスマネージャー
///
/// グラフィックスシステム全体を管理し、レンダラー、シェーダーマネージャー、
/// リソースマネージャーなどのコンポーネントを調整します。
pub struct GraphicsManager {
    /// 設定
    config: GraphicsConfig,
    /// 現在のグラフィックスAPI
    current_api: GraphicsApi,
    /// レンダラー
    renderer: Option<Arc<Mutex<dyn renderer::Renderer>>>,
    /// シェーダーマネージャー
    shader_manager: Option<Arc<RwLock<shader_manager::ShaderManager>>>,
    /// リソースマネージャー
    resource_manager: Option<Arc<RwLock<resource_manager::ResourceManager>>>,
    /// GPUの機能とリミット
    capabilities: HashMap<String, String>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl GraphicsManager {
    /// 新しいグラフィックスマネージャーを作成
    pub fn new() -> Self {
        Self {
            config: GraphicsConfig::default(),
            current_api: GraphicsApi::Vulkan, // デフォルト
            renderer: None,
            shader_manager: None,
            resource_manager: None,
            capabilities: HashMap::new(),
            initialized: false,
        }
    }
    
    /// 設定を指定してグラフィックスマネージャーを作成
    pub fn with_config(config: GraphicsConfig) -> Self {
        Self {
            config,
            current_api: config.preferred_api,
            renderer: None,
            shader_manager: None,
            resource_manager: None,
            capabilities: HashMap::new(),
            initialized: false,
        }
    }
    
    /// グラフィックスシステムを初期化
    pub fn initialize(&mut self) -> Result<(), GraphicsError> {
        if self.initialized {
            return Ok(());
        }
        
        // 利用可能なAPIを確認
        let available_apis = self.detect_available_apis()?;
        
        // 優先順位に基づいてAPIを選択
        self.current_api = self.select_api(&available_apis)?;
        
        // 選択されたAPIに基づいてレンダラーを初期化
        self.initialize_renderer()?;
        
        // シェーダーマネージャーを初期化
        self.initialize_shader_manager()?;
        
        // リソースマネージャーを初期化
        self.initialize_resource_manager()?;
        
        // GPUの機能とリミットを取得
        self.query_capabilities()?;
        
        self.initialized = true;
        Ok(())
    }
    
    /// 利用可能なグラフィックスAPIを検出
    fn detect_available_apis(&self) -> Result<Vec<GraphicsApi>, GraphicsError> {
        let mut available_apis = Vec::new();
        
        // ここに実際の検出ロジックを実装
        // 現在はダミーの実装
        #[cfg(feature = "vulkan")]
        available_apis.push(GraphicsApi::Vulkan);
        
        #[cfg(target_os = "macos")]
        available_apis.push(GraphicsApi::Metal);
        
        #[cfg(target_os = "windows")]
        available_apis.push(GraphicsApi::DirectX);
        
        #[cfg(feature = "opengl")]
        available_apis.push(GraphicsApi::OpenGL);
        
        // ソフトウェアレンダリングは常に利用可能
        available_apis.push(GraphicsApi::Software);
        
        if available_apis.is_empty() {
            return Err(GraphicsError::Initialization(
                "利用可能なグラフィックスAPIが見つかりません".to_string(),
            ));
        }
        
        Ok(available_apis)
    }
    
    /// 優先順位に基づいてAPIを選択
    fn select_api(&self, available_apis: &[GraphicsApi]) -> Result<GraphicsApi, GraphicsError> {
        // 優先APIが利用可能か確認
        if available_apis.contains(&self.config.preferred_api) {
            return Ok(self.config.preferred_api);
        }
        
        // フォールバックAPIを優先順位に基づいて試行
        for api in &self.config.fallback_apis {
            if available_apis.contains(api) {
                return Ok(*api);
            }
        }
        
        // 利用可能なAPIから最初のものを選択
        available_apis.first().copied().ok_or_else(|| {
            GraphicsError::Initialization("APIを選択できません".to_string())
        })
    }
    
    /// レンダラーを初期化
    fn initialize_renderer(&mut self) -> Result<(), GraphicsError> {
        self.renderer = match self.current_api {
            GraphicsApi::Vulkan => {
                #[cfg(feature = "vulkan")]
                {
                    let renderer = vulkan_backend::VulkanRenderer::new(&self.config)?;
                    Some(Arc::new(Mutex::new(renderer)))
                }
                #[cfg(not(feature = "vulkan"))]
                {
                    return Err(GraphicsError::UnsupportedApi(
                        "Vulkanバックエンドがコンパイルされていません".to_string(),
                    ));
                }
            }
            GraphicsApi::Metal => {
                #[cfg(target_os = "macos")]
                {
                    let renderer = metal_backend::MetalRenderer::new(&self.config)?;
                    Some(Arc::new(Mutex::new(renderer)))
                }
                #[cfg(not(target_os = "macos"))]
                {
                    return Err(GraphicsError::UnsupportedApi(
                        "MetalバックエンドはmacOSでのみサポートされています".to_string(),
                    ));
                }
            }
            GraphicsApi::DirectX => {
                #[cfg(target_os = "windows")]
                {
                    let renderer = dx_backend::DirectXRenderer::new(&self.config)?;
                    Some(Arc::new(Mutex::new(renderer)))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Err(GraphicsError::UnsupportedApi(
                        "DirectXバックエンドはWindowsでのみサポートされています".to_string(),
                    ));
                }
            }
            GraphicsApi::OpenGL => {
                #[cfg(feature = "opengl")]
                {
                    // OpenGLレンダラーの実装（将来的に追加予定）
                    return Err(GraphicsError::UnsupportedApi(
                        "OpenGLバックエンドは現在実装されていません".to_string(),
                    ));
                }
                #[cfg(not(feature = "opengl"))]
                {
                    return Err(GraphicsError::UnsupportedApi(
                        "OpenGLバックエンドがコンパイルされていません".to_string(),
                    ));
                }
            }
            GraphicsApi::Software => {
                // ソフトウェアレンダラーの実装（将来的に追加予定）
                return Err(GraphicsError::UnsupportedApi(
                    "ソフトウェアレンダリングは現在実装されていません".to_string(),
                ));
            }
        };
        
        Ok(())
    }
    
    /// シェーダーマネージャーを初期化
    fn initialize_shader_manager(&mut self) -> Result<(), GraphicsError> {
        let renderer_ref = self.renderer.as_ref().ok_or_else(|| {
            GraphicsError::Initialization("レンダラーが初期化されていません".to_string())
        })?;
        
        let renderer = renderer_ref.lock().map_err(|_| {
            GraphicsError::Initialization("レンダラーのロックに失敗しました".to_string())
        })?;
        
        let shader_manager = shader_manager::ShaderManager::new(self.current_api, renderer.get_device_info())?;
        self.shader_manager = Some(Arc::new(RwLock::new(shader_manager)));
        
        Ok(())
    }
    
    /// リソースマネージャーを初期化
    fn initialize_resource_manager(&mut self) -> Result<(), GraphicsError> {
        let renderer_ref = self.renderer.as_ref().ok_or_else(|| {
            GraphicsError::Initialization("レンダラーが初期化されていません".to_string())
        })?;
        
        let resource_manager = resource_manager::ResourceManager::new(
            self.current_api,
            Arc::clone(renderer_ref),
            self.config.max_texture_size,
            self.config.texture_compression,
        )?;
        
        self.resource_manager = Some(Arc::new(RwLock::new(resource_manager)));
        
        Ok(())
    }
    
    /// GPUの機能とリミットを取得
    fn query_capabilities(&mut self) -> Result<(), GraphicsError> {
        let renderer_ref = self.renderer.as_ref().ok_or_else(|| {
            GraphicsError::Initialization("レンダラーが初期化されていません".to_string())
        })?;
        
        let renderer = renderer_ref.lock().map_err(|_| {
            GraphicsError::Initialization("レンダラーのロックに失敗しました".to_string())
        })?;
        
        self.capabilities = renderer.get_capabilities();
        
        Ok(())
    }
    
    /// レンダラーを取得
    pub fn get_renderer(&self) -> Option<Arc<Mutex<dyn renderer::Renderer>>> {
        self.renderer.clone()
    }
    
    /// シェーダーマネージャーを取得
    pub fn get_shader_manager(&self) -> Option<Arc<RwLock<shader_manager::ShaderManager>>> {
        self.shader_manager.clone()
    }
    
    /// リソースマネージャーを取得
    pub fn get_resource_manager(&self) -> Option<Arc<RwLock<resource_manager::ResourceManager>>> {
        self.resource_manager.clone()
    }
    
    /// 現在のグラフィックスAPIを取得
    pub fn get_current_api(&self) -> GraphicsApi {
        self.current_api
    }
    
    /// GPUの機能とリミットを取得
    pub fn get_capabilities(&self) -> &HashMap<String, String> {
        &self.capabilities
    }
    
    /// グラフィックス設定を取得
    pub fn get_config(&self) -> &GraphicsConfig {
        &self.config
    }
    
    /// グラフィックス設定を更新
    pub fn update_config(&mut self, config: GraphicsConfig) -> Result<(), GraphicsError> {
        // 重要な設定が変更された場合は再初期化が必要
        let needs_reinit = self.config.preferred_api != config.preferred_api
            || self.config.fallback_apis != config.fallback_apis
            || self.config.hardware_acceleration != config.hardware_acceleration;
            
        self.config = config;
        
        if needs_reinit && self.initialized {
            // シャットダウンして再初期化
            self.shutdown()?;
            self.initialize()?;
        } else if self.initialized {
            // レンダラーに設定変更を通知
            if let Some(renderer_ref) = &self.renderer {
                let mut renderer = renderer_ref.lock().map_err(|_| {
                    GraphicsError::Initialization("レンダラーのロックに失敗しました".to_string())
                })?;
                
                renderer.update_config(&self.config)?;
            }
        }
        
        Ok(())
    }
    
    /// グラフィックスシステムをシャットダウン
    pub fn shutdown(&mut self) -> Result<(), GraphicsError> {
        if !self.initialized {
            return Ok(());
        }
        
        // リソースマネージャーをクリーンアップ
        if let Some(resource_manager_ref) = &self.resource_manager {
            let mut resource_manager = resource_manager_ref.write().map_err(|_| {
                GraphicsError::Other("リソースマネージャーのロックに失敗しました".to_string())
            })?;
            
            resource_manager.cleanup()?;
        }
        
        // シェーダーマネージャーをクリーンアップ
        if let Some(shader_manager_ref) = &self.shader_manager {
            let mut shader_manager = shader_manager_ref.write().map_err(|_| {
                GraphicsError::Other("シェーダーマネージャーのロックに失敗しました".to_string())
            })?;
            
            shader_manager.cleanup()?;
        }
        
        // レンダラーをシャットダウン
        if let Some(renderer_ref) = &self.renderer {
            let mut renderer = renderer_ref.lock().map_err(|_| {
                GraphicsError::Other("レンダラーのロックに失敗しました".to_string())
            })?;
            
            renderer.shutdown()?;
        }
        
        self.renderer = None;
        self.shader_manager = None;
        self.resource_manager = None;
        self.capabilities.clear();
        self.initialized = false;
        
        Ok(())
    }
}

impl Drop for GraphicsManager {
    fn drop(&mut self) {
        if self.initialized {
            // エラーは無視して最善を尽くしてクリーンアップ
            let _ = self.shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graphics_config_default() {
        let config = GraphicsConfig::default();
        assert_eq!(config.preferred_api, GraphicsApi::Vulkan);
        assert_eq!(config.vsync, true);
        assert_eq!(config.msaa_samples, 4);
    }
    
    #[test]
    fn test_graphics_api_display() {
        assert_eq!(format!("{}", GraphicsApi::Vulkan), "Vulkan");
        assert_eq!(format!("{}", GraphicsApi::Metal), "Metal");
        assert_eq!(format!("{}", GraphicsApi::DirectX), "DirectX");
        assert_eq!(format!("{}", GraphicsApi::OpenGL), "OpenGL");
        assert_eq!(format!("{}", GraphicsApi::Software), "Software");
    }
    
    #[test]
    fn test_graphics_manager_creation() {
        let manager = GraphicsManager::new();
        assert_eq!(manager.initialized, false);
        assert_eq!(manager.current_api, GraphicsApi::Vulkan);
    }
    
    #[test]
    fn test_graphics_error_display() {
        let err = GraphicsError::Initialization("初期化失敗".to_string());
        assert_eq!(format!("{}", err), "グラフィックス初期化エラー: 初期化失敗");
        
        let err = GraphicsError::Rendering("レンダリング失敗".to_string());
        assert_eq!(format!("{}", err), "レンダリングエラー: レンダリング失敗");
    }
} 