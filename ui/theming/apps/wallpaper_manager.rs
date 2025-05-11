// LumosDesktop 壁紙マネージャー
// システム壁紙の設定と管理アプリケーション

use crate::ui::theming::engine::ThemeEngine;
use crate::ui::toolkit::core::{Application, Window, Widget, FileBrowser};
use crate::ui::toolkit::controls::{
    Button, Label, Slider, Switch, Card, 
    GridView, TabView, Tab, Panel, ImageView
};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs;
use log::{info, error, warn};

/// 壁紙情報
struct WallpaperInfo {
    /// ファイルパス
    path: PathBuf,
    /// サムネイルパス
    thumbnail_path: Option<PathBuf>,
    /// 説明
    description: Option<String>,
    /// タグ
    tags: Vec<String>,
}

/// 壁紙マネージャーアプリケーション
pub struct WallpaperManagerApp {
    /// アプリケーションインスタンス
    app: Application,
    /// メインウィンドウ
    window: Window,
    /// テーマエンジンの参照
    theme_engine: Arc<ThemeEngine>,
    /// 壁紙コレクション
    wallpapers: Vec<WallpaperInfo>,
    /// 現在の壁紙パス
    current_wallpaper: Option<PathBuf>,
    /// 壁紙ディレクトリ
    wallpaper_dirs: Vec<PathBuf>,
}

impl WallpaperManagerApp {
    /// 新しい壁紙マネージャーを作成
    pub fn new(theme_engine: Arc<ThemeEngine>) -> Self {
        let app = Application::new("wallpaper_manager", "LumosDesktop 壁紙マネージャー");
        let window = Window::new(&app, "壁紙マネージャー", 1000, 700);
        
        // システム壁紙ディレクトリ
        let mut wallpaper_dirs = vec![
            PathBuf::from("/usr/share/backgrounds"),
            PathBuf::from("/usr/share/lumos/backgrounds"),
        ];
        
        // ホームディレクトリの壁紙フォルダを追加
        if let Some(home) = dirs::home_dir() {
            wallpaper_dirs.push(home.join(".local/share/backgrounds"));
            wallpaper_dirs.push(home.join("Pictures/Wallpapers"));
        }
        
        // 現在のテーマから壁紙を取得
        let current_wallpaper = theme_engine.get_current_theme().wallpaper.clone();
        
        Self {
            app,
            window,
            theme_engine,
            wallpapers: Vec::new(),
            current_wallpaper,
            wallpaper_dirs,
        }
    }
    
    /// アプリケーションを実行
    pub fn run(&mut self) {
        // 壁紙を読み込む
        self.load_wallpapers();
        
        // UIをセットアップ
        self.setup_ui();
        
        // アプリケーションを実行
        self.app.run();
    }
    
    /// 壁紙を読み込む
    fn load_wallpapers(&mut self) {
        self.wallpapers.clear();
        
        for dir in &self.wallpaper_dirs {
            // ディレクトリが存在するか確認
            if !dir.exists() || !dir.is_dir() {
                continue;
            }
            
            // ディレクトリ内の画像ファイルを検索
            match fs::read_dir(dir) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            
                            // 画像ファイルかどうかを拡張子で判断
                            if let Some(ext) = path.extension() {
                                let ext = ext.to_string_lossy().to_lowercase();
                                if ["jpg", "jpeg", "png", "webp", "bmp"].contains(&ext.as_str()) {
                                    // 壁紙情報を追加
                                    self.wallpapers.push(WallpaperInfo {
                                        path: path.clone(),
                                        thumbnail_path: None,
                                        description: None,
                                        tags: Vec::new(),
                                    });
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
        
        info!("{}枚の壁紙を読み込みました", self.wallpapers.len());
    }
    
    /// UIをセットアップ
    fn setup_ui(&mut self) {
        // メインコンテナを作成
        let main_container = self.window.get_main_container();
        
        // ツールバーを追加
        let toolbar = self.create_toolbar();
        main_container.add(&toolbar);
        
        // タブビューを作成
        let tab_view = TabView::new();
        
        // 壁紙グリッドタブ
        let grid_tab = self.create_wallpaper_grid();
        tab_view.add_tab(Tab::new("壁紙コレクション", grid_tab));
        
        // 壁紙プレビュータブ
        let preview_tab = self.create_wallpaper_preview();
        tab_view.add_tab(Tab::new("プレビュー", preview_tab));
        
        // 壁紙設定タブ
        let settings_tab = self.create_wallpaper_settings();
        tab_view.add_tab(Tab::new("設定", settings_tab));
        
        main_container.add(&tab_view);
        
        // ステータスバーを追加
        let status_bar = self.create_status_bar();
        main_container.add(&status_bar);
    }
    
    /// ツールバーを作成
    fn create_toolbar(&self) -> Box<dyn Widget> {
        let toolbar = crate::ui::toolkit::core::Container::new_horizontal();
        
        // 壁紙追加ボタン
        let add_button = Button::new("壁紙を追加");
        add_button.set_icon("list-add");
        add_button.on_click(|| {
            // 壁紙追加ダイアログを表示
            info!("壁紙を追加します");
        });
        toolbar.add(&add_button);
        
        // 壁紙フォルダを開くボタン
        let open_folder_button = Button::new("フォルダを開く");
        open_folder_button.set_icon("folder");
        open_folder_button.on_click(|| {
            // 壁紙フォルダを開く
            info!("壁紙フォルダを開きます");
        });
        toolbar.add(&open_folder_button);
        
        // セパレータ
        toolbar.add_separator();
        
        // 壁紙を適用ボタン
        let apply_button = Button::new("壁紙を適用");
        apply_button.set_icon("document-send");
        apply_button.set_primary(true);
        apply_button.on_click(|| {
            // 選択した壁紙を適用
            info!("壁紙を適用します");
        });
        toolbar.add(&apply_button);
        
        Box::new(toolbar)
    }
    
    /// 壁紙グリッドを作成
    fn create_wallpaper_grid(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // 検索・フィルターバー
        let filter_bar = crate::ui::toolkit::core::Container::new_horizontal();
        
        // 検索フィールド
        let search_field = crate::ui::toolkit::controls::TextField::new();
        search_field.set_placeholder("壁紙を検索...");
        search_field.on_change(|text| {
            // 検索テキストでフィルタリング
            info!("検索: {}", text);
        });
        filter_bar.add(&search_field);
        
        // 表示タイプ切替
        let view_type_label = Label::new("表示形式:");
        filter_bar.add(&view_type_label);
        
        let view_type_dropdown = crate::ui::toolkit::controls::Dropdown::new();
        view_type_dropdown.add_item("グリッド (大)");
        view_type_dropdown.add_item("グリッド (中)");
        view_type_dropdown.add_item("グリッド (小)");
        view_type_dropdown.add_item("リスト");
        view_type_dropdown.set_selected_item("グリッド (中)");
        view_type_dropdown.on_selection_changed(|view_type| {
            // 表示タイプを変更
            info!("表示タイプ: {}", view_type);
        });
        filter_bar.add(&view_type_dropdown);
        
        container.add(&filter_bar);
        
        // 壁紙グリッド
        let grid = GridView::new();
        grid.set_columns(4);
        
        // 壁紙アイテムを追加
        for (index, wallpaper) in self.wallpapers.iter().enumerate() {
            let item = self.create_wallpaper_item(index, wallpaper);
            grid.add_item(&item);
        }
        
        grid.on_item_selected(|index| {
            // 壁紙選択
            info!("壁紙を選択しました: {}", index);
        });
        
        grid.on_item_activated(|index| {
            // 壁紙をアクティベート（ダブルクリックなど）
            info!("壁紙をアクティベートしました: {}", index);
        });
        
        container.add(&grid);
        
        Box::new(container)
    }
    
    /// 壁紙アイテムを作成
    fn create_wallpaper_item(&self, index: usize, wallpaper: &WallpaperInfo) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // サムネイル
        let thumbnail = ImageView::new();
        thumbnail.set_image_path(&wallpaper.path.to_string_lossy());
        thumbnail.set_size(200, 120);
        thumbnail.set_fit_mode(crate::ui::toolkit::core::ImageFitMode::Cover);
        container.add(&thumbnail);
        
        // ファイル名
        let filename = wallpaper.path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
            
        let name_label = Label::new(&filename);
        container.add(&name_label);
        
        // メニューを追加
        let context_menu = crate::ui::toolkit::controls::ContextMenu::new();
        context_menu.add_item("適用", Box::new(move || {
            info!("壁紙を適用します: {}", index);
        }));
        context_menu.add_item("プレビュー", Box::new(move || {
            info!("壁紙をプレビューします: {}", index);
        }));
        context_menu.add_item("削除", Box::new(move || {
            info!("壁紙を削除します: {}", index);
        }));
        
        container.set_context_menu(&context_menu);
        
        Box::new(container)
    }
    
    /// 壁紙プレビューを作成
    fn create_wallpaper_preview(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // プレビュー画像
        let preview = ImageView::new();
        
        // 現在の壁紙があれば表示
        if let Some(ref wallpaper_path) = self.current_wallpaper {
            preview.set_image_path(&wallpaper_path.to_string_lossy());
        }
        
        preview.set_fit_mode(crate::ui::toolkit::core::ImageFitMode::Contain);
        container.add(&preview);
        
        // 壁紙情報
        let info_card = Card::new("壁紙情報");
        let info_container = crate::ui::toolkit::core::Container::new_vertical();
        
        // ファイル名
        let filename = self.current_wallpaper.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "選択されていません".to_string());
            
        let name_label = Label::new(&format!("ファイル名: {}", filename));
        info_container.add(&name_label);
        
        // 解像度
        let resolution_label = Label::new("解像度: -");
        info_container.add(&resolution_label);
        
        // ファイルサイズ
        let size_label = Label::new("ファイルサイズ: -");
        info_container.add(&size_label);
        
        // 場所
        let location_label = Label::new("場所: -");
        info_container.add(&location_label);
        
        info_card.set_content(info_container);
        container.add(&info_card);
        
        Box::new(container)
    }
    
    /// 壁紙設定を作成
    fn create_wallpaper_settings(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // 表示設定
        let display_card = Card::new("表示設定");
        let display_container = crate::ui::toolkit::core::Container::new_vertical();
        
        // 表示モード
        let mode_container = crate::ui::toolkit::core::Container::new_horizontal();
        let mode_label = Label::new("表示モード:");
        mode_container.add(&mode_label);
        
        let mode_dropdown = crate::ui::toolkit::controls::Dropdown::new();
        mode_dropdown.add_item("拡大表示");
        mode_dropdown.add_item("縮小表示");
        mode_dropdown.add_item("中央表示");
        mode_dropdown.add_item("タイル表示");
        mode_dropdown.add_item("拡大縮小表示");
        mode_dropdown.set_selected_item("拡大縮小表示");
        mode_dropdown.on_selection_changed(|mode| {
            // 表示モードを変更
            info!("表示モード: {}", mode);
        });
        mode_container.add(&mode_dropdown);
        
        display_container.add(&mode_container);
        
        // 背景色
        let bg_container = crate::ui::toolkit::core::Container::new_horizontal();
        let bg_label = Label::new("背景色:");
        bg_container.add(&bg_label);
        
        let bg_color = crate::ui::toolkit::controls::ColorPicker::new();
        bg_color.set_color("#000000");
        bg_color.on_color_changed(|color| {
            // 背景色を変更
            info!("背景色: {}", color);
        });
        bg_container.add(&bg_color);
        
        display_container.add(&bg_container);
        
        display_card.set_content(display_container);
        container.add(&display_card);
        
        // 自動変更設定
        let auto_card = Card::new("自動変更設定");
        let auto_container = crate::ui::toolkit::core::Container::new_vertical();
        
        // 自動変更有効
        let auto_switch_container = crate::ui::toolkit::core::Container::new_horizontal();
        let auto_label = Label::new("壁紙を自動的に変更する:");
        auto_switch_container.add(&auto_label);
        
        let auto_switch = Switch::new();
        auto_switch.set_checked(false);
        auto_switch.on_toggle(|checked| {
            // 自動変更を有効/無効化
            info!("自動変更: {}", checked);
        });
        auto_switch_container.add(&auto_switch);
        
        auto_container.add(&auto_switch_container);
        
        // 変更間隔
        let interval_container = crate::ui::toolkit::core::Container::new_horizontal();
        let interval_label = Label::new("変更間隔:");
        interval_container.add(&interval_label);
        
        let interval_dropdown = crate::ui::toolkit::controls::Dropdown::new();
        interval_dropdown.add_item("30分");
        interval_dropdown.add_item("1時間");
        interval_dropdown.add_item("3時間");
        interval_dropdown.add_item("6時間");
        interval_dropdown.add_item("12時間");
        interval_dropdown.add_item("1日");
        interval_dropdown.set_selected_item("1時間");
        interval_dropdown.on_selection_changed(|interval| {
            // 変更間隔を設定
            info!("変更間隔: {}", interval);
        });
        interval_container.add(&interval_dropdown);
        
        auto_container.add(&interval_container);
        
        // 変更順序
        let order_container = crate::ui::toolkit::core::Container::new_horizontal();
        let order_label = Label::new("変更順序:");
        order_container.add(&order_label);
        
        let order_dropdown = crate::ui::toolkit::controls::Dropdown::new();
        order_dropdown.add_item("順番");
        order_dropdown.add_item("ランダム");
        order_dropdown.set_selected_item("ランダム");
        order_dropdown.on_selection_changed(|order| {
            // 変更順序を設定
            info!("変更順序: {}", order);
        });
        order_container.add(&order_dropdown);
        
        auto_container.add(&order_container);
        
        auto_card.set_content(auto_container);
        container.add(&auto_card);
        
        // フォルダ設定
        let folders_card = Card::new("壁紙フォルダ");
        let folders_container = crate::ui::toolkit::core::Container::new_vertical();
        
        // フォルダリスト
        let folder_list = crate::ui::toolkit::controls::ListView::new();
        
        for dir in &self.wallpaper_dirs {
            folder_list.add_item(&dir.to_string_lossy());
        }
        
        folders_container.add(&folder_list);
        
        // フォルダ追加/削除ボタン
        let folder_buttons = crate::ui::toolkit::core::Container::new_horizontal();
        
        let add_folder_button = Button::new("追加");
        add_folder_button.on_click(|| {
            // フォルダ追加ダイアログを表示
            info!("フォルダを追加します");
        });
        folder_buttons.add(&add_folder_button);
        
        let remove_folder_button = Button::new("削除");
        remove_folder_button.on_click(|| {
            // 選択したフォルダを削除
            info!("フォルダを削除します");
        });
        folder_buttons.add(&remove_folder_button);
        
        folders_container.add(&folder_buttons);
        
        folders_card.set_content(folders_container);
        container.add(&folders_card);
        
        Box::new(container)
    }
    
    /// ステータスバーを作成
    fn create_status_bar(&self) -> Box<dyn Widget> {
        let status_bar = crate::ui::toolkit::core::Container::new_horizontal();
        
        // 壁紙数
        let count_label = Label::new(&format!("壁紙数: {}", self.wallpapers.len()));
        status_bar.add(&count_label);
        
        // 現在の壁紙
        let current_label = if let Some(ref path) = self.current_wallpaper {
            let filename = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());
                
            Label::new(&format!("現在の壁紙: {}", filename))
        } else {
            Label::new("現在の壁紙: 設定されていません")
        };
        
        status_bar.add(&current_label);
        
        Box::new(status_bar)
    }
    
    /// 壁紙を設定
    pub fn set_wallpaper(&self, path: &Path) -> Result<(), String> {
        // テーマの壁紙を更新
        let mut theme = self.theme_engine.get_current_theme();
        theme.wallpaper = Some(path.to_path_buf());
        
        // テーマを適用
        self.theme_engine.install_theme(theme);
        
        Ok(())
    }
} 