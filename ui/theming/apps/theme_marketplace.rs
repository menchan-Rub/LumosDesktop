// LumosDesktop テーママーケットプレイス
// テーマの検索、ダウンロード、評価、共有を行うアプリケーション

use crate::ui::theming::engine::ThemeEngine;
use crate::ui::theming::themes::theme_library::{ThemeLibrary, ThemeInfo, ThemeCategory};
use crate::ui::toolkit::core::{Application, Window, Widget, Dialog, AsyncTask};
use crate::ui::toolkit::controls::{
    Button, Label, TextField, Switch, Card, 
    TabView, Tab, Panel, ImageView, ProgressBar,
    StarRating, GridView, ListView, SearchBox
};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use log::{info, warn, error, debug};
use tokio::runtime::Runtime;

/// マーケットプレイスアプリケーション
pub struct ThemeMarketplaceApp {
    /// アプリケーションインスタンス
    app: Application,
    /// メインウィンドウ
    window: Window,
    /// テーマエンジンの参照
    theme_engine: Arc<ThemeEngine>,
    /// テーマライブラリ
    theme_library: Arc<ThemeLibrary>,
    /// 非同期ランタイム
    runtime: Runtime,
    /// 検索結果
    search_results: Vec<ThemeInfo>,
    /// 選択中のテーマ
    selected_theme: Option<ThemeInfo>,
    /// カテゴリフィルター
    category_filter: Option<ThemeCategory>,
    /// 検索クエリ
    search_query: String,
    /// ダウンロード中のテーマID
    downloading_theme_id: Option<String>,
    /// ダウンロード進捗（0.0-1.0）
    download_progress: f32,
    /// インストール済みテーマ一覧
    installed_themes: Vec<ThemeInfo>,
}

impl ThemeMarketplaceApp {
    /// 新しいテーママーケットプレイスを作成
    pub fn new(theme_engine: Arc<ThemeEngine>, theme_library: Arc<ThemeLibrary>) -> Self {
        let app = Application::new("theme_marketplace", "LumosDesktop テーママーケットプレイス");
        let window = Window::new(&app, "テーママーケットプレイス", 1200, 800);
        
        // 非同期ランタイムの作成
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");
        
        let mut marketplace = Self {
            app,
            window,
            theme_engine,
            theme_library,
            runtime,
            search_results: Vec::new(),
            selected_theme: None,
            category_filter: None,
            search_query: String::new(),
            downloading_theme_id: None,
            download_progress: 0.0,
            installed_themes: Vec::new(),
        };
        
        // インストール済みテーマを読み込む
        marketplace.load_installed_themes();
        
        marketplace
    }
    
    /// インストール済みテーマを読み込む
    fn load_installed_themes(&mut self) {
        // テーマライブラリからローカルテーマをスキャンして取得
        self.installed_themes = self.theme_library.scan_local_themes();
    }
    
    /// アプリケーションを実行
    pub fn run(&mut self) {
        // UIをセットアップ
        self.setup_ui();
        
        // 初期検索を実行
        self.search_marketplace("");
        
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
        
        // メインコンテンツエリアを作成
        let content_panel = Panel::new_split_horizontal(0.7);
        
        // 左側：テーマグリッド
        let grid_container = self.create_theme_grid();
        content_panel.set_first_child(&grid_container);
        
        // 右側：テーマ詳細
        let details_container = self.create_theme_details();
        content_panel.set_second_child(&details_container);
        
        main_container.add(&content_panel);
        
        // ステータスバーを追加
        let status_bar = self.create_status_bar();
        main_container.add(&status_bar);
    }
    
    /// 上部バーを作成
    fn create_top_bar(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // 検索バー
        let search_bar = crate::ui::toolkit::core::Container::new_horizontal();
        
        // 検索ボックス
        let search_box = SearchBox::new();
        search_box.set_placeholder("テーマを検索...");
        search_box.on_search(|query| {
            // 検索を実行
            info!("検索クエリ: {}", query);
        });
        search_bar.add(&search_box);
        
        // カテゴリフィルター
        let category_label = Label::new("カテゴリ:");
        search_bar.add(&category_label);
        
        let category_dropdown = crate::ui::toolkit::controls::Dropdown::new();
        category_dropdown.add_item("すべて");
        category_dropdown.add_item("ライト");
        category_dropdown.add_item("ダーク");
        category_dropdown.add_item("ハイコントラスト");
        category_dropdown.add_item("ミニマル");
        category_dropdown.add_item("カラフル");
        category_dropdown.add_item("モダン");
        category_dropdown.add_item("クラシック");
        
        category_dropdown.set_selected_item("すべて");
        category_dropdown.on_selection_changed(|category| {
            // カテゴリでフィルタリング
            info!("カテゴリ: {}", category);
        });
        search_bar.add(&category_dropdown);
        
        // ソート順
        let sort_label = Label::new("並び順:");
        search_bar.add(&sort_label);
        
        let sort_dropdown = crate::ui::toolkit::controls::Dropdown::new();
        sort_dropdown.add_item("人気順");
        sort_dropdown.add_item("最新順");
        sort_dropdown.add_item("評価順");
        sort_dropdown.add_item("名前順");
        
        sort_dropdown.set_selected_item("人気順");
        sort_dropdown.on_selection_changed(|sort_order| {
            // ソート順を変更
            info!("ソート順: {}", sort_order);
        });
        search_bar.add(&sort_dropdown);
        
        container.add(&search_bar);
        
        // タブバー
        let tab_bar = TabView::new();
        
        // 注目のテーマタブ
        let featured_tab = self.create_featured_themes();
        tab_bar.add_tab(Tab::new("注目のテーマ", featured_tab));
        
        // すべてのテーマタブ
        let all_themes_tab = self.create_empty_container();
        tab_bar.add_tab(Tab::new("すべてのテーマ", all_themes_tab));
        
        // インストール済みタブ
        let installed_tab = self.create_empty_container();
        tab_bar.add_tab(Tab::new("インストール済み", installed_tab));
        
        // 更新タブ
        let updates_tab = self.create_empty_container();
        tab_bar.add_tab(Tab::new("アップデート", updates_tab));
        
        container.add(&tab_bar);
        
        Box::new(container)
    }
    
    /// 空のコンテナを作成（プレースホルダー用）
    fn create_empty_container(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        let label = Label::new("コンテンツを読み込み中...");
        container.add(&label);
        
        Box::new(container)
    }
    
    /// 注目のテーマを作成
    fn create_featured_themes(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // バナーエリア
        let banner = ImageView::new();
        banner.set_size(1000, 200);
        banner.set_fit_mode(crate::ui::toolkit::core::ImageFitMode::Cover);
        // バナー画像をセット（実際の実装では、サーバーから取得したバナー画像をセット）
        container.add(&banner);
        
        // 特集テーマのタイトル
        let featured_title = Label::new("今週の特集テーマ");
        featured_title.set_heading_level(2);
        container.add(&featured_title);
        
        // 特集テーマのグリッド
        let featured_grid = GridView::new();
        featured_grid.set_columns(3);
        featured_grid.set_cell_size(320, 240);
        
        // 実際の実装では、サーバーから取得した特集テーマをセット
        
        container.add(&featured_grid);
        
        Box::new(container)
    }
    
    /// テーマグリッドを作成
    fn create_theme_grid(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // グリッドヘッダー
        let grid_header = crate::ui::toolkit::core::Container::new_horizontal();
        
        let results_label = Label::new("検索結果: 0 件");
        grid_header.add(&results_label);
        
        // 表示タイプ切替
        let view_type_container = crate::ui::toolkit::core::Container::new_horizontal();
        
        let grid_button = Button::new("");
        grid_button.set_icon("view-grid");
        grid_button.set_tooltip("グリッド表示");
        grid_button.set_toggle(true);
        grid_button.set_toggled(true);
        grid_button.on_toggle(|toggled| {
            if toggled {
                info!("グリッド表示に切替");
            }
        });
        view_type_container.add(&grid_button);
        
        let list_button = Button::new("");
        list_button.set_icon("view-list");
        list_button.set_tooltip("リスト表示");
        list_button.set_toggle(true);
        list_button.on_toggle(|toggled| {
            if toggled {
                info!("リスト表示に切替");
            }
        });
        view_type_container.add(&list_button);
        
        grid_header.add(&view_type_container);
        
        container.add(&grid_header);
        
        // テーマグリッド
        let grid = GridView::new();
        grid.set_columns(3);
        grid.set_cell_size(250, 200);
        
        // テーマアイテムを追加（実際の実装では検索結果から動的に生成）
        
        grid.on_item_selected(|index| {
            // テーマ選択
            info!("テーマを選択しました: {}", index);
        });
        
        container.add(&grid);
        
        // ページネーション
        let pagination = crate::ui::toolkit::core::Container::new_horizontal();
        
        let prev_button = Button::new("前へ");
        prev_button.on_click(|| {
            // 前ページに移動
            info!("前ページに移動");
        });
        pagination.add(&prev_button);
        
        let page_label = Label::new("1 / 1");
        pagination.add(&page_label);
        
        let next_button = Button::new("次へ");
        next_button.on_click(|| {
            // 次ページに移動
            info!("次ページに移動");
        });
        pagination.add(&next_button);
        
        container.add(&pagination);
        
        Box::new(container)
    }
    
    /// テーマ詳細を作成
    fn create_theme_details(&self) -> Box<dyn Widget> {
        let container = crate::ui::toolkit::core::Container::new_vertical();
        
        // テーマプレビュー
        let preview = ImageView::new();
        preview.set_size(350, 200);
        preview.set_fit_mode(crate::ui::toolkit::core::ImageFitMode::Contain);
        container.add(&preview);
        
        // テーマ情報カード
        let info_card = Card::new("テーマ情報");
        let info_container = crate::ui::toolkit::core::Container::new_vertical();
        
        // テーマ名
        let name_label = Label::new("テーマ名");
        name_label.set_heading_level(2);
        info_container.add(&name_label);
        
        // 作者
        let author_label = Label::new("作者: -");
        info_container.add(&author_label);
        
        // バージョン
        let version_label = Label::new("バージョン: -");
        info_container.add(&version_label);
        
        // 評価
        let rating_container = crate::ui::toolkit::core::Container::new_horizontal();
        let rating_label = Label::new("評価:");
        rating_container.add(&rating_label);
        
        let rating = StarRating::new();
        rating.set_rating(0.0);
        rating.set_readonly(true);
        rating_container.add(&rating);
        
        info_container.add(&rating_container);
        
        // ダウンロード数
        let downloads_label = Label::new("ダウンロード数: -");
        info_container.add(&downloads_label);
        
        // 更新日
        let updated_label = Label::new("更新日: -");
        info_container.add(&updated_label);
        
        info_card.set_content(info_container);
        container.add(&info_card);
        
        // 説明カード
        let desc_card = Card::new("説明");
        let desc_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let description = Label::new("テーマの説明がここに表示されます。");
        desc_container.add(&description);
        
        desc_card.set_content(desc_container);
        container.add(&desc_card);
        
        // アクションボタン
        let action_container = crate::ui::toolkit::core::Container::new_horizontal();
        
        let download_button = Button::new("ダウンロード");
        download_button.set_primary(true);
        download_button.on_click(|| {
            // テーマをダウンロード
            info!("テーマをダウンロードします");
        });
        action_container.add(&download_button);
        
        let preview_button = Button::new("プレビュー");
        preview_button.on_click(|| {
            // テーマをプレビュー
            info!("テーマをプレビューします");
        });
        action_container.add(&preview_button);
        
        let apply_button = Button::new("適用");
        apply_button.on_click(|| {
            // テーマを適用
            info!("テーマを適用します");
        });
        action_container.add(&apply_button);
        
        container.add(&action_container);
        
        // ユーザー評価セクション
        let rating_card = Card::new("このテーマを評価");
        let rating_container = crate::ui::toolkit::core::Container::new_vertical();
        
        let user_rating = StarRating::new();
        user_rating.set_rating(0.0);
        user_rating.on_rating_changed(|rating| {
            // ユーザー評価を送信
            info!("ユーザー評価: {}", rating);
        });
        rating_container.add(&user_rating);
        
        rating_card.set_content(rating_container);
        container.add(&rating_card);
        
        Box::new(container)
    }
    
    /// ステータスバーを作成
    fn create_status_bar(&self) -> Box<dyn Widget> {
        let status_bar = crate::ui::toolkit::core::Container::new_horizontal();
        
        // 接続状態
        let connection_label = Label::new("マーケットプレイス: 接続済み");
        status_bar.add(&connection_label);
        
        // ダウンロード進捗
        let progress_container = crate::ui::toolkit::core::Container::new_horizontal();
        let progress_label = Label::new("ダウンロード:");
        progress_container.add(&progress_label);
        
        let progress_bar = ProgressBar::new();
        progress_bar.set_visible(false); // 初期状態では非表示
        progress_container.add(&progress_bar);
        
        status_bar.add(&progress_container);
        
        Box::new(status_bar)
    }
    
    /// マーケットプレイスでテーマを検索
    fn search_marketplace(&mut self, query: &str) {
        // 検索クエリを保存
        self.search_query = query.to_string();
        
        // 非同期タスクを作成
        let query_string = query.to_string();
        let category_filter = self.category_filter.clone();
        let theme_library = Arc::clone(&self.theme_library);
        
        let task = AsyncTask::new(move || {
            let rt = tokio::runtime::Handle::current();
            
            // 非同期で検索を実行
            rt.block_on(async {
                theme_library.search_marketplace_themes(
                    if query_string.is_empty() { None } else { Some(&query_string) },
                    category_filter
                ).await
            })
        });
        
        // 検索結果を受け取るコールバック
        let window = self.window.clone();
        task.on_completed(move |result| {
            match result {
                Ok(themes) => {
                    info!("検索結果: {} 件のテーマが見つかりました", themes.len());
                    
                    // UI更新
                    // 実際の実装では、検索結果でグリッドを更新する
                    let results_label = window.find_widget_by_id("results_label")
                        .and_then(|w| w.downcast::<Label>());
                    
                    if let Some(label) = results_label {
                        label.set_text(&format!("検索結果: {} 件", themes.len()));
                    }
                    
                    // グリッドを更新
                    // update_theme_grid(&window, &themes);
                },
                Err(e) => {
                    error!("テーマの検索に失敗しました: {}", e);
                    
                    // エラーダイアログを表示
                    let dialog = Dialog::new(&window, "検索エラー");
                    dialog.set_message(&format!("テーマの検索中にエラーが発生しました: {}", e));
                    dialog.show();
                }
            }
        });
        
        // タスクを実行
        task.run();
    }
    
    /// テーマをダウンロード
    fn download_theme(&mut self, theme_id: &str) {
        // ダウンロード状態を更新
        self.downloading_theme_id = Some(theme_id.to_string());
        self.download_progress = 0.0;
        
        // 非同期タスクを作成
        let theme_id_string = theme_id.to_string();
        let theme_library = Arc::clone(&self.theme_library);
        
        let task = AsyncTask::new(move || {
            let rt = tokio::runtime::Handle::current();
            
            // 非同期でダウンロードを実行
            rt.block_on(async {
                theme_library.download_theme(&theme_id_string).await
            })
        });
        
        // ダウンロード完了のコールバック
        let window = self.window.clone();
        let theme_id_copy = theme_id.to_string();
        task.on_completed(move |result| {
            match result {
                Ok(theme_info) => {
                    info!("テーマ「{}」のダウンロードが完了しました", theme_info.name);
                    
                    // UI更新
                    // 実際の実装では、ダウンロード状態の表示を更新する
                    
                    // 成功ダイアログを表示
                    let dialog = Dialog::new(&window, "ダウンロード完了");
                    dialog.set_message(&format!("テーマ「{}」のダウンロードが完了しました。適用しますか？", theme_info.name));
                    dialog.add_button("適用", Box::new(move || {
                        info!("テーマ「{}」を適用します", theme_info.name);
                        // テーマを適用する処理
                    }));
                    dialog.add_button("後で", Box::new(|| {}));
                    dialog.show();
                },
                Err(e) => {
                    error!("テーマ「{}」のダウンロードに失敗しました: {}", theme_id_copy, e);
                    
                    // エラーダイアログを表示
                    let dialog = Dialog::new(&window, "ダウンロードエラー");
                    dialog.set_message(&format!("テーマのダウンロード中にエラーが発生しました: {}", e));
                    dialog.show();
                }
            }
        });
        
        // 進捗コールバックを設定（実際の実装ではダウンロード進捗を受け取る）
        task.on_progress(|progress| {
            // プログレスバーを更新
            let progress_bar = self.window.find_widget_by_id("download_progress")
                .and_then(|w| w.downcast::<ProgressBar>());
                
            if let Some(bar) = progress_bar {
                bar.set_visible(true);
                bar.set_progress(progress);
            }
        });
        
        // タスクを実行
        task.run();
    }
    
    /// テーマを適用
    fn apply_theme(&self, theme_name: &str) -> Result<(), String> {
        // テーマを適用
        self.theme_library.apply_theme(theme_name, "fade")
    }
    
    /// テーマにユーザー評価を送信
    fn rate_theme(&self, theme_id: &str, rating: f32) {
        // 非同期タスクを作成
        let theme_id_string = theme_id.to_string();
        let theme_library = Arc::clone(&self.theme_library);
        
        let task = AsyncTask::new(move || {
            let rt = tokio::runtime::Handle::current();
            
            // 非同期で評価を送信
            rt.block_on(async {
                theme_library.rate_theme(&theme_id_string, rating).await
            })
        });
        
        // 評価送信完了のコールバック
        let window = self.window.clone();
        task.on_completed(move |result| {
            match result {
                Ok(_) => {
                    info!("テーマの評価を送信しました");
                    
                    // 成功メッセージを表示
                    let dialog = Dialog::new(&window, "評価完了");
                    dialog.set_message("テーマの評価を送信しました。ご協力ありがとうございます。");
                    dialog.add_button("閉じる", Box::new(|| {}));
                    dialog.show();
                },
                Err(e) => {
                    error!("テーマの評価送信に失敗しました: {}", e);
                    
                    // エラーダイアログを表示
                    let dialog = Dialog::new(&window, "評価エラー");
                    dialog.set_message(&format!("テーマの評価送信中にエラーが発生しました: {}", e));
                    dialog.show();
                }
            }
        });
        
        // タスクを実行
        task.run();
    }
    
    /// テーマのプレビューを表示
    fn preview_theme(&self, theme_info: &ThemeInfo) {
        // プレビューアプリを起動
        if let Err(e) = std::process::Command::new("lumos-theme-preview")
            .arg(&theme_info.file_path)
            .spawn() {
            error!("プレビューアプリの起動に失敗しました: {}", e);
            
            // エラーダイアログを表示
            let dialog = Dialog::new(&self.window, "プレビューエラー");
            dialog.set_message(&format!("プレビューアプリの起動に失敗しました: {}", e));
            dialog.show();
        }
    }
    
    /// テーマをマーケットプレイスで共有
    fn share_theme(&self, theme_info: &ThemeInfo) {
        // 非同期タスクを作成
        let theme_info_clone = theme_info.clone();
        let theme_library = Arc::clone(&self.theme_library);
        
        let task = AsyncTask::new(move || {
            let rt = tokio::runtime::Handle::current();
            
            // 非同期で共有を実行
            rt.block_on(async {
                theme_library.share_theme(&theme_info_clone).await
            })
        });
        
        // 共有完了のコールバック
        let window = self.window.clone();
        let theme_name = theme_info.name.clone();
        task.on_completed(move |result| {
            match result {
                Ok(id) => {
                    info!("テーマ「{}」をマーケットプレイスで共有しました: ID={}", theme_name, id);
                    
                    // 成功ダイアログを表示
                    let dialog = Dialog::new(&window, "共有完了");
                    dialog.set_message(&format!(
                        "テーマ「{}」をマーケットプレイスで共有しました。\n\nテーマID: {}",
                        theme_name, id
                    ));
                    dialog.add_button("閉じる", Box::new(|| {}));
                    dialog.show();
                },
                Err(e) => {
                    error!("テーマ「{}」の共有に失敗しました: {}", theme_name, e);
                    
                    // エラーダイアログを表示
                    let dialog = Dialog::new(&window, "共有エラー");
                    dialog.set_message(&format!("テーマの共有中にエラーが発生しました: {}", e));
                    dialog.show();
                }
            }
        });
        
        // タスクを実行
        task.run();
    }
} 