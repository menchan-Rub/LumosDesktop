// LumosDesktop テーマアクセシビリティチェッカー
// テーマがアクセシビリティ要件を満たしているかを検証するツール

use crate::ui::theming::engine::{Theme, ColorPalette, FontSettings};
use crate::ui::theming::tools::theme_validator::{ValidationResult, ValidationSeverity};
use log::{info, warn, error, debug};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// WCAGコンプライアンスレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WcagLevel {
    /// A (最小レベル)
    A,
    /// AA (推奨レベル)
    AA,
    /// AAA (最高レベル)
    AAA,
}

/// アクセシビリティ検証結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityResult {
    /// WCAGガイドラインID
    pub guideline_id: String,
    /// ガイドライン名
    pub guideline_name: String,
    /// 重要度
    pub severity: ValidationSeverity,
    /// 問題の説明
    pub message: String,
    /// 推奨される修正
    pub suggested_fix: Option<String>,
    /// 追加情報URL
    pub info_url: Option<String>,
}

/// カラーパレットの視覚シミュレーション種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorVisionType {
    /// 赤色弱（第一色覚異常）
    Protanopia,
    /// 緑色弱（第二色覚異常）
    Deuteranopia,
    /// 青色弱（第三色覚異常）
    Tritanopia,
    /// 完全色覚異常（色が見えない）
    Achromatopsia,
}

/// アクセシビリティチェッカー
pub struct AccessibilityChecker {
    /// 検証するWCAGレベル
    wcag_level: WcagLevel,
}

impl AccessibilityChecker {
    /// 新しいアクセシビリティチェッカーを作成
    pub fn new() -> Self {
        Self {
            wcag_level: WcagLevel::AA,
        }
    }
    
    /// WCAGレベルを設定
    pub fn set_wcag_level(&mut self, level: WcagLevel) {
        self.wcag_level = level;
    }
    
    /// テーマを検証
    pub fn check_theme(&self, theme: &Theme) -> Vec<AccessibilityResult> {
        let mut results = Vec::new();
        
        // コントラスト比検証
        results.extend(self.check_contrast_ratio(theme));
        
        // フォントサイズ検証
        results.extend(self.check_font_size(theme));
        
        // フォーカス表示検証
        results.extend(self.check_focus_visibility(theme));
        
        // 色覚異常シミュレーション
        results.extend(self.check_color_vision_simulation(theme));
        
        // アニメーション検証
        results.extend(self.check_animations(theme));
        
        // UI要素の認識検証
        results.extend(self.check_ui_recognition(theme));
        
        results
    }
    
    /// コントラスト比を検証
    fn check_contrast_ratio(&self, theme: &Theme) -> Vec<AccessibilityResult> {
        let mut results = Vec::new();
        
        // テキストコントラスト比を計算
        if let (Ok(bg), Ok(fg)) = (
            super::super::engine::color::RGB::from_hex(&theme.colors.background),
            super::super::engine::color::RGB::from_hex(&theme.colors.foreground)
        ) {
            let contrast_ratio = calculate_contrast_ratio(bg, fg);
            
            // WCAGレベルに応じた要件を確認
            match self.wcag_level {
                WcagLevel::A => {
                    // WCAG 2.1 Level A requires a contrast ratio of at least 3:1 for large text
                    // and 4.5:1 for normal text
                    if contrast_ratio < 4.5 {
                        results.push(AccessibilityResult {
                            guideline_id: "1.4.3".to_string(),
                            guideline_name: "コントラスト (最低限)".to_string(),
                            severity: ValidationSeverity::Error,
                            message: format!("テキストのコントラスト比 ({:.2}:1) がWCAG Level Aの要件 (4.5:1) を満たしていません", contrast_ratio),
                            suggested_fix: Some("テキストと背景のコントラストを高めてください".to_string()),
                            info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html".to_string()),
                        });
                    }
                },
                WcagLevel::AA => {
                    // Same as Level A
                    if contrast_ratio < 4.5 {
                        results.push(AccessibilityResult {
                            guideline_id: "1.4.3".to_string(),
                            guideline_name: "コントラスト (最低限)".to_string(),
                            severity: ValidationSeverity::Error,
                            message: format!("テキストのコントラスト比 ({:.2}:1) がWCAG Level AAの要件 (4.5:1) を満たしていません", contrast_ratio),
                            suggested_fix: Some("テキストと背景のコントラストを高めてください".to_string()),
                            info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html".to_string()),
                        });
                    }
                },
                WcagLevel::AAA => {
                    // WCAG 2.1 Level AAA requires a contrast ratio of at least 4.5:1 for large text
                    // and 7:1 for normal text
                    if contrast_ratio < 7.0 {
                        results.push(AccessibilityResult {
                            guideline_id: "1.4.6".to_string(),
                            guideline_name: "コントラスト (拡張)".to_string(),
                            severity: ValidationSeverity::Warning,
                            message: format!("テキストのコントラスト比 ({:.2}:1) がWCAG Level AAAの要件 (7:1) を満たしていません", contrast_ratio),
                            suggested_fix: Some("テキストと背景のコントラストをさらに高めてください".to_string()),
                            info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/contrast-enhanced.html".to_string()),
                        });
                    }
                },
            }
        }
        
        // UI要素のコントラスト比
        if let (Ok(bg), Ok(primary)) = (
            super::super::engine::color::RGB::from_hex(&theme.colors.background),
            super::super::engine::color::RGB::from_hex(&theme.colors.primary)
        ) {
            let contrast_ratio = calculate_contrast_ratio(bg, primary);
            
            // WCAG 2.1 1.4.11 Non-text Contrast requires a contrast ratio of at least 3:1
            if contrast_ratio < 3.0 {
                results.push(AccessibilityResult {
                    guideline_id: "1.4.11".to_string(),
                    guideline_name: "非テキストのコントラスト".to_string(),
                    severity: ValidationSeverity::Warning,
                    message: format!("UI要素のコントラスト比 ({:.2}:1) がWCAGの要件 (3:1) を満たしていません", contrast_ratio),
                    suggested_fix: Some("UI要素と背景のコントラストを高めてください".to_string()),
                    info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast.html".to_string()),
                });
            }
        }
        
        // ステータスカラーのコントラスト
        let status_colors = vec![
            ("エラー色", &theme.colors.error),
            ("警告色", &theme.colors.warning),
            ("成功色", &theme.colors.success),
            ("情報色", &theme.colors.info),
        ];
        
        if let Ok(bg) = super::super::engine::color::RGB::from_hex(&theme.colors.background) {
            for (name, color) in status_colors {
                if let Ok(status_color) = super::super::engine::color::RGB::from_hex(color) {
                    let contrast_ratio = calculate_contrast_ratio(bg, status_color);
                    
                    if contrast_ratio < 3.0 {
                        results.push(AccessibilityResult {
                            guideline_id: "1.4.11".to_string(),
                            guideline_name: "非テキストのコントラスト".to_string(),
                            severity: ValidationSeverity::Warning,
                            message: format!("{}のコントラスト比 ({:.2}:1) が不十分です", name, contrast_ratio),
                            suggested_fix: Some(format!("{}と背景のコントラストを高めてください", name)),
                            info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast.html".to_string()),
                        });
                    }
                }
            }
        }
        
        results
    }
    
    /// フォントサイズを検証
    fn check_font_size(&self, theme: &Theme) -> Vec<AccessibilityResult> {
        let mut results = Vec::new();
        
        // 基本フォントサイズをチェック
        let base_size = theme.fonts.base_size;
        
        // 12px is generally considered the minimum for readable text
        if base_size < 12 {
            results.push(AccessibilityResult {
                guideline_id: "1.4.4".to_string(),
                guideline_name: "テキストのサイズ変更".to_string(),
                severity: ValidationSeverity::Warning,
                message: format!("基本フォントサイズ ({}px) が小さすぎます", base_size),
                suggested_fix: Some("可読性を高めるため、基本フォントサイズを12px以上にしてください".to_string()),
                info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/resize-text.html".to_string()),
            });
        }
        
        results
    }
    
    /// フォーカス表示を検証
    fn check_focus_visibility(&self, theme: &Theme) -> Vec<AccessibilityResult> {
        let mut results = Vec::new();
        
        // フォーカスリングの幅をチェック
        let focus_ring_width = theme.widget_style.focus_ring_width;
        
        if focus_ring_width < 2 {
            results.push(AccessibilityResult {
                guideline_id: "2.4.7".to_string(),
                guideline_name: "フォーカスの可視性".to_string(),
                severity: ValidationSeverity::Warning,
                message: format!("フォーカスリングの幅 ({}px) が細すぎます", focus_ring_width),
                suggested_fix: Some("キーボードフォーカスを明確に表示するため、フォーカスリングの幅を2px以上にしてください".to_string()),
                info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/focus-visible.html".to_string()),
            });
        }
        
        results
    }
    
    /// 色覚異常シミュレーションを実行
    fn check_color_vision_simulation(&self, theme: &Theme) -> Vec<AccessibilityResult> {
        let mut results = Vec::new();
        
        // カラーパレットから主要な色を取得
        let primary = super::super::engine::color::RGB::from_hex(&theme.colors.primary).unwrap_or_default();
        let secondary = super::super::engine::color::RGB::from_hex(&theme.colors.secondary).unwrap_or_default();
        let accent = super::super::engine::color::RGB::from_hex(&theme.colors.accent).unwrap_or_default();
        let error = super::super::engine::color::RGB::from_hex(&theme.colors.error).unwrap_or_default();
        let warning = super::super::engine::color::RGB::from_hex(&theme.colors.warning).unwrap_or_default();
        let success = super::super::engine::color::RGB::from_hex(&theme.colors.success).unwrap_or_default();
        
        // 第一色覚異常（赤色弱）シミュレーション
        let protanopia_primary = simulate_color_vision(primary, ColorVisionType::Protanopia);
        let protanopia_secondary = simulate_color_vision(secondary, ColorVisionType::Protanopia);
        
        // 第一色覚異常でプライマリとセカンダリが区別しにくい場合
        if calculate_color_difference(protanopia_primary, protanopia_secondary) < 25.0 {
            results.push(AccessibilityResult {
                guideline_id: "1.4.1".to_string(),
                guideline_name: "色の使用".to_string(),
                severity: ValidationSeverity::Warning,
                message: "赤色弱の方にはプライマリカラーとセカンダリカラーの区別が難しい可能性があります".to_string(),
                suggested_fix: Some("輝度（明るさ）の差を大きくするか、形状や記号などの追加的な視覚的手がかりを使用してください".to_string()),
                info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/use-of-color.html".to_string()),
            });
        }
        
        // 第二色覚異常（緑色弱）シミュレーション
        let deuteranopia_success = simulate_color_vision(success, ColorVisionType::Deuteranopia);
        let deuteranopia_error = simulate_color_vision(error, ColorVisionType::Deuteranopia);
        
        // 第二色覚異常で成功と警告、エラーが区別しにくい場合
        if calculate_color_difference(deuteranopia_success, deuteranopia_error) < 25.0 {
            results.push(AccessibilityResult {
                guideline_id: "1.4.1".to_string(),
                guideline_name: "色の使用".to_string(),
                severity: ValidationSeverity::Warning,
                message: "緑色弱の方には成功とエラーの状態の区別が難しい可能性があります".to_string(),
                suggested_fix: Some("成功とエラーの状態を色だけでなく、形状やテキストでも区別できるようにしてください".to_string()),
                info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/use-of-color.html".to_string()),
            });
        }
        
        results
    }
    
    /// アニメーションを検証
    fn check_animations(&self, theme: &Theme) -> Vec<AccessibilityResult> {
        let mut results = Vec::new();
        
        // アニメーション時間をチェック
        if theme.animations.enabled {
            let transition_ms = theme.animations.transition_ms;
            
            if transition_ms > 500 {
                results.push(AccessibilityResult {
                    guideline_id: "2.3.3".to_string(),
                    guideline_name: "アニメーションによる操作".to_string(),
                    severity: ValidationSeverity::Info,
                    message: format!("アニメーション時間 ({}ms) が長すぎる可能性があります", transition_ms),
                    suggested_fix: Some("認知負荷を減らし、前庭障害のある方への配慮として、アニメーション時間を短く（500ms以下）してください".to_string()),
                    info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions.html".to_string()),
                });
            }
        }
        
        results
    }
    
    /// UI要素の認識性を検証
    fn check_ui_recognition(&self, theme: &Theme) -> Vec<AccessibilityResult> {
        let mut results = Vec::new();
        
        // ボタンの視認性をチェック
        let border_width = theme.widget_style.border_width;
        
        if border_width == 0 {
            // ボーダーがない場合、他の視覚的な区別があるか確認
            if let (Ok(bg), Ok(primary)) = (
                super::super::engine::color::RGB::from_hex(&theme.colors.background),
                super::super::engine::color::RGB::from_hex(&theme.colors.primary)
            ) {
                let contrast_ratio = calculate_contrast_ratio(bg, primary);
                
                if contrast_ratio < 3.0 {
                    results.push(AccessibilityResult {
                        guideline_id: "1.4.11".to_string(),
                        guideline_name: "非テキストのコントラスト".to_string(),
                        severity: ValidationSeverity::Warning,
                        message: "ボーダーがなく、背景とのコントラストも低いため、インタラクティブ要素の識別が難しい可能性があります".to_string(),
                        suggested_fix: Some("ボーダーを追加するか、背景とのコントラストを高めてください".to_string()),
                        info_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast.html".to_string()),
                    });
                }
            }
        }
        
        results
    }
    
    /// アクセシビリティ検証結果のサマリーを取得
    pub fn get_summary(&self, results: &[AccessibilityResult]) -> AccessibilitySummary {
        let mut summary = AccessibilitySummary {
            total_issues: results.len(),
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            wcag_compliance: if results.is_empty() {
                self.wcag_level
            } else {
                // エラーがある場合はA未満、警告のみの場合はAとする
                let has_errors = results.iter().any(|r| r.severity == ValidationSeverity::Error);
                if has_errors {
                    match self.wcag_level {
                        WcagLevel::AAA => WcagLevel::AA,
                        WcagLevel::AA => WcagLevel::A,
                        WcagLevel::A => WcagLevel::A, // A未満という概念はないのでAとする
                    }
                } else {
                    WcagLevel::A
                }
            },
            guideline_issues: HashMap::new(),
        };
        
        // 各問題をカウント
        for result in results {
            match result.severity {
                ValidationSeverity::Error => summary.error_count += 1,
                ValidationSeverity::Warning => summary.warning_count += 1,
                ValidationSeverity::Info => summary.info_count += 1,
            }
            
            // ガイドラインごとの問題数をカウント
            let count = summary.guideline_issues
                .entry(result.guideline_id.clone())
                .or_insert(0);
            *count += 1;
        }
        
        summary
    }
}

/// アクセシビリティ検証結果のサマリー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySummary {
    /// 問題の総数
    pub total_issues: usize,
    /// エラーの数
    pub error_count: usize,
    /// 警告の数
    pub warning_count: usize,
    /// 情報の数
    pub info_count: usize,
    /// WCAG準拠レベル
    pub wcag_compliance: WcagLevel,
    /// ガイドラインごとの問題数
    pub guideline_issues: HashMap<String, usize>,
}

impl AccessibilitySummary {
    /// テーマが指定されたWCAGレベルに準拠しているかどうか
    pub fn complies_with(&self, level: WcagLevel) -> bool {
        match (self.wcag_compliance, level) {
            (WcagLevel::AAA, _) => true,
            (WcagLevel::AA, WcagLevel::A) | (WcagLevel::AA, WcagLevel::AA) => true,
            (WcagLevel::A, WcagLevel::A) => true,
            _ => false,
        }
    }
}

/// コントラスト比を計算
fn calculate_contrast_ratio(color1: super::super::engine::color::RGB, color2: super::super::engine::color::RGB) -> f32 {
    // 相対輝度を計算
    let l1 = calculate_relative_luminance(color1);
    let l2 = calculate_relative_luminance(color2);
    
    // 明るい色と暗い色を決定
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    
    // コントラスト比を計算: (L1 + 0.05) / (L2 + 0.05)
    (lighter + 0.05) / (darker + 0.05)
}

/// 相対輝度を計算
fn calculate_relative_luminance(color: super::super::engine::color::RGB) -> f32 {
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

/// 色覚異常のシミュレーション
fn simulate_color_vision(color: super::super::engine::color::RGB, vision_type: ColorVisionType) -> super::super::engine::color::RGB {
    // LMS色空間に変換
    let (l, m, s) = rgb_to_lms(color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0);
    
    // 色覚異常のシミュレーション
    let (l_sim, m_sim, s_sim) = match vision_type {
        ColorVisionType::Protanopia => {
            // 赤色弱のシミュレーション
            let l_sim = 0.0 * l + 2.02344 * m + -2.52581 * s;
            let m_sim = 0.0 * l + 1.0 * m + 0.0 * s;
            let s_sim = 0.0 * l + 0.0 * m + 1.0 * s;
            (l_sim, m_sim, s_sim)
        },
        ColorVisionType::Deuteranopia => {
            // 緑色弱のシミュレーション
            let l_sim = 1.0 * l + 0.0 * m + 0.0 * s;
            let m_sim = 0.494207 * l + 0.0 * m + 1.24827 * s;
            let s_sim = 0.0 * l + 0.0 * m + 1.0 * s;
            (l_sim, m_sim, s_sim)
        },
        ColorVisionType::Tritanopia => {
            // 青色弱のシミュレーション
            let l_sim = 1.0 * l + 0.0 * m + 0.0 * s;
            let m_sim = 0.0 * l + 1.0 * m + 0.0 * s;
            let s_sim = -0.395913 * l + 0.801109 * m + 0.0 * s;
            (l_sim, m_sim, s_sim)
        },
        ColorVisionType::Achromatopsia => {
            // 完全色覚異常のシミュレーション（白黒）
            let gray = 0.2126 * l + 0.7152 * m + 0.0722 * s;
            (gray, gray, gray)
        },
    };
    
    // RGBに戻す
    let (r, g, b) = lms_to_rgb(l_sim, m_sim, s_sim);
    
    // 値の範囲を0-255に制限
    let r = (r * 255.0).round().max(0.0).min(255.0) as u8;
    let g = (g * 255.0).round().max(0.0).min(255.0) as u8;
    let b = (b * 255.0).round().max(0.0).min(255.0) as u8;
    
    super::super::engine::color::RGB::new(r, g, b)
}

/// RGBからLMS色空間への変換
fn rgb_to_lms(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let l = 0.3904725 * r + 0.5497849 * g + 0.0089818 * b;
    let m = 0.0709473 * r + 0.9637693 * g + 0.0011722 * b;
    let s = 0.0231290 * r + 0.1282910 * g + 0.9362570 * b;
    (l, m, s)
}

/// LMS色空間からRGBへの変換
fn lms_to_rgb(l: f32, m: f32, s: f32) -> (f32, f32, f32) {
    let r =  4.0767416 * l - 3.3077115 * m + 0.2307995 * s;
    let g = -1.2684063 * l + 2.6097574 * m - 0.3413513 * s;
    let b = -0.0041960 * l - 0.7034186 * m + 1.7076147 * s;
    (r, g, b)
}

/// 色の差を計算（CIEDE2000色差）
fn calculate_color_difference(color1: super::super::engine::color::RGB, color2: super::super::engine::color::RGB) -> f32 {
    // 簡略化のため、ユークリッド距離で色差を近似
    let r_diff = (color1.r as f32 - color2.r as f32).powi(2);
    let g_diff = (color1.g as f32 - color2.g as f32).powi(2);
    let b_diff = (color1.b as f32 - color2.b as f32).powi(2);
    
    (r_diff + g_diff + b_diff).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contrast_calculation() {
        // 白と黒のコントラスト比は最大の21になるはず
        let white = super::super::super::engine::color::RGB::new(255, 255, 255);
        let black = super::super::super::engine::color::RGB::new(0, 0, 0);
        
        let contrast = calculate_contrast_ratio(white, black);
        assert!((contrast - 21.0).abs() < 0.1);
    }
    
    #[test]
    fn test_color_simulation() {
        // 赤色のプロタノピアシミュレーション
        let red = super::super::super::engine::color::RGB::new(255, 0, 0);
        let simulated = simulate_color_vision(red, ColorVisionType::Protanopia);
        
        // 赤色弱では赤が暗い灰色に見える
        assert!(simulated.r < 100);
        assert!(simulated.g < 100);
        assert!(simulated.b < 100);
    }
    
    #[test]
    fn test_accessibility_checker() {
        let checker = AccessibilityChecker::new();
        
        // テスト用の問題のあるテーマを作成
        let mut theme = super::super::super::engine::Theme::default();
        
        // コントラスト問題を作る
        theme.colors.background = "#ffffff".to_string(); // 白背景
        theme.colors.foreground = "#bbbbbb".to_string(); // 薄いグレー (コントラスト不足)
        
        // フォーカスリング問題
        theme.widget_style.focus_ring_width = 1; // 細すぎる
        
        // 検証
        let results = checker.check_theme(&theme);
        
        // 少なくとも2つの問題があるはず
        assert!(results.len() >= 2);
        
        // サマリーを取得
        let summary = checker.get_summary(&results);
        assert!(summary.total_issues >= 2);
        assert!(!summary.complies_with(WcagLevel::AA));
    }
} 