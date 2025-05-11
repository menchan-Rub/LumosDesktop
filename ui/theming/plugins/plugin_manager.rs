// LumosDesktop テーマプラグイン管理システム
// テーマエンジンを拡張するプラグインの管理

use crate::ui::theming::engine::{Theme, ThemeEngine};
use log::{info, warn, error, debug};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex, RwLock};
use serde::{Serialize, Deserialize};
use libloading::{Library, Symbol};

/// プラグインのタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// テーマエフェクト（視覚効果プラグイン）
    ThemeEffect,
    /// テーマトランスフォーマー（テーマ変換プラグイン）
    ThemeTransformer,
    /// テーマプロバイダー（外部テーマソースプラグイン）
    ThemeProvider,
    /// カラースキームジェネレーター（カラーパレット生成プラグイン）
    ColorSchemeGenerator,
    /// テーマモニター（環境変数監視プラグイン）
    ThemeMonitor,
}

/// プラグインの情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// プラグインID
    pub id: String,
    /// プラグイン名
    pub name: String,
    /// 説明
    pub description: String,
    /// バージョン
    pub version: String,
    /// 作者
    pub author: String,
    /// プラグインタイプ
    pub plugin_type: PluginType,
    /// 依存関係
    pub dependencies: Vec<String>,
    /// ファイルパス
    pub file_path: Option<PathBuf>,
    /// 有効かどうか
    pub enabled: bool,
    /// 設定スキーマ
    pub settings_schema: Option<HashMap<String, SettingSchema>>,
}

/// プラグイン設定のスキーマ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingSchema {
    /// 設定名
    pub name: String,
    /// 説明
    pub description: String,
    /// 設定タイプ
    pub setting_type: SettingType,
    /// デフォルト値
    pub default_value: serde_json::Value,
    /// 可能な値（列挙型の場合）
    pub possible_values: Option<Vec<serde_json::Value>>,
    /// 最小値（数値の場合）
    pub min_value: Option<f64>,
    /// 最大値（数値の場合）
    pub max_value: Option<f64>,
}

/// 設定タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingType {
    /// 文字列
    String,
    /// 整数
    Integer,
    /// 浮動小数点数
    Float,
    /// 真偽値
    Boolean,
    /// 色
    Color,
    /// 列挙型
    Enum,
    /// ファイルパス
    FilePath,
    /// ディレクトリパス
    DirectoryPath,
}

/// プラグイン設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    /// プラグインID
    pub plugin_id: String,
    /// 設定値
    pub values: HashMap<String, serde_json::Value>,
}

/// プラグインのアクション引数
#[derive(Debug, Clone)]
pub enum PluginActionArg {
    /// テーマ
    Theme(Theme),
    /// 文字列
    String(String),
    /// 整数
    Integer(i64),
    /// 浮動小数点数
    Float(f64),
    /// 真偽値
    Boolean(bool),
    /// JSON値
    Json(serde_json::Value),
}

/// プラグインのアクション結果
#[derive(Debug, Clone)]
pub enum PluginActionResult {
    /// テーマ
    Theme(Theme),
    /// 文字列
    String(String),
    /// 整数
    Integer(i64),
    /// 浮動小数点数
    Float(f64),
    /// 真偽値
    Boolean(bool),
    /// JSON値
    Json(serde_json::Value),
    /// 成功（値なし）
    Success,
    /// エラー
    Error(String),
}

/// プラグインAPI（プラグインが実装すべきインターフェース）
pub trait PluginApi {
    /// プラグイン情報を取得
    fn get_info(&self) -> PluginInfo;
    
    /// プラグインを初期化
    fn initialize(&mut self, settings: &PluginSettings) -> Result<(), String>;
    
    /// プラグインをシャットダウン
    fn shutdown(&mut self) -> Result<(), String>;
    
    /// アクションを実行
    fn execute_action(&mut self, action_name: &str, args: &[PluginActionArg]) -> PluginActionResult;
}

type PluginCreate = unsafe fn() -> Box<dyn PluginApi>;

/// プラグインインスタンス
struct PluginInstance {
    /// プラグイン情報
    info: PluginInfo,
    /// ライブラリハンドル
    library: Library,
    /// プラグインAPI
    api: Box<dyn PluginApi>,
    /// プラグイン設定
    settings: PluginSettings,
}

/// プラグイン管理システム
pub struct PluginManager {
    /// テーマエンジンの参照
    theme_engine: Arc<ThemeEngine>,
    /// プラグインディレクトリ
    plugin_dirs: Vec<PathBuf>,
    /// 読み込まれたプラグイン
    plugins: HashMap<String, PluginInstance>,
    /// プラグイン設定の保存パス
    settings_path: PathBuf,
}

impl PluginManager {
    /// 新しいプラグイン管理システムを作成
    pub fn new(theme_engine: Arc<ThemeEngine>) -> Self {
        // システムプラグインディレクトリ
        let mut plugin_dirs = vec![
            PathBuf::from("/usr/share/lumos/plugins/theming"),
            PathBuf::from("/usr/local/share/lumos/plugins/theming"),
        ];
        
        // ホームディレクトリのプラグインフォルダを追加
        if let Some(home) = dirs::home_dir() {
            plugin_dirs.push(home.join(".local/share/lumos/plugins/theming"));
            plugin_dirs.push(home.join(".config/lumos/plugins/theming"));
        }
        
        // 設定保存パス
        let settings_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("lumos/plugin_settings.json");
        
        Self {
            theme_engine,
            plugin_dirs,
            plugins: HashMap::new(),
            settings_path,
        }
    }
    
    /// プラグインディレクトリを追加
    pub fn add_plugin_directory<P: AsRef<Path>>(&mut self, path: P) {
        let path_buf = path.as_ref().to_path_buf();
        if !self.plugin_dirs.contains(&path_buf) {
            self.plugin_dirs.push(path_buf);
        }
    }
    
    /// プラグインを読み込む
    pub fn load_plugins(&mut self) -> Vec<Result<PluginInfo, String>> {
        let mut results = Vec::new();
        
        // 設定を読み込む
        let settings = self.load_settings();
        
        for dir in &self.plugin_dirs {
            // ディレクトリが存在するか確認
            if !dir.exists() || !dir.is_dir() {
                continue;
            }
            
            // ディレクトリ内のライブラリファイルを検索
            match fs::read_dir(dir) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            
                            // 共有ライブラリかどうかを拡張子で判断
                            if let Some(ext) = path.extension() {
                                let ext = ext.to_string_lossy();
                                let is_lib = ext == "so" || ext == "dll" || ext == "dylib";
                                
                                if is_lib {
                                    // プラグインをロード
                                    let result = self.load_plugin(&path, &settings);
                                    results.push(result);
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    warn!("ディレクトリの読み込みに失敗しました: {}: {}", dir.display(), e);
                }
            }
        }
        
        info!("{}個のプラグインを読み込みました", self.plugins.len());
        results
    }
    
    /// 単一のプラグインを読み込む
    fn load_plugin<P: AsRef<Path>>(&mut self, path: P, all_settings: &HashMap<String, PluginSettings>) -> Result<PluginInfo, String> {
        let path = path.as_ref();
        
        // ライブラリを読み込む
        let library = unsafe {
            match Library::new(path) {
                Ok(lib) => lib,
                Err(e) => return Err(format!("プラグインライブラリの読み込みに失敗しました: {}", e)),
            }
        };
        
        // プラグイン作成関数を取得
        let create_fn: Symbol<PluginCreate> = unsafe {
            match library.get(b"create_plugin") {
                Ok(func) => func,
                Err(e) => return Err(format!("プラグインのcreate_plugin関数が見つかりません: {}", e)),
            }
        };
        
        // プラグインインスタンスを作成
        let mut api = unsafe { create_fn() };
        
        // プラグイン情報を取得
        let mut info = api.get_info();
        
        // ファイルパスを設定
        info.file_path = Some(path.to_path_buf());
        
        // プラグインIDが既に存在するか確認
        if self.plugins.contains_key(&info.id) {
            return Err(format!("プラグインID「{}」は既に登録されています", info.id));
        }
        
        // プラグイン設定を取得または作成
        let settings = all_settings.get(&info.id).cloned().unwrap_or_else(|| {
            // デフォルト設定を作成
            let mut values = HashMap::new();
            
            if let Some(schema) = &info.settings_schema {
                for (key, setting_schema) in schema {
                    values.insert(key.clone(), setting_schema.default_value.clone());
                }
            }
            
            PluginSettings {
                plugin_id: info.id.clone(),
                values,
            }
        });
        
        // プラグインを初期化
        if let Err(e) = api.initialize(&settings) {
            return Err(format!("プラグインの初期化に失敗しました: {}", e));
        }
        
        // プラグインインスタンスを保存
        let instance = PluginInstance {
            info: info.clone(),
            library,
            api,
            settings,
        };
        
        self.plugins.insert(info.id.clone(), instance);
        
        Ok(info)
    }
    
    /// プラグインリストを取得
    pub fn get_plugin_list(&self) -> Vec<PluginInfo> {
        self.plugins.values()
            .map(|instance| instance.info.clone())
            .collect()
    }
    
    /// プラグイン情報を取得
    pub fn get_plugin_info(&self, plugin_id: &str) -> Option<PluginInfo> {
        self.plugins.get(plugin_id).map(|instance| instance.info.clone())
    }
    
    /// プラグインを有効化
    pub fn enable_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        if let Some(instance) = self.plugins.get_mut(plugin_id) {
            if !instance.info.enabled {
                // プラグインを初期化
                if let Err(e) = instance.api.initialize(&instance.settings) {
                    return Err(format!("プラグインの初期化に失敗しました: {}", e));
                }
                
                instance.info.enabled = true;
                
                // 設定を保存
                self.save_settings();
            }
            Ok(())
        } else {
            Err(format!("プラグインID「{}」が見つかりません", plugin_id))
        }
    }
    
    /// プラグインを無効化
    pub fn disable_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        if let Some(instance) = self.plugins.get_mut(plugin_id) {
            if instance.info.enabled {
                // プラグインをシャットダウン
                if let Err(e) = instance.api.shutdown() {
                    return Err(format!("プラグインのシャットダウンに失敗しました: {}", e));
                }
                
                instance.info.enabled = false;
                
                // 設定を保存
                self.save_settings();
            }
            Ok(())
        } else {
            Err(format!("プラグインID「{}」が見つかりません", plugin_id))
        }
    }
    
    /// プラグインのアクションを実行
    pub fn execute_plugin_action(&mut self, plugin_id: &str, action_name: &str, args: &[PluginActionArg]) -> Result<PluginActionResult, String> {
        if let Some(instance) = self.plugins.get_mut(plugin_id) {
            if !instance.info.enabled {
                return Err(format!("プラグイン「{}」は無効になっています", plugin_id));
            }
            
            // アクションを実行
            let result = instance.api.execute_action(action_name, args);
            
            // エラーチェック
            if let PluginActionResult::Error(e) = &result {
                return Err(e.clone());
            }
            
            Ok(result)
        } else {
            Err(format!("プラグインID「{}」が見つかりません", plugin_id))
        }
    }
    
    /// テーマを変換するプラグインを適用
    pub fn apply_theme_transformers(&mut self, theme: &Theme) -> Theme {
        let mut transformed_theme = theme.clone();
        
        // テーマトランスフォーマープラグインを検索
        for (id, instance) in &mut self.plugins {
            if !instance.info.enabled {
                continue;
            }
            
            if instance.info.plugin_type == PluginType::ThemeTransformer {
                // トランスフォーマーを適用
                let args = [PluginActionArg::Theme(transformed_theme.clone())];
                let result = instance.api.execute_action("transform", &args);
                
                if let PluginActionResult::Theme(new_theme) = result {
                    transformed_theme = new_theme;
                } else if let PluginActionResult::Error(e) = result {
                    error!("テーマトランスフォーマープラグイン「{}」でエラーが発生しました: {}", id, e);
                }
            }
        }
        
        transformed_theme
    }
    
    /// プラグイン設定を取得
    pub fn get_plugin_settings(&self, plugin_id: &str) -> Option<PluginSettings> {
        self.plugins.get(plugin_id).map(|instance| instance.settings.clone())
    }
    
    /// プラグイン設定を更新
    pub fn update_plugin_settings(&mut self, plugin_id: &str, settings: PluginSettings) -> Result<(), String> {
        if let Some(instance) = self.plugins.get_mut(plugin_id) {
            // 設定のバリデーション
            if let Some(schema) = &instance.info.settings_schema {
                for (key, value) in &settings.values {
                    if let Some(setting_schema) = schema.get(key) {
                        // 型チェック
                        match setting_schema.setting_type {
                            SettingType::String => {
                                if !value.is_string() {
                                    return Err(format!("設定「{}」は文字列である必要があります", key));
                                }
                            },
                            SettingType::Integer => {
                                if !value.is_i64() {
                                    return Err(format!("設定「{}」は整数である必要があります", key));
                                }
                                
                                // 範囲チェック
                                if let Some(min) = setting_schema.min_value {
                                    if value.as_i64().unwrap() < min as i64 {
                                        return Err(format!("設定「{}」は{}以上である必要があります", key, min));
                                    }
                                }
                                
                                if let Some(max) = setting_schema.max_value {
                                    if value.as_i64().unwrap() > max as i64 {
                                        return Err(format!("設定「{}」は{}以下である必要があります", key, max));
                                    }
                                }
                            },
                            SettingType::Float => {
                                if !value.is_f64() {
                                    return Err(format!("設定「{}」は浮動小数点数である必要があります", key));
                                }
                                
                                // 範囲チェック
                                if let Some(min) = setting_schema.min_value {
                                    if value.as_f64().unwrap() < min {
                                        return Err(format!("設定「{}」は{}以上である必要があります", key, min));
                                    }
                                }
                                
                                if let Some(max) = setting_schema.max_value {
                                    if value.as_f64().unwrap() > max {
                                        return Err(format!("設定「{}」は{}以下である必要があります", key, max));
                                    }
                                }
                            },
                            SettingType::Boolean => {
                                if !value.is_boolean() {
                                    return Err(format!("設定「{}」は真偽値である必要があります", key));
                                }
                            },
                            SettingType::Color => {
                                if !value.is_string() {
                                    return Err(format!("設定「{}」は色コード文字列である必要があります", key));
                                }
                                
                                // 色コードのバリデーション
                                let color_str = value.as_str().unwrap();
                                if !color_str.starts_with('#') || (color_str.len() != 7 && color_str.len() != 9) {
                                    return Err(format!("設定「{}」は有効な色コード (#RRGGBB または #RRGGBBAA) である必要があります", key));
                                }
                            },
                            SettingType::Enum => {
                                if let Some(possible_values) = &setting_schema.possible_values {
                                    if !possible_values.contains(value) {
                                        return Err(format!("設定「{}」は許可された値のいずれかである必要があります", key));
                                    }
                                }
                            },
                            SettingType::FilePath | SettingType::DirectoryPath => {
                                if !value.is_string() {
                                    return Err(format!("設定「{}」はファイルパス文字列である必要があります", key));
                                }
                            },
                        }
                    } else {
                        return Err(format!("不明な設定キー「{}」が指定されました", key));
                    }
                }
            }
            
            // プラグインに設定を適用
            if let Err(e) = instance.api.initialize(&settings) {
                return Err(format!("プラグイン設定の適用に失敗しました: {}", e));
            }
            
            // 設定を更新
            instance.settings = settings;
            
            // 設定を保存
            self.save_settings();
            
            Ok(())
        } else {
            Err(format!("プラグインID「{}」が見つかりません", plugin_id))
        }
    }
    
    /// 設定を読み込む
    fn load_settings(&self) -> HashMap<String, PluginSettings> {
        if self.settings_path.exists() {
            match fs::read_to_string(&self.settings_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(settings) => settings,
                        Err(e) => {
                            error!("プラグイン設定の解析に失敗しました: {}", e);
                            HashMap::new()
                        }
                    }
                },
                Err(e) => {
                    error!("プラグイン設定ファイルの読み込みに失敗しました: {}", e);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        }
    }
    
    /// 設定を保存
    fn save_settings(&self) {
        // 設定を収集
        let mut settings = HashMap::new();
        for (id, instance) in &self.plugins {
            settings.insert(id.clone(), instance.settings.clone());
        }
        
        // JSONに変換
        let json = match serde_json::to_string_pretty(&settings) {
            Ok(json) => json,
            Err(e) => {
                error!("プラグイン設定のシリアライズに失敗しました: {}", e);
                return;
            }
        };
        
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = self.settings_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    error!("設定ディレクトリの作成に失敗しました: {}", e);
                    return;
                }
            }
        }
        
        // ファイルに保存
        if let Err(e) = fs::write(&self.settings_path, json) {
            error!("プラグイン設定の保存に失敗しました: {}", e);
        }
    }
    
    /// すべてのプラグインをアンロード
    pub fn unload_all_plugins(&mut self) {
        for (id, instance) in self.plugins.drain() {
            if instance.info.enabled {
                if let Err(e) = instance.api.shutdown() {
                    error!("プラグイン「{}」のシャットダウンに失敗しました: {}", id, e);
                }
            }
            
            // Libraryはドロップ時に自動的にアンロードされる
        }
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        self.unload_all_plugins();
    }
}

/// プラグイン作成マクロ（プラグイン開発用）
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn create_plugin() -> Box<dyn $crate::ui::theming::plugins::plugin_manager::PluginApi> {
            Box::new(<$plugin_type>::new())
        }
    };
}

/// テーマトランスフォーマープラグインの基本実装
pub trait ThemeTransformer: PluginApi {
    /// テーマを変換
    fn transform_theme(&mut self, theme: &Theme) -> Result<Theme, String>;
}

/// カラースキームジェネレータープラグインの基本実装
pub trait ColorSchemeGenerator: PluginApi {
    /// カラーパレットを生成
    fn generate_color_scheme(&mut self, base_color: &str) -> Result<ColorPalette, String>;
}

/// テーマエフェクトプラグインの基本実装
pub trait ThemeEffect: PluginApi {
    /// エフェクトを適用
    fn apply_effect(&mut self, theme: &Theme, strength: f32) -> Result<Theme, String>;
}

/// プラグインAPI実装のサンプル
#[cfg(test)]
mod sample_plugin {
    use super::*;
    
    pub struct SampleThemeTransformer {
        info: PluginInfo,
        settings: PluginSettings,
    }
    
    impl SampleThemeTransformer {
        pub fn new() -> Self {
            let mut settings_schema = HashMap::new();
            settings_schema.insert("brightness_adjust".to_string(), SettingSchema {
                name: "Brightness Adjustment".to_string(),
                description: "Adjust theme brightness".to_string(),
                setting_type: SettingType::Float,
                default_value: serde_json::json!(1.0),
                possible_values: None,
                min_value: Some(0.5),
                max_value: Some(1.5),
            });
            
            let info = PluginInfo {
                id: "sample_transformer".to_string(),
                name: "Sample Theme Transformer".to_string(),
                description: "A sample theme transformer plugin".to_string(),
                version: "1.0.0".to_string(),
                author: "LumosDesktop Team".to_string(),
                plugin_type: PluginType::ThemeTransformer,
                dependencies: Vec::new(),
                file_path: None,
                enabled: false,
                settings_schema: Some(settings_schema),
            };
            
            let settings = PluginSettings {
                plugin_id: info.id.clone(),
                values: HashMap::new(),
            };
            
            Self {
                info,
                settings,
            }
        }
    }
    
    impl PluginApi for SampleThemeTransformer {
        fn get_info(&self) -> PluginInfo {
            self.info.clone()
        }
        
        fn initialize(&mut self, settings: &PluginSettings) -> Result<(), String> {
            self.settings = settings.clone();
            Ok(())
        }
        
        fn shutdown(&mut self) -> Result<(), String> {
            Ok(())
        }
        
        fn execute_action(&mut self, action_name: &str, args: &[PluginActionArg]) -> PluginActionResult {
            match action_name {
                "transform" => {
                    if args.len() != 1 {
                        return PluginActionResult::Error("Invalid arguments for transform action".to_string());
                    }
                    
                    if let PluginActionArg::Theme(theme) = &args[0] {
                        match self.transform_theme(theme) {
                            Ok(transformed) => PluginActionResult::Theme(transformed),
                            Err(e) => PluginActionResult::Error(e),
                        }
                    } else {
                        PluginActionResult::Error("Expected Theme argument".to_string())
                    }
                },
                _ => PluginActionResult::Error(format!("Unknown action: {}", action_name)),
            }
        }
    }
    
    impl ThemeTransformer for SampleThemeTransformer {
        fn transform_theme(&mut self, theme: &Theme) -> Result<Theme, String> {
            let mut transformed = theme.clone();
            
            // 明るさ調整
            let brightness_adjust = self.settings.values.get("brightness_adjust")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
                
            // 全色に明るさ調整を適用
            if let Ok(mut foreground) = super::super::engine::color::RGB::from_hex(&theme.colors.foreground) {
                foreground.adjust_brightness(brightness_adjust);
                transformed.colors.foreground = foreground.to_hex();
            }
            
            if let Ok(mut primary) = super::super::engine::color::RGB::from_hex(&theme.colors.primary) {
                primary.adjust_brightness(brightness_adjust);
                transformed.colors.primary = primary.to_hex();
            }
            
            if let Ok(mut secondary) = super::super::engine::color::RGB::from_hex(&theme.colors.secondary) {
                secondary.adjust_brightness(brightness_adjust);
                transformed.colors.secondary = secondary.to_hex();
            }
            
            if let Ok(mut accent) = super::super::engine::color::RGB::from_hex(&theme.colors.accent) {
                accent.adjust_brightness(brightness_adjust);
                transformed.colors.accent = accent.to_hex();
            }
            
            Ok(transformed)
        }
    }
    
    // プラグイン宣言
    // declare_plugin!(SampleThemeTransformer);
} 