// LumosDesktop シェーダーマネージャー
// シェーダーの管理、コンパイル、キャッシュを担当

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use super::{GraphicsApi, GraphicsError};
use super::renderer::ShaderStage;

/// シェーダーコンパイルエラー
#[derive(Debug)]
pub enum ShaderCompilationError {
    IoError(io::Error),
    CompilationFailed(String),
    UnsupportedFormat(String),
    InvalidStage(String),
    ValidationFailed(String),
}

impl fmt::Display for ShaderCompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShaderCompilationError::IoError(err) => write!(f, "I/Oエラー: {}", err),
            ShaderCompilationError::CompilationFailed(msg) => write!(f, "コンパイル失敗: {}", msg),
            ShaderCompilationError::UnsupportedFormat(msg) => write!(f, "サポートされていないフォーマット: {}", msg),
            ShaderCompilationError::InvalidStage(msg) => write!(f, "無効なシェーダーステージ: {}", msg),
            ShaderCompilationError::ValidationFailed(msg) => write!(f, "検証失敗: {}", msg),
        }
    }
}

impl From<io::Error> for ShaderCompilationError {
    fn from(error: io::Error) -> Self {
        ShaderCompilationError::IoError(error)
    }
}

impl From<ShaderCompilationError> for GraphicsError {
    fn from(error: ShaderCompilationError) -> Self {
        GraphicsError::Shader(format!("{}", error))
    }
}

/// シェーダー種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    Glsl,
    Hlsl,
    SpirV,
    Metal,
    Wgsl,
}

impl ShaderType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" | "glsl" => Some(ShaderType::Glsl),
            "hlsl" | "vs" | "ps" | "gs" | "cs" | "hs" | "ds" => Some(ShaderType::Hlsl),
            "spv" | "spirv" => Some(ShaderType::SpirV),
            "metal" => Some(ShaderType::Metal),
            "wgsl" => Some(ShaderType::Wgsl),
            _ => None,
        }
    }
    
    pub fn is_compatible_with(&self, api: GraphicsApi) -> bool {
        match (self, api) {
            (ShaderType::Glsl, GraphicsApi::Vulkan) => true,
            (ShaderType::Glsl, GraphicsApi::OpenGL) => true,
            (ShaderType::Hlsl, GraphicsApi::DirectX) => true,
            (ShaderType::SpirV, GraphicsApi::Vulkan) => true,
            (ShaderType::Metal, GraphicsApi::Metal) => true,
            (ShaderType::Wgsl, _) => true, // 将来の拡張性のため
            _ => false,
        }
    }
}

/// シェーダーソース
#[derive(Debug, Clone)]
pub struct ShaderSource {
    pub source: String,
    pub type_: ShaderType,
    pub entry_point: String,
    pub stage: ShaderStage,
    pub defines: HashMap<String, String>,
}

/// コンパイル済みシェーダー
#[derive(Debug, Clone)]
pub struct CompiledShader {
    pub data: Vec<u8>,
    pub format: ShaderType,
    pub entry_point: String,
    pub stage: ShaderStage,
    pub timestamp: SystemTime,
}

/// シェーダー情報
#[derive(Debug, Clone)]
pub struct Shader {
    pub name: String,
    pub sources: HashMap<ShaderStage, ShaderSource>,
    pub compiled: HashMap<(GraphicsApi, ShaderStage), CompiledShader>,
    pub uniforms: HashMap<String, (u32, u32)>, // (offset, size)
    pub last_modified: SystemTime,
    pub metadata: HashMap<String, String>,
}

impl Shader {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sources: HashMap::new(),
            compiled: HashMap::new(),
            uniforms: HashMap::new(),
            last_modified: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn add_source(&mut self, stage: ShaderStage, source: ShaderSource) {
        self.sources.insert(stage, source);
        self.last_modified = SystemTime::now();
    }
    
    pub fn add_compiled(&mut self, api: GraphicsApi, stage: ShaderStage, compiled: CompiledShader) {
        self.compiled.insert((api, stage), compiled);
        self.last_modified = SystemTime::now();
    }
    
    pub fn needs_recompile(&self, api: GraphicsApi, stage: ShaderStage) -> bool {
        if let Some(compiled) = self.compiled.get(&(api, stage)) {
            if let Some(source) = self.sources.get(&stage) {
                // ソースのタイムスタンプがコンパイル済みシェーダーより新しい場合
                if self.last_modified > compiled.timestamp {
                    return true;
                }
                
                // シェーダータイプが互換性がない場合
                if !source.type_.is_compatible_with(api) {
                    return true;
                }
            } else {
                // ソースがない場合はリコンパイル不要
                return false;
            }
        } else {
            // コンパイル済みがない場合は要コンパイル
            return true;
        }
        
        false
    }
    
    pub fn get_compiled(&self, api: GraphicsApi, stage: ShaderStage) -> Option<&CompiledShader> {
        self.compiled.get(&(api, stage))
    }
}

/// シェーダーマネージャー
#[derive(Debug)]
pub struct ShaderManager {
    shaders: HashMap<String, Shader>,
    search_paths: Vec<PathBuf>,
    current_api: GraphicsApi,
    device_info: HashMap<String, String>,
    compiler_version: String,
}

impl ShaderManager {
    /// 新しいシェーダーマネージャーを作成
    pub fn new(api: GraphicsApi, device_info: HashMap<String, String>) -> Result<Self, GraphicsError> {
        Ok(Self {
            shaders: HashMap::new(),
            search_paths: vec![PathBuf::from("shaders")],
            current_api: api,
            device_info,
            compiler_version: "1.0.0".to_string(), // ダミーバージョン
        })
    }
    
    /// 検索パスを追加
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(PathBuf::from(path.as_ref()));
    }
    
    /// ファイルからシェーダーを読み込む
    pub fn load_shader<P: AsRef<Path>>(&mut self, path: P, name: Option<&str>) -> Result<String, GraphicsError> {
        let path = path.as_ref();
        let shader_name = name.map(|s| s.to_string()).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unnamed".to_string())
        });
        
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| GraphicsError::Shader("シェーダーファイルに拡張子がありません".to_string()))?;
        
        let shader_type = ShaderType::from_extension(extension)
            .ok_or_else(|| GraphicsError::Shader(format!("未知のシェーダー拡張子: {}", extension)))?;
        
        let stage = match extension {
            "vert" | "vs" => ShaderStage::Vertex,
            "frag" | "ps" => ShaderStage::Fragment,
            "geom" | "gs" => ShaderStage::Geometry,
            "comp" | "cs" => ShaderStage::Compute,
            "tesc" | "hs" => ShaderStage::TessControl,
            "tese" | "ds" => ShaderStage::TessEvaluation,
            _ => {
                return Err(GraphicsError::Shader(format!(
                    "拡張子からシェーダーステージを判断できません: {}", extension
                )));
            }
        };
        
        let source = fs::read_to_string(path)
            .map_err(|e| GraphicsError::Shader(format!("シェーダーファイルの読み込みに失敗: {}", e)))?;
        
        let mut shader = if let Some(existing) = self.shaders.get_mut(&shader_name) {
            existing.clone()
        } else {
            Shader::new(&shader_name)
        };
        
        let shader_source = ShaderSource {
            source,
            type_: shader_type,
            entry_point: "main".to_string(), // デフォルトエントリーポイント
            stage,
            defines: HashMap::new(),
        };
        
        shader.add_source(stage, shader_source);
        self.shaders.insert(shader_name.clone(), shader);
        
        // この時点でコンパイルを試みる
        let _ = self.compile_shader(&shader_name, stage);
        
        Ok(shader_name)
    }
    
    /// シェーダーをコンパイル
    pub fn compile_shader(&mut self, name: &str, stage: ShaderStage) -> Result<(), GraphicsError> {
        let shader = self.shaders.get_mut(name).ok_or_else(|| {
            GraphicsError::Shader(format!("シェーダーが見つかりません: {}", name))
        })?;
        
        if !shader.needs_recompile(self.current_api, stage) {
            return Ok(());
        }
        
        let source = shader.sources.get(&stage).ok_or_else(|| {
            GraphicsError::Shader(format!("シェーダーソースが見つかりません: {} (ステージ: {:?})", name, stage))
        })?;
        
        let compiled = match (source.type_, self.current_api) {
            (ShaderType::Glsl, GraphicsApi::Vulkan) => {
                self.compile_glsl_to_spirv(source)?
            }
            (ShaderType::Hlsl, GraphicsApi::DirectX) => {
                self.compile_hlsl(source)?
            }
            (ShaderType::Metal, GraphicsApi::Metal) => {
                self.compile_metal(source)?
            }
            (ShaderType::SpirV, GraphicsApi::Vulkan) => {
                // SPIRVはすでにコンパイル済み、検証のみ
                self.validate_spirv(source)?
            }
            _ => {
                return Err(GraphicsError::Shader(format!(
                    "互換性のないシェーダー形式とAPI: {:?} と {:?}", 
                    source.type_, self.current_api
                )));
            }
        };
        
        shader.add_compiled(self.current_api, stage, compiled);
        Ok(())
    }
    
    /// GLSLをSPIR-Vにコンパイル
    fn compile_glsl_to_spirv(&self, source: &ShaderSource) -> Result<CompiledShader, GraphicsError> {
        // 実際のコンパイルはシェーダーコンパイラライブラリを使用する
        // ここではダミーの実装
        Ok(CompiledShader {
            data: source.source.as_bytes().to_vec(), // ダミーデータ
            format: ShaderType::SpirV,
            entry_point: source.entry_point.clone(),
            stage: source.stage,
            timestamp: SystemTime::now(),
        })
    }
    
    /// HLSLをコンパイル
    fn compile_hlsl(&self, source: &ShaderSource) -> Result<CompiledShader, GraphicsError> {
        // 実際のコンパイルはDXC等を使用する
        // ここではダミーの実装
        Ok(CompiledShader {
            data: source.source.as_bytes().to_vec(), // ダミーデータ
            format: ShaderType::Hlsl,
            entry_point: source.entry_point.clone(),
            stage: source.stage,
            timestamp: SystemTime::now(),
        })
    }
    
    /// Metalをコンパイル
    fn compile_metal(&self, source: &ShaderSource) -> Result<CompiledShader, GraphicsError> {
        // 実際のコンパイルはmetal-lib等を使用する
        // ここではダミーの実装
        Ok(CompiledShader {
            data: source.source.as_bytes().to_vec(), // ダミーデータ
            format: ShaderType::Metal,
            entry_point: source.entry_point.clone(),
            stage: source.stage,
            timestamp: SystemTime::now(),
        })
    }
    
    /// SPIR-Vを検証
    fn validate_spirv(&self, source: &ShaderSource) -> Result<CompiledShader, GraphicsError> {
        // 実際の検証はSPIRV-Toolsを使用する
        // ここではダミーの実装
        Ok(CompiledShader {
            data: source.source.as_bytes().to_vec(), // ダミーデータ
            format: ShaderType::SpirV,
            entry_point: source.entry_point.clone(),
            stage: source.stage,
            timestamp: SystemTime::now(),
        })
    }
    
    /// コンパイル済みシェーダーを取得
    pub fn get_compiled_shader(&self, name: &str, stage: ShaderStage) -> Result<Option<&CompiledShader>, GraphicsError> {
        let shader = self.shaders.get(name).ok_or_else(|| {
            GraphicsError::Shader(format!("シェーダーが見つかりません: {}", name))
        })?;
        
        Ok(shader.get_compiled(self.current_api, stage))
    }
    
    /// 現在のAPIを設定
    pub fn set_current_api(&mut self, api: GraphicsApi) {
        self.current_api = api;
        
        // APIが変更された場合、必要に応じてシェーダーを再コンパイル
        for (name, _) in &self.shaders {
            let name = name.clone();
            // エラーは無視して続行
            for stage in [
                ShaderStage::Vertex,
                ShaderStage::Fragment,
                ShaderStage::Geometry,
                ShaderStage::Compute,
                ShaderStage::TessControl,
                ShaderStage::TessEvaluation,
            ] {
                let _ = self.compile_shader(&name, stage);
            }
        }
    }
    
    /// シェーダーを削除
    pub fn remove_shader(&mut self, name: &str) -> bool {
        self.shaders.remove(name).is_some()
    }
    
    /// すべてのシェーダーをクリア
    pub fn clear(&mut self) {
        self.shaders.clear();
    }
    
    /// クリーンアップ
    pub fn cleanup(&mut self) -> Result<(), GraphicsError> {
        self.clear();
        Ok(())
    }
    
    /// 利用可能なシェーダー名のリストを取得
    pub fn get_shader_names(&self) -> Vec<String> {
        self.shaders.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shader_type_from_extension() {
        assert_eq!(ShaderType::from_extension("vert"), Some(ShaderType::Glsl));
        assert_eq!(ShaderType::from_extension("frag"), Some(ShaderType::Glsl));
        assert_eq!(ShaderType::from_extension("vs"), Some(ShaderType::Hlsl));
        assert_eq!(ShaderType::from_extension("ps"), Some(ShaderType::Hlsl));
        assert_eq!(ShaderType::from_extension("metal"), Some(ShaderType::Metal));
        assert_eq!(ShaderType::from_extension("spv"), Some(ShaderType::SpirV));
        assert_eq!(ShaderType::from_extension("wgsl"), Some(ShaderType::Wgsl));
        assert_eq!(ShaderType::from_extension("unknown"), None);
    }
    
    #[test]
    fn test_shader_type_compatibility() {
        assert_eq!(ShaderType::Glsl.is_compatible_with(GraphicsApi::Vulkan), true);
        assert_eq!(ShaderType::Hlsl.is_compatible_with(GraphicsApi::DirectX), true);
        assert_eq!(ShaderType::Metal.is_compatible_with(GraphicsApi::Metal), true);
        assert_eq!(ShaderType::SpirV.is_compatible_with(GraphicsApi::Vulkan), true);
        assert_eq!(ShaderType::Glsl.is_compatible_with(GraphicsApi::DirectX), false);
        assert_eq!(ShaderType::Hlsl.is_compatible_with(GraphicsApi::Vulkan), false);
    }
    
    #[test]
    fn test_shader_creation() {
        let mut shader = Shader::new("test_shader");
        assert_eq!(shader.name, "test_shader");
        assert!(shader.sources.is_empty());
        assert!(shader.compiled.is_empty());
        
        let source = ShaderSource {
            source: "void main() {}".to_string(),
            type_: ShaderType::Glsl,
            entry_point: "main".to_string(),
            stage: ShaderStage::Vertex,
            defines: HashMap::new(),
        };
        
        shader.add_source(ShaderStage::Vertex, source);
        assert_eq!(shader.sources.len(), 1);
        assert!(shader.needs_recompile(GraphicsApi::Vulkan, ShaderStage::Vertex));
    }
    
    #[test]
    fn test_shader_manager_creation() {
        let device_info = HashMap::new();
        let manager = ShaderManager::new(GraphicsApi::Vulkan, device_info).unwrap();
        assert_eq!(manager.current_api, GraphicsApi::Vulkan);
        assert_eq!(manager.shaders.len(), 0);
        assert_eq!(manager.search_paths.len(), 1);
    }
} 