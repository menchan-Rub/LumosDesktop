// LumosDesktop テーマエンジン
// デスクトップ環境のテーマ管理システム

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};

// サブモジュールを公開
pub mod theme_effects;
pub mod dynamic_theme;

/// テーマのカラーパレット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    /// プライマリーカラー
    pub primary: String,
    /// セカンダリーカラー
    pub secondary: String,
    /// アクセントカラー
    pub accent: String,
    /// 背景色
    pub background: String,
    /// 前景色（テキストなど）
    pub foreground: String,
    /// 成功を表す色
    pub success: String,
    /// 警告を表す色
    pub warning: String,
    /// エラーを表す色
    pub error: String,
    /// 情報を表す色
    pub info: String,
    /// 無効状態の色
    pub disabled: String,
    /// カスタム色のマップ
    pub custom: HashMap<String, String>,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: "#1a73e8".to_string(),
            secondary: "#8ab4f8".to_string(),
            accent: "#d93025".to_string(),
            background: "#ffffff".to_string(),
            foreground: "#202124".to_string(),
            success: "#0f9d58".to_string(),
            warning: "#f29900".to_string(),
            error: "#d93025".to_string(),
            info: "#1a73e8".to_string(),
            disabled: "#5f6368".to_string(),
            custom: HashMap::new(),
        }
    }
}

/// フォント設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    /// デフォルトフォントファミリー
    pub family: String,
    /// 見出しフォントファミリー（オプション）
    pub heading_family: Option<String>,
    /// モノスペースフォントファミリー
    pub monospace_family: String,
    /// ベースフォントサイズ（ピクセル）
    pub base_size: u8,
    /// フォントの太さ
    pub weight: u16,
    /// 行の高さ（倍率）
    pub line_height: f32,
    /// フォントレンダリング設定
    pub rendering: FontRendering,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            family: "Noto Sans".to_string(),
            heading_family: None,
            monospace_family: "Noto Sans Mono".to_string(),
            base_size: 14,
            weight: 400,
            line_height: 1.5,
            rendering: FontRendering::default(),
        }
    }
}

/// フォントレンダリング設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontRendering {
    /// アンチエイリアス
    pub antialias: bool,
    /// サブピクセルレンダリング
    pub subpixel: bool,
    /// ヒンティング
    pub hinting: FontHinting,
    /// 自動ヒンティング
    pub autohint: bool,
}

impl Default for FontRendering {
    fn default() -> Self {
        Self {
            antialias: true,
            subpixel: true,
            hinting: FontHinting::Medium,
            autohint: false,
        }
    }
}

/// フォントヒンティングレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontHinting {
    /// ヒンティングなし
    None,
    /// 軽度ヒンティング
    Slight,
    /// 中程度ヒンティング
    Medium,
    /// 完全ヒンティング
    Full,
}

/// アイコンテーマ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconTheme {
    /// テーマ名
    pub name: String,
    /// テーマパス
    pub path: PathBuf,
    /// 親テーマ（フォールバック用）
    pub parent: Option<String>,
    /// アイコンディレクトリ
    pub directories: Vec<PathBuf>,
    /// スケーリング設定
    pub scale: f32,
}

/// テーマの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    /// ライトテーマ
    Light,
    /// ダークテーマ
    Dark,
    /// 自動（システム設定に従う）
    Auto,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// アニメーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSettings {
    /// アニメーションを有効にするか
    pub enabled: bool,
    /// アニメーション速度の係数
    pub speed_factor: f32,
    /// トランジションの長さ（ミリ秒）
    pub transition_ms: u32,
    /// 減衰関数
    pub easing: EasingFunction,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            speed_factor: 1.0,
            transition_ms: 250,
            easing: EasingFunction::EaseOutCubic,
        }
    }
}

/// イージング関数の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EasingFunction {
    /// 線形
    Linear,
    /// イーズイン（加速）
    EaseIn,
    /// イーズアウト（減速）
    EaseOut,
    /// イーズインアウト（加速後減速）
    EaseInOut,
    /// イーズアウト（3次曲線）
    EaseOutCubic,
    /// イーズインアウト（正弦波）
    EaseInOutSine,
    /// バウンス
    Bounce,
    /// スプリング
    Spring,
}

/// ウィジェットスタイル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetStyle {
    /// ボタンの丸み（ピクセル）
    pub button_radius: u8,
    /// インプットの丸み（ピクセル）
    pub input_radius: u8,
    /// カードの丸み（ピクセル）
    pub card_radius: u8,
    /// 影の強さ
    pub shadow_strength: f32,
    /// ボーダーの太さ（ピクセル）
    pub border_width: u8,
    /// フォーカスリングの太さ（ピクセル）
    pub focus_ring_width: u8,
    /// コントロールの内部パディング（ピクセル）
    pub control_padding: u8,
}

impl Default for WidgetStyle {
    fn default() -> Self {
        Self {
            button_radius: 4,
            input_radius: 4,
            card_radius: 8,
            shadow_strength: 0.2,
            border_width: 1,
            focus_ring_width: 2,
            control_padding: 8,
        }
    }
}

/// ディスプレイ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// スケールファクター
    pub scale_factor: f32,
    /// 高DPI対応
    pub hidpi_mode: HiDpiMode,
    /// テキストのシャープネス
    pub text_sharpness: f32,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            hidpi_mode: HiDpiMode::Auto,
            text_sharpness: 1.0,
        }
    }
}

/// 高DPIモード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HiDpiMode {
    /// 自動検出
    Auto,
    /// 標準DPI
    Normal,
    /// 高DPI
    HiDpi,
    /// カスタム
    Custom,
}

/// テーマ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// テーマ名
    pub name: String,
    /// テーマの作者
    pub author: Option<String>,
    /// テーマの説明
    pub description: Option<String>,
    /// テーマのバージョン
    pub version: Option<String>,
    /// テーマモード
    pub mode: ThemeMode,
    /// カラーパレット
    pub colors: ColorPalette,
    /// フォント設定
    pub fonts: FontSettings,
    /// ウィジェットスタイル
    pub widget_style: WidgetStyle,
    /// アニメーション設定
    pub animations: AnimationSettings,
    /// ディスプレイ設定
    pub display: DisplaySettings,
    /// アイコンテーマ
    pub icon_theme: Option<String>,
    /// カーソルテーマ
    pub cursor_theme: Option<String>,
    /// 背景画像パス
    pub wallpaper: Option<PathBuf>,
    /// カスタム設定
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            author: Some("Lumos Team".to_string()),
            description: Some("Default Lumos Desktop theme".to_string()),
            version: Some("1.0.0".to_string()),
            mode: ThemeMode::Light,
            colors: ColorPalette::default(),
            fonts: FontSettings::default(),
            widget_style: WidgetStyle::default(),
            animations: AnimationSettings::default(),
            display: DisplaySettings::default(),
            icon_theme: Some("Lumos-Icons".to_string()),
            cursor_theme: Some("Lumos-Cursors".to_string()),
            wallpaper: None,
            custom: HashMap::new(),
        }
    }
}

/// テーマ変更イベント
#[derive(Debug, Clone)]
pub struct ThemeChangedEvent {
    /// 以前のテーマ名
    pub previous_theme: String,
    /// 新しいテーマ名
    pub new_theme: String,
    /// テーマモードが変更されたかどうか
    pub mode_changed: bool,
}

/// テーマ変更リスナー
pub type ThemeChangeListener = Box<dyn Fn(&ThemeChangedEvent) + Send + Sync + 'static>;

/// カラーユーティリティ
pub mod color {
    use std::num::ParseIntError;

    /// RGB色
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RGB {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    /// RGBA色
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct RGBA {
        pub r: u8,
        pub g: u8,
        pub b: u8,
        pub a: f32,
    }

    /// HSL色
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct HSL {
        pub h: f32, // 0.0 - 360.0
        pub s: f32, // 0.0 - 1.0
        pub l: f32, // 0.0 - 1.0
    }

    impl RGB {
        /// 新しいRGB色を作成
        pub fn new(r: u8, g: u8, b: u8) -> Self {
            Self { r, g, b }
        }

        /// 16進数文字列からRGB色を作成
        pub fn from_hex(hex: &str) -> Result<Self, ParseIntError> {
            let hex = hex.trim_start_matches('#');
            
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16)?;
                let g = u8::from_str_radix(&hex[2..4], 16)?;
                let b = u8::from_str_radix(&hex[4..6], 16)?;
                Ok(Self { r, g, b })
            } else if hex.len() == 3 {
                let r = u8::from_str_radix(&hex[0..1], 16)?;
                let g = u8::from_str_radix(&hex[1..2], 16)?;
                let b = u8::from_str_radix(&hex[2..3], 16)?;
                Ok(Self { 
                    r: r * 17, 
                    g: g * 17, 
                    b: b * 17 
                })
            } else {
                Err(ParseIntError::empty())
            }
        }

        /// 16進数文字列に変換
        pub fn to_hex(&self) -> String {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        }

        /// HSLに変換
        pub fn to_hsl(&self) -> HSL {
            let r = self.r as f32 / 255.0;
            let g = self.g as f32 / 255.0;
            let b = self.b as f32 / 255.0;
            
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let delta = max - min;
            
            let l = (max + min) / 2.0;
            
            if delta == 0.0 {
                HSL {
                    h: 0.0,
                    s: 0.0,
                    l,
                }
            } else {
                let s = if l <= 0.5 {
                    delta / (max + min)
                } else {
                    delta / (2.0 - max - min)
                };
                
                let h = if r == max {
                    (g - b) / delta + (if g < b { 6.0 } else { 0.0 })
                } else if g == max {
                    (b - r) / delta + 2.0
                } else {
                    (r - g) / delta + 4.0
                };
                
                HSL {
                    h: h * 60.0,
                    s,
                    l,
                }
            }
        }

        /// 透明度を追加
        pub fn with_alpha(&self, alpha: f32) -> RGBA {
            RGBA {
                r: self.r,
                g: self.g,
                b: self.b,
                a: alpha.max(0.0).min(1.0),
            }
        }
    }

    impl RGBA {
        /// 新しいRGBA色を作成
        pub fn new(r: u8, g: u8, b: u8, a: f32) -> Self {
            Self { 
                r, 
                g, 
                b, 
                a: a.max(0.0).min(1.0),
            }
        }

        /// CSSの rgba() 文字列に変換
        pub fn to_css_rgba(&self) -> String {
            format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a)
        }
    }

    impl HSL {
        /// 新しいHSL色を作成
        pub fn new(h: f32, s: f32, l: f32) -> Self {
            Self { 
                h: h % 360.0, 
                s: s.max(0.0).min(1.0), 
                l: l.max(0.0).min(1.0),
            }
        }

        /// RGBに変換
        pub fn to_rgb(&self) -> RGB {
            if self.s == 0.0 {
                let l = (self.l * 255.0) as u8;
                return RGB { r: l, g: l, b: l };
            }
            
            let q = if self.l < 0.5 {
                self.l * (1.0 + self.s)
            } else {
                self.l + self.s - self.l * self.s
            };
            
            let p = 2.0 * self.l - q;
            let h = self.h / 360.0;
            
            let tr = h + 1.0/3.0;
            let tg = h;
            let tb = h - 1.0/3.0;
            
            let r = hue_to_rgb(p, q, tr);
            let g = hue_to_rgb(p, q, tg);
            let b = hue_to_rgb(p, q, tb);
            
            RGB {
                r: (r * 255.0) as u8,
                g: (g * 255.0) as u8,
                b: (b * 255.0) as u8,
            }
        }
    }

    /// HSLのヘルパー関数
    fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
        let t = if t < 0.0 {
            t + 1.0
        } else if t > 1.0 {
            t - 1.0
        } else {
            t
        };
        
        if t < 1.0/6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 1.0/2.0 {
            q
        } else if t < 2.0/3.0 {
            p + (q - p) * (2.0/3.0 - t) * 6.0
        } else {
            p
        }
    }
}

/// テーマエンジン
pub struct ThemeEngine {
    /// 現在のテーマ
    current_theme: RwLock<Theme>,
    /// 登録されたテーマのマップ
    themes: RwLock<HashMap<String, Theme>>,
    /// テーマディレクトリのパス
    theme_paths: RwLock<Vec<PathBuf>>,
    /// アイコンテーマディレクトリのパス
    icon_paths: RwLock<Vec<PathBuf>>,
    /// カーソルテーマディレクトリのパス
    cursor_paths: RwLock<Vec<PathBuf>>,
    /// ダークモード検出機能
    dark_mode_detector: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    /// テーマ変更リスナー
    change_listeners: RwLock<Vec<ThemeChangeListener>>,
    /// 高DPIスケール検出機能
    hidpi_detector: Option<Box<dyn Fn() -> f32 + Send + Sync>>,
    /// エフェクトマネージャー
    effect_manager: RwLock<theme_effects::EffectManager>,
    /// シーンブレンドマネージャー
    scene_blend_manager: RwLock<theme_effects::SceneBlendManager>,
    /// 動的テーママネージャー
    dynamic_manager: Option<Arc<RwLock<dynamic_theme::DynamicThemeManager>>>,
}

impl ThemeEngine {
    /// 新しいテーマエンジンを作成
    pub fn new() -> Self {
        Self {
            current_theme: RwLock::new(Theme::default()),
            themes: RwLock::new(HashMap::new()),
            theme_paths: RwLock::new(Vec::new()),
            icon_paths: RwLock::new(Vec::new()),
            cursor_paths: RwLock::new(Vec::new()),
            dark_mode_detector: None,
            change_listeners: RwLock::new(Vec::new()),
            hidpi_detector: None,
            effect_manager: RwLock::new(theme_effects::EffectManager::new()),
            scene_blend_manager: RwLock::new(theme_effects::SceneBlendManager::new()),
            dynamic_manager: None,
        }
    }

    /// テーマパスを追加
    pub fn add_theme_path<P: AsRef<Path>>(&self, path: P) {
        let mut paths = self.theme_paths.write().unwrap();
        let path_buf = path.as_ref().to_path_buf();
        if !paths.contains(&path_buf) {
            paths.push(path_buf);
        }
    }

    /// アイコンテーマパスを追加
    pub fn add_icon_path<P: AsRef<Path>>(&self, path: P) {
        let mut paths = self.icon_paths.write().unwrap();
        let path_buf = path.as_ref().to_path_buf();
        if !paths.contains(&path_buf) {
            paths.push(path_buf);
        }
    }

    /// カーソルテーマパスを追加
    pub fn add_cursor_path<P: AsRef<Path>>(&self, path: P) {
        let mut paths = self.cursor_paths.write().unwrap();
        let path_buf = path.as_ref().to_path_buf();
        if !paths.contains(&path_buf) {
            paths.push(path_buf);
        }
    }

    /// テーマファイルを読み込み
    pub fn load_theme<P: AsRef<Path>>(&self, path: P) -> Result<Theme, String> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(format!("テーマファイルが存在しません: {}", path.display()));
        }
        
        let file = std::fs::File::open(path)
            .map_err(|e| format!("テーマファイルを開けませんでした: {}", e))?;
            
        let theme: Theme = serde_json::from_reader(file)
            .map_err(|e| format!("テーマファイルの解析に失敗しました: {}", e))?;
            
        Ok(theme)
    }

    /// テーマをインストール
    pub fn install_theme(&self, theme: Theme) {
        let mut themes = self.themes.write().unwrap();
        themes.insert(theme.name.clone(), theme);
    }

    /// テーマを名前で設定し、ブレンドエフェクトを使用
    pub fn set_theme_with_blend(&self, name: &str, blend_type: &str) -> Result<(), String> {
        let themes = self.themes.read().unwrap();
        
        if let Some(theme) = themes.get(name) {
            let previous_theme_name;
            let mode_changed;
            
            {
                let current = self.current_theme.read().unwrap();
                previous_theme_name = current.name.clone();
                mode_changed = current.mode != theme.mode;
            }
            
            // ブレンドを開始
            let mut blend_manager = self.scene_blend_manager.write().unwrap();
            blend_manager.start_theme_blend(&previous_theme_name, name, blend_type);
            
            // テーマを設定
            {
                let mut current = self.current_theme.write().unwrap();
                *current = theme.clone();
            }
            
            info!("テーマを変更しました (ブレンド: {}): {}", blend_type, name);
            
            // リスナーに通知
            let event = ThemeChangedEvent {
                previous_theme: previous_theme_name,
                new_theme: name.to_string(),
                mode_changed,
            };
            
            self.notify_listeners(&event);
            
            Ok(())
        } else {
            Err(format!("テーマが見つかりません: {}", name))
        }
    }

    /// 現在のテーマを取得
    pub fn get_current_theme(&self) -> Theme {
        self.current_theme.read().unwrap().clone()
    }

    /// カラーパレットを取得
    pub fn get_color_palette(&self) -> ColorPalette {
        self.current_theme.read().unwrap().colors.clone()
    }

    /// フォント設定を取得
    pub fn get_font_settings(&self) -> FontSettings {
        self.current_theme.read().unwrap().fonts.clone()
    }

    /// ウィジェットスタイルを取得
    pub fn get_widget_style(&self) -> WidgetStyle {
        self.current_theme.read().unwrap().widget_style.clone()
    }

    /// アニメーション設定を取得
    pub fn get_animation_settings(&self) -> AnimationSettings {
        self.current_theme.read().unwrap().animations.clone()
    }

    /// カラーを取得
    pub fn get_color(&self, name: &str) -> Option<String> {
        let theme = self.current_theme.read().unwrap();
        
        match name {
            "primary" => Some(theme.colors.primary.clone()),
            "secondary" => Some(theme.colors.secondary.clone()),
            "accent" => Some(theme.colors.accent.clone()),
            "background" => Some(theme.colors.background.clone()),
            "foreground" => Some(theme.colors.foreground.clone()),
            "success" => Some(theme.colors.success.clone()),
            "warning" => Some(theme.colors.warning.clone()),
            "error" => Some(theme.colors.error.clone()),
            "info" => Some(theme.colors.info.clone()),
            "disabled" => Some(theme.colors.disabled.clone()),
            _ => theme.colors.custom.get(name).cloned(),
        }
    }

    /// RGB色を取得
    pub fn get_color_rgb(&self, name: &str) -> Option<color::RGB> {
        self.get_color(name)
            .and_then(|hex| color::RGB::from_hex(&hex).ok())
    }

    /// ダークモード検出の設定
    pub fn set_dark_mode_detector<F>(&mut self, detector: F)
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.dark_mode_detector = Some(Box::new(detector));
    }

    /// システムテーマモードを取得
    pub fn is_system_dark_mode(&self) -> bool {
        if let Some(detector) = &self.dark_mode_detector {
            detector()
        } else {
            false
        }
    }

    /// テーマモードがダークかどうか
    pub fn is_dark_mode(&self) -> bool {
        let theme = self.current_theme.read().unwrap();
        
        match theme.mode {
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
            ThemeMode::Auto => self.is_system_dark_mode(),
        }
    }

    /// ダークモードに切り替え
    pub fn switch_to_dark_mode(&self) {
        let mut theme = self.current_theme.write().unwrap();
        theme.mode = ThemeMode::Dark;
    }

    /// ライトモードに切り替え
    pub fn switch_to_light_mode(&self) {
        let mut theme = self.current_theme.write().unwrap();
        theme.mode = ThemeMode::Light;
    }

    /// 自動モードに切り替え
    pub fn switch_to_auto_mode(&self) {
        let mut theme = self.current_theme.write().unwrap();
        theme.mode = ThemeMode::Auto;
    }

    /// 利用可能なすべてのテーマ名を取得
    pub fn get_available_themes(&self) -> Vec<String> {
        let themes = self.themes.read().unwrap();
        themes.keys().cloned().collect()
    }

    /// テーマ変更リスナーを追加
    pub fn add_theme_change_listener<F>(&self, listener: F)
    where
        F: Fn(&ThemeChangedEvent) + Send + Sync + 'static,
    {
        let mut listeners = self.change_listeners.write().unwrap();
        listeners.push(Box::new(listener));
    }
    
    /// テーマ変更リスナーに通知
    fn notify_listeners(&self, event: &ThemeChangedEvent) {
        let listeners = self.change_listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(event);
        }
    }
    
    /// 高DPI検出の設定
    pub fn set_hidpi_detector<F>(&mut self, detector: F)
    where
        F: Fn() -> f32 + Send + Sync + 'static,
    {
        self.hidpi_detector = Some(Box::new(detector));
    }
    
    /// システムのスケールファクターを取得
    pub fn get_system_scale_factor(&self) -> f32 {
        if let Some(detector) = &self.hidpi_detector {
            detector()
        } else {
            1.0
        }
    }
    
    /// 現在のスケールファクターを取得
    pub fn get_scale_factor(&self) -> f32 {
        let theme = self.current_theme.read().unwrap();
        
        match theme.display.hidpi_mode {
            HiDpiMode::Auto => self.get_system_scale_factor(),
            HiDpiMode::Normal => 1.0,
            HiDpiMode::HiDpi => 2.0,
            HiDpiMode::Custom => theme.display.scale_factor,
        }
    }
    
    /// ディスプレイ設定を取得
    pub fn get_display_settings(&self) -> DisplaySettings {
        self.current_theme.read().unwrap().display.clone()
    }
    
    /// スケールファクターを設定
    pub fn set_scale_factor(&self, scale: f32) {
        let mut theme = self.current_theme.write().unwrap();
        theme.display.scale_factor = scale;
        theme.display.hidpi_mode = HiDpiMode::Custom;
    }

    /// エフェクトマネージャーを取得
    pub fn get_effect_manager(&self) -> theme_effects::EffectManager {
        self.effect_manager.read().unwrap().clone()
    }
    
    /// エフェクトマネージャーに対する操作を実行
    pub fn with_effect_manager<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut theme_effects::EffectManager) -> R,
    {
        let mut manager = self.effect_manager.write().unwrap();
        f(&mut manager)
    }
    
    /// ブレンドマネージャーを取得
    pub fn get_scene_blend_manager(&self) -> theme_effects::SceneBlendManager {
        self.scene_blend_manager.read().unwrap().clone()
    }
    
    /// ブレンドマネージャーに対する操作を実行
    pub fn with_scene_blend_manager<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut theme_effects::SceneBlendManager) -> R,
    {
        let mut manager = self.scene_blend_manager.write().unwrap();
        f(&mut manager)
    }
    
    /// 動的テーママネージャーを初期化
    pub fn init_dynamic_theme_manager(&mut self) -> Arc<RwLock<dynamic_theme::DynamicThemeManager>> {
        if let Some(manager) = &self.dynamic_manager {
            return manager.clone();
        }
        
        let engine_arc = Arc::new(self.clone());
        let manager = dynamic_theme::DynamicThemeManager::new(engine_arc);
        let manager_arc = Arc::new(RwLock::new(manager));
        self.dynamic_manager = Some(manager_arc.clone());
        manager_arc
    }
    
    /// 動的テーママネージャーを取得
    pub fn get_dynamic_theme_manager(&self) -> Option<Arc<RwLock<dynamic_theme::DynamicThemeManager>>> {
        self.dynamic_manager.clone()
    }
    
    /// 動的テーマを有効化
    pub fn enable_dynamic_theme(&mut self, settings: Option<dynamic_theme::DynamicThemeSettings>) {
        let manager_arc = self.init_dynamic_theme_manager();
        let mut manager = manager_arc.write().unwrap();
        
        // 設定を適用
        if let Some(settings) = settings {
            manager.set_settings(settings);
        }
        
        // 動的テーマを開始
        manager.start();
    }
    
    /// 動的テーマを無効化
    pub fn disable_dynamic_theme(&self) {
        if let Some(manager_arc) = &self.dynamic_manager {
            let mut manager = manager_arc.write().unwrap();
            manager.stop();
        }
    }
}

impl Default for ThemeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_color_conversions() {
        let rgb = color::RGB::new(255, 0, 0);
        assert_eq!(rgb.to_hex(), "#ff0000");
        
        let rgb_from_hex = color::RGB::from_hex("#ff0000").unwrap();
        assert_eq!(rgb, rgb_from_hex);
        
        let rgb_from_short_hex = color::RGB::from_hex("#f00").unwrap();
        assert_eq!(rgb, rgb_from_short_hex);
        
        let hsl = rgb.to_hsl();
        assert_eq!(hsl.h, 0.0);
        assert_eq!(hsl.s, 1.0);
        assert_eq!(hsl.l, 0.5);
        
        let rgb_from_hsl = hsl.to_rgb();
        assert_eq!(rgb, rgb_from_hsl);
    }

    #[test]
    fn test_theme_engine_basic() {
        let engine = ThemeEngine::new();
        let current = engine.get_current_theme();
        
        assert_eq!(current.name, "Default");
        assert_eq!(current.mode, ThemeMode::Light);
        
        let color = engine.get_color("primary").unwrap();
        assert_eq!(color, "#1a73e8");
        
        engine.switch_to_dark_mode();
        assert!(engine.is_dark_mode());
    }

    #[test]
    fn test_theme_loading() {
        let dir = tempdir().unwrap();
        let theme_path = dir.path().join("test-theme.json");
        
        let theme = Theme {
            name: "TestTheme".to_string(),
            mode: ThemeMode::Dark,
            ..Theme::default()
        };
        
        let theme_json = serde_json::to_string_pretty(&theme).unwrap();
        let mut file = File::create(&theme_path).unwrap();
        file.write_all(theme_json.as_bytes()).unwrap();
        
        let engine = ThemeEngine::new();
        let loaded_theme = engine.load_theme(&theme_path).unwrap();
        
        assert_eq!(loaded_theme.name, "TestTheme");
        assert_eq!(loaded_theme.mode, ThemeMode::Dark);
    }

    #[test]
    fn test_theme_change_listener() {
        let engine = ThemeEngine::new();
        
        // テスト用のテーマを作成
        let light_theme = Theme {
            name: "LightTheme".to_string(),
            mode: ThemeMode::Light,
            ..Theme::default()
        };
        
        let dark_theme = Theme {
            name: "DarkTheme".to_string(),
            mode: ThemeMode::Dark,
            ..Theme::default()
        };
        
        // テーマをインストール
        engine.install_theme(light_theme);
        engine.install_theme(dark_theme);
        
        // コールバックがトリガーされたかを確認するフラグ
        let callback_triggered = Arc::new(AtomicBool::new(false));
        let callback_triggered_clone = callback_triggered.clone();
        
        // リスナーを追加
        engine.add_theme_change_listener(move |event| {
            assert_eq!(event.previous_theme, "Default");
            assert_eq!(event.new_theme, "DarkTheme");
            assert!(event.mode_changed);
            callback_triggered_clone.store(true, Ordering::SeqCst);
        });
        
        // テーマを変更
        engine.set_theme_with_blend("DarkTheme", "test_blend").unwrap();
        
        // コールバックが呼ばれたことを確認
        assert!(callback_triggered.load(Ordering::SeqCst));
    }
    
    #[test]
    fn test_hidpi_support() {
        let mut engine = ThemeEngine::new();
        
        // デフォルトでは1.0を返す
        assert_eq!(engine.get_scale_factor(), 1.0);
        
        // カスタムスケールを設定
        engine.set_scale_factor(1.5);
        assert_eq!(engine.get_scale_factor(), 1.5);
        
        // 高DPI検出器を設定
        engine.set_hidpi_detector(|| 2.0);
        
        // 自動モードに設定し、検出器の値が使われることを確認
        {
            let mut theme = engine.current_theme.write().unwrap();
            theme.display.hidpi_mode = HiDpiMode::Auto;
        }
        
        assert_eq!(engine.get_scale_factor(), 2.0);
    }

    #[test]
    fn test_effect_integration() {
        let engine = ThemeEngine::new();
        
        // エフェクトマネージャーにアクセス
        engine.with_effect_manager(|manager| {
            let settings = theme_effects::EffectSettings::default();
            manager.apply_effect("test_target", theme_effects::EffectType::FadeIn, Some(settings));
        });
        
        // エフェクトが適用されたことを確認
        let effect = engine.with_effect_manager(|manager| {
            manager.get_effect_progress("test_target", theme_effects::EffectType::FadeIn).cloned()
        });
        
        assert!(effect.is_some());
    }
    
    #[test]
    fn test_blend_integration() {
        let engine = ThemeEngine::new();
        
        // テスト用のテーマを作成・登録
        let theme1 = Theme {
            name: "Theme1".to_string(),
            ..Theme::default()
        };
        
        let theme2 = Theme {
            name: "Theme2".to_string(),
            ..Theme::default()
        };
        
        engine.install_theme(theme1);
        engine.install_theme(theme2);
        
        // ブレンド設定を追加
        engine.with_scene_blend_manager(|manager| {
            let mut settings = theme_effects::EffectSettings::default();
            settings.duration_ms = 100; // テスト用に短く
            manager.add_blend_setting("fade", settings);
        });
        
        // テーマ1を設定
        engine.set_theme_with_blend("Theme1", "fade").unwrap();
        
        // ブレンドでテーマ2に変更
        engine.set_theme_with_blend("Theme2", "fade").unwrap();
        
        // ブレンドが開始されたことを確認
        let blend = engine.with_scene_blend_manager(|manager| manager.get_current_blend());
        assert!(blend.is_some());
        
        if let Some((from, to, _)) = blend {
            assert_eq!(from, "Theme1");
            assert_eq!(to, "Theme2");
        }
    }
    
    #[test]
    fn test_dynamic_theme_integration() {
        let mut engine = ThemeEngine::new();
        
        // 動的テーマを初期化
        engine.init_dynamic_theme_manager();
        
        // 動的テーママネージャーが作成されたことを確認
        let manager = engine.get_dynamic_theme_manager();
        assert!(manager.is_some());
        
        // 動的テーマを有効化
        engine.enable_dynamic_theme(None);
        
        // 1秒待機
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // 無効化
        engine.disable_dynamic_theme();
    }
} 