// LumosDesktop テーマプレビューアプリ
// テーマの視覚的な確認とテスト用アプリケーション

use crate::ui::theming::engine::{Theme, ThemeEngine, ThemeMode};
use crate::ui::toolkit::core::{Application, Window, Widget};
use crate::ui::toolkit::controls::{
    Button, Label, Slider, TextField, Switch, Card, 
    Dropdown, ProgressBar, Checkbox, RadioButton, TabView, Tab
};
use std::sync::Arc;
use log::info;

/// テーマプレビューアプリケーション
pub struct ThemePreviewApp {
    /// アプリケーションインスタンス
    app: Application,
    /// メインウィンドウ
    window: Window,
    /// テーマエンジンの参照
    theme_engine: Arc<ThemeEngine>,
    /// 現在選択されているテーマ名
    current_theme: String,
    /// テーマモード
    theme_mode: ThemeMode,
}

impl ThemePreviewApp {
    /// 新しいテーマプレビューアプリを作成
    pub fn new(theme_engine: Arc<ThemeEngine>) -> Self {
        let app = Application::new("theme_preview", "LumosDesktop テーマプレビュー");
        let window = Window::new(&app, "テーマプレビュー", 1000, 700);
        
        let current_theme = theme_engine.get_current_theme().name;
        let theme_mode = theme_engine.get_current_theme().mode;
        
        Self {
            app,
            window,
            theme_engine,
            current_theme,
            theme_mode,
        }
    }
    
    /// アプリケーションを実行
    pub fn run(&mut self) {
        // UIをセットアップ
        self.setup_ui();
        
        // アプリケーションを実行
        self.app.run();
    }
    
    /// UIをセットアップ
    fn setup_ui(&mut self) {
        // メインコンテナを作成
        let main_container = self.window.get_main_container();
        
        // 上部バーを追加
        let top_bar = self.create_top_bar();
        main_container.add(&top_bar);
        
        // タブビューを作成
        let tab_view = TabView::new();
        
        // 基本コンポーネントタブ
        let basic_tab = self.create_basic_components_tab();
        tab_view.add_tab(Tab::new("基本コンポーネント", basic_tab));
        
        // カラーパレットタブ
        let palette_tab = self.create_color_palette_tab();
        tab_view.add_tab(Tab::new("カラーパレット", palette_tab));
        
        // タイポグラフィタブ
        let typography_tab = self.create_typography_tab();
        tab_view.add_tab(Tab::new("タイポグラフィ", typography_tab));
        
        // アニメーションタブ
        let animation_tab = self.create_animation_tab();
        tab_view.add_tab(Tab::new("アニメーション", animation_tab));
        
        // 高DPIタブ
        let hidpi_tab = self.create_hidpi_tab();
        tab_view.add_tab(Tab::new("高DPI", hidpi_tab));
        
        // タブビューをメインコンテナに追加
        main_container.add(&tab_view);
        
        // ステータスバーを追加
        let status_bar = self.create_status_bar();
        main_container.add(&status_bar);
    }
    
    /// 上部バーを作成
    fn create_top_bar(&self) -> Box<dyn Widget> {
        // 上部バーコンテナ
        let container = crate::ui::toolkit::core::Container::new_horizontal();
        
        // テーマセレクタドロップダウン
        let theme_selector = Dropdown::new();
        let theme_names = self.theme_engine.get_available_themes();
        
        for name in theme_names {
            theme_selector.add_item(&name);
        }
        
        theme_selector.set_selected_item(&self.current_theme);
        theme_selector.on_selection_changed(|item_name| {
            info!("テーマを変更します: {}", item_name);
            // テーマエンジンへの参照を取得するため、クロージャ内でself.theme_engineを使用できるようにクローンなどが必要
            // 実際の実装では、Arc<ThemeEngine>を使用してクローン可能にするなどの対応が必要
        });
        
        container.add(&theme_selector);
        
        // テーマモード切替スイッチ
        let mode_label = Label::new("ダークモード");
        container.add(&mode_label);
        
        let mode_switch = Switch::new();
        mode_switch.set_checked(self.theme_mode == ThemeMode::Dark);
        mode_switch.on_toggle(|checked| {
            info!("テーマモードを変更します: {}", if checked { "ダーク" } else { "ライト" });
            // テーマモードを切り替える処理
        });
        
        container.add(&mode_switch);
        
        // エフェクト選択
        let effect_label = Label::new("切替エフェクト:");
        container.add(&effect_label);
        
        let effect_selector = Dropdown::new();
        effect_selector.add_item("フェード");
        effect_selector.add_item("スライド");
        effect_selector.add_item("ズーム");
        effect_selector.add_item("なし");
        
        effect_selector.set_selected_item("フェード");
        effect_selector.on_selection_changed(|effect_name| {
            info!("エフェクトを変更します: {}", effect_name);
            // エフェクト変更処理
        });
        
        container.add(&effect_selector);
        
        // 適用ボタン
        let apply_button = Button::new("適用");
        apply_button.on_click(|| {
            info!("テーマを適用します");
            // テーマ適用処理
        });
        
        container.add(&apply_button);
        
        Box::new(container)
    }
    
    /// 基本コンポーネントタブを作成
    fn create_basic_components_tab(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // ボタンセクション
        let button_card = Card::new("ボタン");
        let button_container = crate::ui::toolkit::core::Container::new_horizontal();
        
        // 標準ボタン
        let standard_button = Button::new("標準ボタン");
        button_container.add(&standard_button);
        
        // プライマリボタン
        let primary_button = Button::new("プライマリ");
        primary_button.set_primary(true);
        button_container.add(&primary_button);
        
        // アウトラインボタン
        let outline_button = Button::new("アウトライン");
        outline_button.set_outline(true);
        button_container.add(&outline_button);
        
        // 無効ボタン
        let disabled_button = Button::new("無効");
        disabled_button.set_enabled(false);
        button_container.add(&disabled_button);
        
        button_card.set_content(button_container);
        container.add(&button_card);
        
        // フォーム要素セクション
        let form_card = Card::new("フォーム要素");
        let form_container = crate::ui::toolkit::core::Container::new_vertical();
        
        // テキストフィールド
        let text_field = TextField::new();
        text_field.set_placeholder("テキストを入力...");
        form_container.add(&text_field);
        
        // チェックボックス
        let checkbox = Checkbox::new("チェックボックス");
        form_container.add(&checkbox);
        
        // ラジオボタン
        let radio_container = crate::ui::toolkit::core::Container::new_horizontal();
        let radio1 = RadioButton::new("オプション1", "group1");
        let radio2 = RadioButton::new("オプション2", "group1");
        let radio3 = RadioButton::new("オプション3", "group1");
        radio_container.add(&radio1);
        radio_container.add(&radio2);
        radio_container.add(&radio3);
        form_container.add(&radio_container);
        
        // スライダー
        let slider = Slider::new(0, 100);
        slider.set_value(50);
        form_container.add(&slider);
        
        form_card.set_content(form_container);
        container.add(&form_card);
        
        // プログレスインジケータセクション
        let progress_card = Card::new("進行状況インジケータ");
        let progress_container = crate::ui::toolkit::core::Container::new_vertical();
        
        // プログレスバー (確定的)
        let progress_bar = ProgressBar::new();
        progress_bar.set_progress(0.7);
        progress_container.add(&progress_bar);
        
        // プログレスバー (不確定)
        let indeterminate_bar = ProgressBar::new();
        indeterminate_bar.set_indeterminate(true);
        progress_container.add(&indeterminate_bar);
        
        progress_card.set_content(progress_container);
        container.add(&progress_card);
        
        Box::new(container)
    }
    
    /// カラーパレットタブを作成
    fn create_color_palette_tab(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // 現在のカラーパレットを取得
        let palette = self.theme_engine.get_color_palette();
        
        // メインカラー
        let main_card = Card::new("メインカラー");
        let main_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let primary_row = self.create_color_row("プライマリ", &palette.primary);
        main_container.add(&primary_row);
        
        let secondary_row = self.create_color_row("セカンダリ", &palette.secondary);
        main_container.add(&secondary_row);
        
        let accent_row = self.create_color_row("アクセント", &palette.accent);
        main_container.add(&accent_row);
        
        main_card.set_content(main_container);
        container.add(&main_card);
        
        // 背景・前景色
        let bg_card = Card::new("背景・前景色");
        let bg_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let background_row = self.create_color_row("背景", &palette.background);
        bg_container.add(&background_row);
        
        let foreground_row = self.create_color_row("前景", &palette.foreground);
        bg_container.add(&foreground_row);
        
        bg_card.set_content(bg_container);
        container.add(&bg_card);
        
        // ステータスカラー
        let status_card = Card::new("ステータスカラー");
        let status_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let success_row = self.create_color_row("成功", &palette.success);
        status_container.add(&success_row);
        
        let warning_row = self.create_color_row("警告", &palette.warning);
        status_container.add(&warning_row);
        
        let error_row = self.create_color_row("エラー", &palette.error);
        status_container.add(&error_row);
        
        let info_row = self.create_color_row("情報", &palette.info);
        status_container.add(&info_row);
        
        status_card.set_content(status_container);
        container.add(&status_card);
        
        Box::new(container)
    }
    
    /// 色表示行を作成
    fn create_color_row(&self, label_text: &str, color_hex: &str) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_horizontal();
        
        // ラベル
        let label = Label::new(label_text);
        container.add(&label);
        
        // 色サンプル
        let color_sample = crate::ui::toolkit::core::ColorSample::new(color_hex);
        container.add(&color_sample);
        
        // HEX値
        let hex_label = Label::new(color_hex);
        container.add(&hex_label);
        
        Box::new(container)
    }
    
    /// タイポグラフィタブを作成
    fn create_typography_tab(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // 現在のフォント設定を取得
        let font_settings = self.theme_engine.get_font_settings();
        
        // フォントファミリー
        let family_card = Card::new("フォントファミリー");
        let family_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let family_label = Label::new(&format!("メインフォント: {}", font_settings.family));
        family_container.add(&family_label);
        
        if let Some(heading_family) = &font_settings.heading_family {
            let heading_label = Label::new(&format!("見出しフォント: {}", heading_family));
            family_container.add(&heading_label);
        }
        
        let mono_label = Label::new(&format!("等幅フォント: {}", font_settings.monospace_family));
        family_container.add(&mono_label);
        
        family_card.set_content(family_container);
        container.add(&family_card);
        
        // フォントサイズ
        let size_card = Card::new("フォントサイズと行高");
        let size_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let base_size_label = Label::new(&format!("基本サイズ: {}px", font_settings.base_size));
        size_container.add(&base_size_label);
        
        let line_height_label = Label::new(&format!("行の高さ: {:.1}", font_settings.line_height));
        size_container.add(&line_height_label);
        
        // 各見出しサイズのサンプル
        let h1 = Label::new("見出し1");
        h1.set_heading_level(1);
        size_container.add(&h1);
        
        let h2 = Label::new("見出し2");
        h2.set_heading_level(2);
        size_container.add(&h2);
        
        let h3 = Label::new("見出し3");
        h3.set_heading_level(3);
        size_container.add(&h3);
        
        let body = Label::new("これは標準テキストです。読みやすさとスタイルのバランスを確認します。");
        size_container.add(&body);
        
        let small = Label::new("小さなテキスト");
        small.set_small(true);
        size_container.add(&small);
        
        size_card.set_content(size_container);
        container.add(&size_card);
        
        // レンダリング設定
        let render_card = Card::new("レンダリング設定");
        let render_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let antialias_label = Label::new(&format!("アンチエイリアス: {}", if font_settings.rendering.antialias { "有効" } else { "無効" }));
        render_container.add(&antialias_label);
        
        let subpixel_label = Label::new(&format!("サブピクセルレンダリング: {}", if font_settings.rendering.subpixel { "有効" } else { "無効" }));
        render_container.add(&subpixel_label);
        
        let hinting_label = Label::new(&format!("ヒンティング: {:?}", font_settings.rendering.hinting));
        render_container.add(&hinting_label);
        
        render_card.set_content(render_container);
        container.add(&render_card);
        
        Box::new(container)
    }
    
    /// アニメーションタブを作成
    fn create_animation_tab(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // 現在のアニメーション設定を取得
        let anim_settings = self.theme_engine.get_animation_settings();
        
        // アニメーション設定
        let settings_card = Card::new("アニメーション設定");
        let settings_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let enabled_label = Label::new(&format!("アニメーション: {}", if anim_settings.enabled { "有効" } else { "無効" }));
        settings_container.add(&enabled_label);
        
        let speed_label = Label::new(&format!("速度係数: {:.1}", anim_settings.speed_factor));
        settings_container.add(&speed_label);
        
        let transition_label = Label::new(&format!("トランジション時間: {}ms", anim_settings.transition_ms));
        settings_container.add(&transition_label);
        
        let easing_label = Label::new(&format!("イージング関数: {:?}", anim_settings.easing));
        settings_container.add(&easing_label);
        
        settings_card.set_content(settings_container);
        container.add(&settings_card);
        
        // アニメーションデモ
        let demo_card = Card::new("アニメーションデモ");
        let demo_container = crate::ui::toolkit::core::Container::new_vertical();
        
        // フェードデモボタン
        let fade_button = Button::new("フェードエフェクト");
        fade_button.on_click(|| {
            // フェードアニメーションを実行する処理
        });
        demo_container.add(&fade_button);
        
        // スライドデモボタン
        let slide_button = Button::new("スライドエフェクト");
        slide_button.on_click(|| {
            // スライドアニメーションを実行する処理
        });
        demo_container.add(&slide_button);
        
        // スケールデモボタン
        let scale_button = Button::new("スケールエフェクト");
        scale_button.on_click(|| {
            // スケールアニメーションを実行する処理
        });
        demo_container.add(&scale_button);
        
        // ブラーデモボタン
        let blur_button = Button::new("ブラーエフェクト");
        blur_button.on_click(|| {
            // ブラーアニメーションを実行する処理
        });
        demo_container.add(&blur_button);
        
        // デモキャンバス
        let demo_canvas = crate::ui::toolkit::core::Canvas::new(400, 200);
        demo_container.add(&demo_canvas);
        
        demo_card.set_content(demo_container);
        container.add(&demo_card);
        
        // イージング関数比較
        let easing_card = Card::new("イージング関数比較");
        let easing_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let easing_canvas = crate::ui::toolkit::core::Canvas::new(600, 300);
        easing_container.add(&easing_canvas);
        
        easing_card.set_content(easing_container);
        container.add(&easing_card);
        
        Box::new(container)
    }
    
    /// 高DPIタブを作成
    fn create_hidpi_tab(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // 現在のディスプレイ設定を取得
        let display_settings = self.theme_engine.get_display_settings();
        
        // スケーリング設定
        let scaling_card = Card::new("スケーリング設定");
        let scaling_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let mode_label = Label::new(&format!("高DPIモード: {:?}", display_settings.hidpi_mode));
        scaling_container.add(&mode_label);
        
        let factor_label = Label::new(&format!("スケールファクター: {:.2}", display_settings.scale_factor));
        scaling_container.add(&factor_label);
        
        let system_factor_label = Label::new(&format!("システムスケール: {:.2}", self.theme_engine.get_system_scale_factor()));
        scaling_container.add(&system_factor_label);
        
        let sharpness_label = Label::new(&format!("テキストシャープネス: {:.2}", display_settings.text_sharpness));
        scaling_container.add(&sharpness_label);
        
        // スケール選択ラジオボタン
        let scale_radio_container = crate::ui::toolkit::core::Container::new_horizontal();
        let auto_radio = RadioButton::new("自動", "scale_mode");
        let normal_radio = RadioButton::new("標準 (1x)", "scale_mode");
        let hidpi_radio = RadioButton::new("高DPI (2x)", "scale_mode");
        let custom_radio = RadioButton::new("カスタム", "scale_mode");
        
        scale_radio_container.add(&auto_radio);
        scale_radio_container.add(&normal_radio);
        scale_radio_container.add(&hidpi_radio);
        scale_radio_container.add(&custom_radio);
        
        scaling_container.add(&scale_radio_container);
        
        // カスタムスケール用スライダー
        let scale_slider = Slider::new(10, 30);
        scale_slider.set_value((display_settings.scale_factor * 10.0) as i32);
        scaling_container.add(&scale_slider);
        
        scaling_card.set_content(scaling_container);
        container.add(&scaling_card);
        
        // サンプル表示エリア
        let sample_card = Card::new("各スケールでのサンプル表示");
        let sample_container = crate::ui::toolkit::core::Container::new_vertical();
        
        // サンプルキャンバス
        let sample_canvas = crate::ui::toolkit::core::Canvas::new(600, 300);
        sample_container.add(&sample_canvas);
        
        sample_card.set_content(sample_container);
        container.add(&sample_card);
        
        Box::new(container)
    }
    
    /// ステータスバーを作成
    fn create_status_bar(&self) -> Box<dyn Widget> {
        let status_bar = crate::ui::toolkit::core::Container::new_horizontal();
        
        // 現在のテーマ情報
        let theme = self.theme_engine.get_current_theme();
        let info_label = Label::new(&format!("テーマ: {} | モード: {:?} | 作者: {}", 
            theme.name, 
            theme.mode, 
            theme.author.unwrap_or_else(|| "不明".to_string())
        ));
        
        status_bar.add(&info_label);
        
        Box::new(status_bar)
    }
} 