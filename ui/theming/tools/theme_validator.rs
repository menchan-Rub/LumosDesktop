// LumosDesktop テーマ検証ツール
// テーマがガイドラインに準拠しているかを検証するツール

use crate::ui::theming::engine::{Theme, ColorPalette, FontSettings, WidgetStyle};
use log::{info, warn, error, debug};
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// 検証結果の重要度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// 情報（軽微な問題や提案）
    Info,
    /// 警告（推奨されない設定）
    Warning,
    /// エラー（動作に影響する問題）
    Error,
}

/// 検証結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 重要度
    pub severity: ValidationSeverity,
    /// メッセージ
    pub message: String,
    /// 該当するプロパティパス
    pub property_path: String,
    /// 推奨される修正
    pub suggested_fix: Option<String>,
}

/// 検証ルール
pub trait ValidationRule {
    /// ルールの名前を取得
    fn name(&self) -> &str;
    
    /// 説明を取得
    fn description(&self) -> &str;
    
    /// テーマを検証
    fn validate(&self, theme: &Theme) -> Vec<ValidationResult>;
}

/// カラーコントラスト検証ルール
pub struct ColorContrastRule;

impl ValidationRule for ColorContrastRule {
    fn name(&self) -> &str {
        "color_contrast"
    }
    
    fn description(&self) -> &str {
        "前景色と背景色のコントラスト比が十分か検証します"
    }
    
    fn validate(&self, theme: &Theme) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // 背景色と前景色のコントラスト比を計算
        if let (Ok(bg), Ok(fg)) = (
            super::engine::color::RGB::from_hex(&theme.colors.background),
            super::engine::color::RGB::from_hex(&theme.colors.foreground)
        ) {
            let contrast_ratio = calculate_contrast_ratio(bg, fg);
            
            if contrast_ratio < 4.5 {
                results.push(ValidationResult {
                    severity: ValidationSeverity::Error,
                    message: format!("背景色と前景色のコントラスト比が不足しています: {:.2}", contrast_ratio),
                    property_path: "colors.foreground".to_string(),
                    suggested_fix: Some("コントラスト比が4.5以上になるように色を調整してください".to_string()),
                });
            } else if contrast_ratio < 7.0 {
                results.push(ValidationResult {
                    severity: ValidationSeverity::Warning,
                    message: format!("背景色と前景色のコントラスト比が推奨値未満です: {:.2}", contrast_ratio),
                    property_path: "colors.foreground".to_string(),
                    suggested_fix: Some("より良いアクセシビリティのために、コントラスト比7.0以上を推奨します".to_string()),
                });
            }
        }
        
        // プライマリカラーと背景色のコントラスト
        if let (Ok(bg), Ok(primary)) = (
            super::engine::color::RGB::from_hex(&theme.colors.background),
            super::engine::color::RGB::from_hex(&theme.colors.primary)
        ) {
            let contrast_ratio = calculate_contrast_ratio(bg, primary);
            
            if contrast_ratio < 3.0 {
                results.push(ValidationResult {
                    severity: ValidationSeverity::Warning,
                    message: format!("プライマリカラーと背景色のコントラスト比が低すぎます: {:.2}", contrast_ratio),
                    property_path: "colors.primary".to_string(),
                    suggested_fix: Some("視認性向上のため、コントラスト比を高くしてください".to_string()),
                });
            }
        }
        
        // エラー色のコントラスト
        if let (Ok(bg), Ok(error)) = (
            super::engine::color::RGB::from_hex(&theme.colors.background),
            super::engine::color::RGB::from_hex(&theme.colors.error)
        ) {
            let contrast_ratio = calculate_contrast_ratio(bg, error);
            
            if contrast_ratio < 4.5 {
                results.push(ValidationResult {
                    severity: ValidationSeverity::Warning,
                    message: format!("エラー色と背景色のコントラスト比が不足しています: {:.2}", contrast_ratio),
                    property_path: "colors.error".to_string(),
                    suggested_fix: Some("エラーメッセージの視認性向上のため、コントラスト比を高くしてください".to_string()),
                });
            }
        }
        
        results
    }
}

/// フォント設定検証ルール
pub struct FontSettingsRule;

impl ValidationRule for FontSettingsRule {
    fn name(&self) -> &str {
        "font_settings"
    }
    
    fn description(&self) -> &str {
        "フォント設定が可読性に適しているか検証します"
    }
    
    fn validate(&self, theme: &Theme) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // 基本フォントサイズのチェック
        let base_size = theme.fonts.base_size;
        
        if base_size < 12 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Warning,
                message: format!("基本フォントサイズが小さすぎます: {}px", base_size),
                property_path: "fonts.base_size".to_string(),
                suggested_fix: Some("可読性を高めるため、12px以上を推奨します".to_string()),
            });
        } else if base_size > 24 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Info,
                message: format!("基本フォントサイズが大きいです: {}px", base_size),
                property_path: "fonts.base_size".to_string(),
                suggested_fix: Some("通常のインターフェースでは14-18pxが適切です".to_string()),
            });
        }
        
        // 行の高さのチェック
        let line_height = theme.fonts.line_height;
        
        if line_height < 1.2 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Warning,
                message: format!("行の高さが小さすぎます: {:.1}", line_height),
                property_path: "fonts.line_height".to_string(),
                suggested_fix: Some("可読性を高めるため、1.2以上を推奨します".to_string()),
            });
        } else if line_height > 2.0 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Info,
                message: format!("行の高さが大きいです: {:.1}", line_height),
                property_path: "fonts.line_height".to_string(),
                suggested_fix: Some("通常のテキストでは1.4-1.6が適切です".to_string()),
            });
        }
        
        // フォントレンダリング設定
        if !theme.fonts.rendering.antialias {
            results.push(ValidationResult {
                severity: ValidationSeverity::Warning,
                message: "アンチエイリアスが無効になっています".to_string(),
                property_path: "fonts.rendering.antialias".to_string(),
                suggested_fix: Some("可読性向上のためアンチエイリアスを有効にすることを推奨します".to_string()),
            });
        }
        
        results
    }
}

/// ウィジェットスタイル検証ルール
pub struct WidgetStyleRule;

impl ValidationRule for WidgetStyleRule {
    fn name(&self) -> &str {
        "widget_style"
    }
    
    fn description(&self) -> &str {
        "ウィジェットスタイルの一貫性と使いやすさを検証します"
    }
    
    fn validate(&self, theme: &Theme) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // ボタンの丸みチェック
        let button_radius = theme.widget_style.button_radius;
        
        if button_radius > 20 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Warning,
                message: format!("ボタンの丸みが大きすぎます: {}px", button_radius),
                property_path: "widget_style.button_radius".to_string(),
                suggested_fix: Some("通常のボタンでは2-8pxが適切です".to_string()),
            });
        }
        
        // コントロールパディングチェック
        let control_padding = theme.widget_style.control_padding;
        
        if control_padding < 4 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Warning,
                message: format!("コントロールのパディングが小さすぎます: {}px", control_padding),
                property_path: "widget_style.control_padding".to_string(),
                suggested_fix: Some("タッチ操作の使いやすさのため、8px以上を推奨します".to_string()),
            });
        } else if control_padding > 16 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Info,
                message: format!("コントロールのパディングが大きいです: {}px", control_padding),
                property_path: "widget_style.control_padding".to_string(),
                suggested_fix: Some("密度の高いUIのためには8-12pxが適切です".to_string()),
            });
        }
        
        // 一貫性チェック
        if theme.widget_style.button_radius != theme.widget_style.input_radius {
            results.push(ValidationResult {
                severity: ValidationSeverity::Info,
                message: "ボタンとインプットの丸みが一致していません".to_string(),
                property_path: "widget_style".to_string(),
                suggested_fix: Some("視覚的一貫性のため、同じ値を使用することを推奨します".to_string()),
            });
        }
        
        results
    }
}

/// アニメーション設定検証ルール
pub struct AnimationSettingsRule;

impl ValidationRule for AnimationSettingsRule {
    fn name(&self) -> &str {
        "animation_settings"
    }
    
    fn description(&self) -> &str {
        "アニメーション設定がユーザビリティに適しているか検証します"
    }
    
    fn validate(&self, theme: &Theme) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // トランジション時間チェック
        let transition_ms = theme.animations.transition_ms;
        
        if transition_ms < 100 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Info,
                message: format!("トランジション時間が短すぎます: {}ms", transition_ms),
                property_path: "animations.transition_ms".to_string(),
                suggested_fix: Some("視認可能なアニメーションには100ms以上を推奨します".to_string()),
            });
        } else if transition_ms > 500 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Warning,
                message: format!("トランジション時間が長すぎます: {}ms", transition_ms),
                property_path: "animations.transition_ms".to_string(),
                suggested_fix: Some("応答性を高めるため、UIアニメーションは300ms以下が適切です".to_string()),
            });
        }
        
        // 速度係数チェック
        let speed_factor = theme.animations.speed_factor;
        
        if speed_factor > 2.0 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Warning,
                message: format!("アニメーション速度係数が高すぎます: {:.1}", speed_factor),
                property_path: "animations.speed_factor".to_string(),
                suggested_fix: Some("わかりやすいアニメーションのため、0.5-1.5の範囲内が適切です".to_string()),
            });
        } else if speed_factor < 0.5 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Warning,
                message: format!("アニメーション速度係数が低すぎます: {:.1}", speed_factor),
                property_path: "animations.speed_factor".to_string(),
                suggested_fix: Some("応答性を高めるため、0.5以上を推奨します".to_string()),
            });
        }
        
        results
    }
}

/// カラーハーモニー検証ルール
pub struct ColorHarmonyRule;

impl ValidationRule for ColorHarmonyRule {
    fn name(&self) -> &str {
        "color_harmony"
    }
    
    fn description(&self) -> &str {
        "カラーパレットの色調和を検証します"
    }
    
    fn validate(&self, theme: &Theme) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // プライマリとアクセント色の関係チェック
        if let (Ok(primary), Ok(accent)) = (
            super::engine::color::RGB::from_hex(&theme.colors.primary),
            super::engine::color::RGB::from_hex(&theme.colors.accent)
        ) {
            let primary_hsl = primary.to_hsl();
            let accent_hsl = accent.to_hsl();
            
            // 色相差のチェック
            let hue_diff = (primary_hsl.h - accent_hsl.h).abs();
            let normalized_hue_diff = if hue_diff > 180.0 { 360.0 - hue_diff } else { hue_diff };
            
            // 補色関係（約180度）でも、類似色（30度以内）でもない場合
            if normalized_hue_diff > 30.0 && (normalized_hue_diff < 150.0 || normalized_hue_diff > 210.0) {
                results.push(ValidationResult {
                    severity: ValidationSeverity::Info,
                    message: format!("プライマリとアクセント色の色相差が調和的でない可能性があります: {:.1}度", normalized_hue_diff),
                    property_path: "colors".to_string(),
                    suggested_fix: Some("補色関係（約180度）または類似色（30度以内）の使用を検討してください".to_string()),
                });
            }
            
            // 彩度のバランスチェック
            if accent_hsl.s < primary_hsl.s {
                results.push(ValidationResult {
                    severity: ValidationSeverity::Info,
                    message: "アクセント色の彩度がプライマリ色より低くなっています".to_string(),
                    property_path: "colors.accent".to_string(),
                    suggested_fix: Some("アクセント色はプライマリ色より彩度を高くすることで目立たせることができます".to_string()),
                });
            }
        }
        
        results
    }
}

/// テーマ名検証ルール
pub struct ThemeNameRule;

impl ValidationRule for ThemeNameRule {
    fn name(&self) -> &str {
        "theme_name"
    }
    
    fn description(&self) -> &str {
        "テーマ名とメタデータの完全性を検証します"
    }
    
    fn validate(&self, theme: &Theme) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // 名前の長さチェック
        if theme.name.len() < 3 {
            results.push(ValidationResult {
                severity: ValidationSeverity::Warning,
                message: format!("テーマ名が短すぎます: {}", theme.name),
                property_path: "name".to_string(),
                suggested_fix: Some("わかりやすいテーマ名のために3文字以上を推奨します".to_string()),
            });
        }
        
        // 作者情報のチェック
        if theme.author.is_none() {
            results.push(ValidationResult {
                severity: ValidationSeverity::Info,
                message: "作者情報が設定されていません".to_string(),
                property_path: "author".to_string(),
                suggested_fix: Some("テーマの出所を明確にするために作者情報を設定してください".to_string()),
            });
        }
        
        // バージョン情報のチェック
        if theme.version.is_none() {
            results.push(ValidationResult {
                severity: ValidationSeverity::Info,
                message: "バージョン情報が設定されていません".to_string(),
                property_path: "version".to_string(),
                suggested_fix: Some("テーマの管理を容易にするためにバージョン情報を設定してください".to_string()),
            });
        }
        
        // 説明のチェック
        if let Some(ref desc) = theme.description {
            if desc.len() < 10 {
                results.push(ValidationResult {
                    severity: ValidationSeverity::Info,
                    message: "テーマの説明が短すぎます".to_string(),
                    property_path: "description".to_string(),
                    suggested_fix: Some("テーマの特徴がわかるように詳細な説明を追加してください".to_string()),
                });
            }
        } else {
            results.push(ValidationResult {
                severity: ValidationSeverity::Info,
                message: "テーマの説明が設定されていません".to_string(),
                property_path: "description".to_string(),
                suggested_fix: Some("テーマの特徴を説明するために説明文を追加してください".to_string()),
            });
        }
        
        results
    }
}

/// テーマ検証ツール
pub struct ThemeValidator {
    /// 検証ルール
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ThemeValidator {
    /// 新しいテーマ検証ツールを作成
    pub fn new() -> Self {
        let mut validator = Self {
            rules: Vec::new(),
        };
        
        // デフォルトのルールを追加
        validator.add_rule(Box::new(ColorContrastRule));
        validator.add_rule(Box::new(FontSettingsRule));
        validator.add_rule(Box::new(WidgetStyleRule));
        validator.add_rule(Box::new(AnimationSettingsRule));
        validator.add_rule(Box::new(ColorHarmonyRule));
        validator.add_rule(Box::new(ThemeNameRule));
        
        validator
    }
    
    /// 検証ルールを追加
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }
    
    /// テーマを検証
    pub fn validate(&self, theme: &Theme) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        for rule in &self.rules {
            let rule_results = rule.validate(theme);
            if !rule_results.is_empty() {
                info!("ルール '{}' で {}件の問題が見つかりました", rule.name(), rule_results.len());
                results.extend(rule_results);
            }
        }
        
        // 結果を重要度でソート
        results.sort_by(|a, b| {
            use std::cmp::Ordering;
            
            match (a.severity, b.severity) {
                (ValidationSeverity::Error, ValidationSeverity::Error) => Ordering::Equal,
                (ValidationSeverity::Error, _) => Ordering::Less,
                (ValidationSeverity::Warning, ValidationSeverity::Error) => Ordering::Greater,
                (ValidationSeverity::Warning, ValidationSeverity::Warning) => Ordering::Equal,
                (ValidationSeverity::Warning, _) => Ordering::Less,
                (ValidationSeverity::Info, ValidationSeverity::Info) => Ordering::Equal,
                (ValidationSeverity::Info, _) => Ordering::Greater,
            }
        });
        
        results
    }
    
    /// テーマファイルを検証
    pub fn validate_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<ValidationResult>, String> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| format!("テーマファイルを開けませんでした: {}", e))?;
            
        let theme: Theme = serde_json::from_reader(file)
            .map_err(|e| format!("テーマファイルの解析に失敗しました: {}", e))?;
            
        Ok(self.validate(&theme))
    }
    
    /// 検証結果のサマリーを取得
    pub fn get_summary(&self, results: &[ValidationResult]) -> ValidationSummary {
        let mut summary = ValidationSummary {
            total_count: results.len(),
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            property_issues: HashMap::new(),
        };
        
        for result in results {
            match result.severity {
                ValidationSeverity::Error => summary.error_count += 1,
                ValidationSeverity::Warning => summary.warning_count += 1,
                ValidationSeverity::Info => summary.info_count += 1,
            }
            
            let count = summary.property_issues
                .entry(result.property_path.clone())
                .or_insert(0);
            *count += 1;
        }
        
        summary
    }
}

/// 検証結果のサマリー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// 問題の総数
    pub total_count: usize,
    /// エラーの数
    pub error_count: usize,
    /// 警告の数
    pub warning_count: usize,
    /// 情報の数
    pub info_count: usize,
    /// プロパティごとの問題数
    pub property_issues: HashMap<String, usize>,
}

impl ValidationSummary {
    /// 検証に合格したかどうか
    pub fn is_passed(&self) -> bool {
        self.error_count == 0
    }
    
    /// エラーと警告がないかどうか
    pub fn is_clean(&self) -> bool {
        self.error_count == 0 && self.warning_count == 0
    }
}

/// コントラスト比を計算
fn calculate_contrast_ratio(color1: super::engine::color::RGB, color2: super::engine::color::RGB) -> f32 {
    // 相対輝度を計算
    let l1 = calculate_relative_luminance(color1);
    let l2 = calculate_relative_luminance(color2);
    
    // 明るい色と暗い色を決定
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    
    // コントラスト比を計算: (L1 + 0.05) / (L2 + 0.05)
    (lighter + 0.05) / (darker + 0.05)
}

/// 相対輝度を計算
fn calculate_relative_luminance(color: super::engine::color::RGB) -> f32 {
    // sRGB色空間から線形RGB値に変換
    let r = convert_srgb_to_linear(color.r as f32 / 255.0);
    let g = convert_srgb_to_linear(color.g as f32 / 255.0);
    let b = convert_srgb_to_linear(color.b as f32 / 255.0);
    
    // 相対輝度の計算: L = 0.2126 * R + 0.7152 * G + 0.0722 * B
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// sRGB値を線形RGBに変換
fn convert_srgb_to_linear(value: f32) -> f32 {
    if value <= 0.03928 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contrast_calculation() {
        // 白と黒のコントラスト比は最大の21になるはず
        let white = super::super::engine::color::RGB::new(255, 255, 255);
        let black = super::super::engine::color::RGB::new(0, 0, 0);
        
        let contrast = calculate_contrast_ratio(white, black);
        assert!((contrast - 21.0).abs() < 0.1);
        
        // 白と中間グレーのコントラスト比を検証
        let mid_gray = super::super::engine::color::RGB::new(128, 128, 128);
        let contrast = calculate_contrast_ratio(white, mid_gray);
        assert!(contrast > 3.0 && contrast < 5.0);
    }
    
    #[test]
    fn test_theme_validator() {
        let validator = ThemeValidator::new();
        
        // テスト用の問題のあるテーマを作成
        let mut theme = Theme::default();
        
        // コントラスト問題を作る
        theme.colors.background = "#ffffff".to_string(); // 白背景
        theme.colors.foreground = "#bbbbbb".to_string(); // 薄いグレー (コントラスト不足)
        
        // フォントサイズ問題
        theme.fonts.base_size = 9; // 小さすぎる
        
        // 検証
        let results = validator.validate(&theme);
        
        // 少なくとも2つの問題があるはず
        assert!(results.len() >= 2);
        
        // コントラスト問題があるはず
        let contrast_issue = results.iter().find(|r| r.property_path == "colors.foreground");
        assert!(contrast_issue.is_some());
        
        // フォントサイズ問題があるはず
        let font_issue = results.iter().find(|r| r.property_path == "fonts.base_size");
        assert!(font_issue.is_some());
        
        // サマリーを取得
        let summary = validator.get_summary(&results);
        assert!(summary.total_count >= 2);
        assert!(!summary.is_clean());
    }
} 