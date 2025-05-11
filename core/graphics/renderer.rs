// LumosDesktop レンダラーモジュール
// 異なるグラフィックスバックエンド向けのレンダリング抽象化層

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use super::GraphicsConfig;
use super::GraphicsError;

/// 頂点属性フォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexFormat {
    Float,
    Float2,
    Float3,
    Float4,
    Int,
    Int2,
    Int3,
    Int4,
    UInt,
    UInt2,
    UInt3,
    UInt4,
    Short2,
    Short4,
    Byte4,
    UByte4,
}

/// 頂点属性
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub format: VertexFormat,
    pub offset: u32,
    pub location: u32,
    pub divisor: u32,
}

/// 頂点レイアウト
#[derive(Debug, Clone)]
pub struct VertexLayout {
    pub attributes: Vec<VertexAttribute>,
    pub stride: u32,
}

/// プリミティブタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

/// カリングモード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CullMode {
    None,
    Front,
    Back,
}

/// ポリゴン塗りつぶしモード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FillMode {
    Point,
    Wireframe,
    Solid,
}

/// ブレンド要素
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SrcAlphaSaturate,
}

/// ブレンド操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

/// ブレンド状態
#[derive(Debug, Clone)]
pub struct BlendState {
    pub enabled: bool,
    pub src_rgb: BlendFactor,
    pub dst_rgb: BlendFactor,
    pub op_rgb: BlendOp,
    pub src_alpha: BlendFactor,
    pub dst_alpha: BlendFactor,
    pub op_alpha: BlendOp,
    pub color: [f32; 4],
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            enabled: false,
            src_rgb: BlendFactor::One,
            dst_rgb: BlendFactor::Zero,
            op_rgb: BlendOp::Add,
            src_alpha: BlendFactor::One,
            dst_alpha: BlendFactor::Zero,
            op_alpha: BlendOp::Add,
            color: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

/// 比較関数
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompareFunc {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

/// ステンシル操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    Increment,
    IncrementWrap,
    Decrement,
    DecrementWrap,
    Invert,
}

/// 深度ステンシル状態
#[derive(Debug, Clone)]
pub struct DepthStencilState {
    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_func: CompareFunc,
    pub stencil_test: bool,
    pub stencil_read_mask: u8,
    pub stencil_write_mask: u8,
    pub stencil_ref: u8,
    pub stencil_fail_op: StencilOp,
    pub stencil_depth_fail_op: StencilOp,
    pub stencil_pass_op: StencilOp,
    pub stencil_func: CompareFunc,
}

impl Default for DepthStencilState {
    fn default() -> Self {
        Self {
            depth_test: true,
            depth_write: true,
            depth_func: CompareFunc::Less,
            stencil_test: false,
            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
            stencil_ref: 0,
            stencil_fail_op: StencilOp::Keep,
            stencil_depth_fail_op: StencilOp::Keep,
            stencil_pass_op: StencilOp::Keep,
            stencil_func: CompareFunc::Always,
        }
    }
}

/// ラスタライザ状態
#[derive(Debug, Clone)]
pub struct RasterizerState {
    pub fill_mode: FillMode,
    pub cull_mode: CullMode,
    pub front_face_ccw: bool,
    pub depth_bias: f32,
    pub depth_bias_slope_scale: f32,
    pub scissor_test: bool,
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            fill_mode: FillMode::Solid,
            cull_mode: CullMode::Back,
            front_face_ccw: true,
            depth_bias: 0.0,
            depth_bias_slope_scale: 0.0,
            scissor_test: false,
        }
    }
}

/// パイプライン状態
#[derive(Debug, Clone)]
pub struct PipelineState {
    pub vertex_layout: VertexLayout,
    pub primitive_type: PrimitiveType,
    pub blend_state: BlendState,
    pub depth_stencil_state: DepthStencilState,
    pub rasterizer_state: RasterizerState,
}

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            vertex_layout: VertexLayout {
                attributes: Vec::new(),
                stride: 0,
            },
            primitive_type: PrimitiveType::Triangles,
            blend_state: BlendState::default(),
            depth_stencil_state: DepthStencilState::default(),
            rasterizer_state: RasterizerState::default(),
        }
    }
}

/// テクスチャフォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    R8,
    R8G8,
    R8G8B8,
    R8G8B8A8,
    B8G8R8A8,
    R16F,
    R16G16F,
    R16G16B16F,
    R16G16B16A16F,
    R32F,
    R32G32F,
    R32G32B32F,
    R32G32B32A32F,
    BC1, // DXT1
    BC2, // DXT3
    BC3, // DXT5
    BC7, // BPTC
    Depth16,
    Depth24,
    Depth32F,
    Depth24Stencil8,
    Depth32FStencil8,
}

/// テクスチャフィルターモード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterMode {
    Point,
    Bilinear,
    Trilinear,
    Anisotropic(u8),
}

/// テクスチャラップモード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WrapMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

/// バッファの使用方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferUsage {
    Static,
    Dynamic,
    Stream,
}

/// バッファターゲット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferTarget {
    Vertex,
    Index,
    Uniform,
    Storage,
}

/// シェーダーステージ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Geometry,
    Compute,
    TessControl,
    TessEvaluation,
}

/// レンダーターゲット
pub trait RenderTarget: fmt::Debug {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn format(&self) -> TextureFormat;
    fn samples(&self) -> u8;
}

/// レンダーパス
pub trait RenderPass: fmt::Debug {
    fn begin(&mut self) -> Result<(), GraphicsError>;
    fn end(&mut self) -> Result<(), GraphicsError>;
    fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32, min_depth: f32, max_depth: f32) -> Result<(), GraphicsError>;
    fn set_scissor(&mut self, x: i32, y: i32, width: u32, height: u32) -> Result<(), GraphicsError>;
}

/// レンダーコマンドバッファ
pub trait RenderCommandBuffer: fmt::Debug {
    fn begin(&mut self) -> Result<(), GraphicsError>;
    fn end(&mut self) -> Result<(), GraphicsError>;
    fn submit(&mut self) -> Result<(), GraphicsError>;
    
    fn clear(&mut self, color: [f32; 4], depth: f32, stencil: u8) -> Result<(), GraphicsError>;
    fn draw(&mut self, vertex_count: u32, first_vertex: u32) -> Result<(), GraphicsError>;
    fn draw_indexed(&mut self, index_count: u32, first_index: u32, vertex_offset: i32) -> Result<(), GraphicsError>;
    fn draw_instanced(&mut self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) -> Result<(), GraphicsError>;
    fn draw_indexed_instanced(&mut self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) -> Result<(), GraphicsError>;
}

/// レンダーコンテキスト
pub trait RenderContext: fmt::Debug {
    fn create_render_pass(&mut self, target: Box<dyn RenderTarget>) -> Result<Box<dyn RenderPass>, GraphicsError>;
    fn create_command_buffer(&mut self) -> Result<Box<dyn RenderCommandBuffer>, GraphicsError>;
    
    fn set_pipeline_state(&mut self, state: &PipelineState) -> Result<(), GraphicsError>;
    fn set_shader(&mut self, shader: &str) -> Result<(), GraphicsError>;
    fn set_vertex_buffer(&mut self, buffer: &str, slot: u32) -> Result<(), GraphicsError>;
    fn set_index_buffer(&mut self, buffer: &str) -> Result<(), GraphicsError>;
    fn set_uniform_buffer(&mut self, buffer: &str, slot: u32) -> Result<(), GraphicsError>;
    fn set_texture(&mut self, texture: &str, slot: u32) -> Result<(), GraphicsError>;
    
    fn update_uniform(&mut self, name: &str, data: &[u8]) -> Result<(), GraphicsError>;
}

/// レンダラーインターフェース
pub trait Renderer: fmt::Debug + Send + Sync {
    /// レンダラーの名前を取得
    fn name(&self) -> &str;
    
    /// レンダラーを初期化
    fn initialize(&mut self, config: &GraphicsConfig) -> Result<(), GraphicsError>;
    
    /// レンダラーをシャットダウン
    fn shutdown(&mut self) -> Result<(), GraphicsError>;
    
    /// 設定を更新
    fn update_config(&mut self, config: &GraphicsConfig) -> Result<(), GraphicsError>;
    
    /// レンダーコンテキストを作成
    fn create_context(&mut self) -> Result<Box<dyn RenderContext>, GraphicsError>;
    
    /// テクスチャを作成
    fn create_texture(&mut self, name: &str, width: u32, height: u32, format: TextureFormat, data: Option<&[u8]>) -> Result<(), GraphicsError>;
    
    /// バッファを作成
    fn create_buffer(&mut self, name: &str, target: BufferTarget, usage: BufferUsage, data: Option<&[u8]>, size: usize) -> Result<(), GraphicsError>;
    
    /// デバイス情報を取得
    fn get_device_info(&self) -> HashMap<String, String>;
    
    /// 機能とリミットを取得
    fn get_capabilities(&self) -> HashMap<String, String>;
} 