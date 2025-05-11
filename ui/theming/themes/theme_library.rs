// LumosDesktop テーマライブラリ
// テーマの管理とマーケットプレイス連携

use crate::ui::theming::engine::{Theme, ThemeEngine};
use log::{info, warn, error, debug};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use reqwest::Client;

/// テーマカテゴリ
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeCategory {
    /// ライトテーマ
    Light,
    /// ダークテーマ
    Dark,
    /// コントラスト高
    HighContrast,
    /// ミニマル
    Minimal,
    /// カラフル
    Colorful,
    /// モダン
    Modern,
    /// クラシック
    Classic,
    /// カスタム
    Custom(String),
}

impl ThemeCategory {
    /// カテゴリの名前を取得
    pub fn name(&self) -> String {
        match self {
            ThemeCategory::Light => "ライト".to_string(),
            ThemeCategory::Dark => "ダーク".to_string(),
            ThemeCategory::HighContrast => "ハイコントラスト".to_string(),
            ThemeCategory::Minimal => "ミニマル".to_string(),
            ThemeCategory::Colorful => "カラフル".to_string(),
            ThemeCategory::Modern => "モダン".to_string(),
            ThemeCategory::Classic => "クラシック".to_string(),
            ThemeCategory::Custom(name) => name.clone(),
        }
    }
}

/// テーマ詳細情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    /// テーマ名
    pub name: String,
    /// 作者
    pub author: String,
    /// 説明
    pub description: String,
    /// バージョン
    pub version: String,
    /// カテゴリ
    pub categories: Vec<ThemeCategory>,
    /// ファイルパス
    pub file_path: PathBuf,
    /// サムネイルパス
    pub thumbnail_path: Option<PathBuf>,
    /// ダウンロード数（マーケットプレイスの場合）
    pub downloads: Option<u64>,
    /// 評価（0-5）
    pub rating: Option<f32>,
    /// 最終更新日
    pub updated_at: SystemTime,
    /// マーケットプレイスID（マーケットプレイスの場合）
    pub marketplace_id: Option<String>,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// マーケットプレイスアイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceItem {
    /// アイテムID
    pub id: String,
    /// テーマ名
    pub name: String,
    /// 作者
    pub author: String,
    /// 説明
    pub description: String,
    /// バージョン
    pub version: String,
    /// カテゴリ
    pub categories: Vec<String>,
    /// サムネイルURL
    pub thumbnail_url: String,
    /// ダウンロードURL
    pub download_url: String,
    /// ダウンロード数
    pub downloads: u64,
    /// 評価（0-5）
    pub rating: f32,
    /// 最終更新日（ISO 8601形式）
    pub updated_at: String,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// テーマライブラリ
pub struct ThemeLibrary {
    /// テーマエンジンの参照
    theme_engine: Arc<ThemeEngine>,
    /// テーマディレクトリ
    theme_dirs: Vec<PathBuf>,
    /// テーマ情報キャッシュ
    theme_cache: RwLock<HashMap<String, ThemeInfo>>,
    /// HTTPクライアント
    http_client: Client,
    /// マーケットプレイスのURL
    marketplace_url: String,
}

impl ThemeLibrary {
    /// 新しいテーマライブラリを作成
    pub fn new(theme_engine: Arc<ThemeEngine>) -> Self {
        // システムテーマディレクトリ
        let mut theme_dirs = vec![
            PathBuf::from("/usr/share/lumos/themes"),
            PathBuf::from("/usr/local/share/lumos/themes"),
        ];
        
        // ホームディレクトリのテーマフォルダを追加
        if let Some(home) = dirs::home_dir() {
            theme_dirs.push(home.join(".local/share/lumos/themes"));
            theme_dirs.push(home.join(".config/lumos/themes"));
        }
        
        Self {
            theme_engine,
            theme_dirs,
            theme_cache: RwLock::new(HashMap::new()),
            http_client: Client::new(),
            marketplace_url: "https://marketplace.lumosdesktop.org/api/themes".to_string(),
        }
    }
    
    /// テーマディレクトリを追加
    pub fn add_theme_directory<P: AsRef<Path>>(&mut self, path: P) {
        let path_buf = path.as_ref().to_path_buf();
        if !self.theme_dirs.contains(&path_buf) {
            self.theme_dirs.push(path_buf);
        }
    }
    
    /// マーケットプレイスURLを設定
    pub fn set_marketplace_url(&mut self, url: &str) {
        self.marketplace_url = url.to_string();
    }
    
    /// ローカルテーマを検索
    pub fn scan_local_themes(&self) -> Vec<ThemeInfo> {
        let mut themes = Vec::new();
        let mut cache = self.theme_cache.write().unwrap();
        
        for dir in &self.theme_dirs {
            // ディレクトリが存在するか確認
            if !dir.exists() || !dir.is_dir() {
                continue;
            }
            
            // ディレクトリ内のJSONファイルを検索
            match fs::read_dir(dir) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            
                            // JSONファイルかどうかを拡張子で判断
                            if let Some(ext) = path.extension() {
                                if ext.to_string_lossy().to_lowercase() == "json" {
                                    match self.load_theme_info(&path) {
                                        Ok(info) => {
                                            // キャッシュに追加
                                            cache.insert(info.name.clone(), info.clone());
                                            themes.push(info);
                                        },
                                        Err(e) => {
                                            warn!("テーマ情報の読み込みに失敗しました: {}: {}", path.display(), e);
                                        }
                                    }
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
        
        info!("{}個のローカルテーマを読み込みました", themes.len());
        themes
    }
    
    /// テーマ情報を読み込む
    fn load_theme_info<P: AsRef<Path>>(&self, path: P) -> Result<ThemeInfo, String> {
        let path = path.as_ref();
        
        // テーマファイルを読み込む
        let theme = self.theme_engine.load_theme(path)?;
        
        // サムネイルパスを生成
        let thumbnail_path = {
            let stem = path.file_stem().unwrap_or_default();
            let parent = path.parent().unwrap_or_else(|| Path::new(""));
            let thumbnail_name = format!("{}_thumbnail.png", stem.to_string_lossy());
            let thumbnail_path = parent.join(thumbnail_name);
            
            if thumbnail_path.exists() {
                Some(thumbnail_path)
            } else {
                None
            }
        };
        
        // メタデータを収集
        let mut metadata = HashMap::new();
        for (key, value) in &theme.custom {
            if let Some(value_str) = value.as_str() {
                metadata.insert(key.clone(), value_str.to_string());
            }
        }
        
        // カテゴリを決定
        let mut categories = Vec::new();
        match theme.mode {
            crate::ui::theming::engine::ThemeMode::Light => categories.push(ThemeCategory::Light),
            crate::ui::theming::engine::ThemeMode::Dark => categories.push(ThemeCategory::Dark),
            _ => {}
        }
        
        // カスタムカテゴリがあれば追加
        if let Some(value) = metadata.get("category") {
            categories.push(ThemeCategory::Custom(value.clone()));
        }
        
        // ファイルの更新日時を取得
        let updated_at = fs::metadata(path)
            .map(|m| m.modified().unwrap_or_else(|_| SystemTime::now()))
            .unwrap_or_else(|_| SystemTime::now());
        
        Ok(ThemeInfo {
            name: theme.name,
            author: theme.author.unwrap_or_else(|| "Unknown".to_string()),
            description: theme.description.unwrap_or_else(|| "".to_string()),
            version: theme.version.unwrap_or_else(|| "1.0.0".to_string()),
            categories,
            file_path: path.to_path_buf(),
            thumbnail_path,
            downloads: None,
            rating: None,
            updated_at,
            marketplace_id: None,
            metadata,
        })
    }
    
    /// テーマ情報をキャッシュから取得
    pub fn get_theme_info(&self, name: &str) -> Option<ThemeInfo> {
        let cache = self.theme_cache.read().unwrap();
        cache.get(name).cloned()
    }
    
    /// マーケットプレイスのテーマを検索
    pub async fn search_marketplace_themes(&self, query: Option<&str>, category: Option<ThemeCategory>) -> Result<Vec<ThemeInfo>, String> {
        // APIリクエストURLを構築
        let mut url = self.marketplace_url.clone();
        
        let mut query_params = Vec::new();
        
        if let Some(q) = query {
            query_params.push(format!("q={}", urlencoding::encode(q)));
        }
        
        if let Some(cat) = category {
            query_params.push(format!("category={}", urlencoding::encode(&cat.name())));
        }
        
        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }
        
        // APIリクエストを送信
        let response = self.http_client.get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("マーケットプレイスへの接続に失敗しました: {}", e))?;
            
        if !response.status().is_success() {
            return Err(format!("マーケットプレイスから不正なレスポンスを受信しました: {}", response.status()));
        }
        
        // レスポンスをJSONとして解析
        let items: Vec<MarketplaceItem> = response.json()
            .await
            .map_err(|e| format!("レスポンスの解析に失敗しました: {}", e))?;
            
        // マーケットプレイスアイテムをThemeInfoに変換
        let themes = items.into_iter()
            .map(|item| self.convert_marketplace_item(item))
            .collect();
            
        Ok(themes)
    }
    
    /// マーケットプレイスアイテムをThemeInfoに変換
    fn convert_marketplace_item(&self, item: MarketplaceItem) -> ThemeInfo {
        // カテゴリを変換
        let categories = item.categories.into_iter()
            .map(|c| match c.as_str() {
                "light" => ThemeCategory::Light,
                "dark" => ThemeCategory::Dark,
                "high_contrast" => ThemeCategory::HighContrast,
                "minimal" => ThemeCategory::Minimal,
                "colorful" => ThemeCategory::Colorful,
                "modern" => ThemeCategory::Modern,
                "classic" => ThemeCategory::Classic,
                _ => ThemeCategory::Custom(c),
            })
            .collect();
            
        // 更新日時をパース
        let updated_at = chrono::DateTime::parse_from_rfc3339(&item.updated_at)
            .map(|dt| dt.into())
            .unwrap_or_else(|_| SystemTime::now());
            
        ThemeInfo {
            name: item.name,
            author: item.author,
            description: item.description,
            version: item.version,
            categories,
            file_path: PathBuf::new(), // ダウンロードまでファイルパスはなし
            thumbnail_path: None, // 画像URLはあるが、ローカルパスはまだない
            downloads: Some(item.downloads),
            rating: Some(item.rating),
            updated_at,
            marketplace_id: Some(item.id),
            metadata: item.metadata,
        }
    }
    
    /// マーケットプレイスからテーマをダウンロード
    pub async fn download_theme(&self, marketplace_id: &str) -> Result<ThemeInfo, String> {
        // テーマの詳細情報を取得
        let url = format!("{}/{}", self.marketplace_url, marketplace_id);
        
        let response = self.http_client.get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("マーケットプレイスへの接続に失敗しました: {}", e))?;
            
        if !response.status().is_success() {
            return Err(format!("マーケットプレイスから不正なレスポンスを受信しました: {}", response.status()));
        }
        
        // レスポンスをJSONとして解析
        let item: MarketplaceItem = response.json()
            .await
            .map_err(|e| format!("レスポンスの解析に失敗しました: {}", e))?;
            
        // テーマファイルをダウンロード
        let content = self.http_client.get(&item.download_url)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("テーマファイルのダウンロードに失敗しました: {}", e))?
            .bytes()
            .await
            .map_err(|e| format!("テーマファイルの読み込みに失敗しました: {}", e))?;
            
        // 保存先パスを決定
        let user_theme_dir = dirs::home_dir()
            .ok_or_else(|| "ホームディレクトリが見つかりません".to_string())?
            .join(".local/share/lumos/themes");
            
        // ディレクトリが存在しない場合は作成
        if !user_theme_dir.exists() {
            fs::create_dir_all(&user_theme_dir)
                .map_err(|e| format!("テーマディレクトリの作成に失敗しました: {}", e))?;
        }
        
        // ファイル名を作成（スペースをアンダースコアに置換）
        let file_name = format!("{}_{}.json", 
            item.name.replace(" ", "_"),
            item.version.replace(".", "_")
        );
        
        let file_path = user_theme_dir.join(file_name);
        
        // ファイルに保存
        fs::write(&file_path, content)
            .map_err(|e| format!("テーマファイルの保存に失敗しました: {}", e))?;
            
        info!("テーマをダウンロードしました: {}", file_path.display());
        
        // サムネイルもダウンロード
        let thumbnail_path = if !item.thumbnail_url.is_empty() {
            let thumbnail_content = self.http_client.get(&item.thumbnail_url)
                .timeout(Duration::from_secs(10))
                .send()
                .await
                .map_err(|e| format!("サムネイルのダウンロードに失敗しました: {}", e))?
                .bytes()
                .await
                .map_err(|e| format!("サムネイルの読み込みに失敗しました: {}", e))?;
                
            let thumbnail_name = file_path.file_stem().unwrap_or_default();
            let thumbnail_path = user_theme_dir.join(format!("{}_thumbnail.png", thumbnail_name.to_string_lossy()));
            
            fs::write(&thumbnail_path, thumbnail_content)
                .map_err(|e| format!("サムネイルの保存に失敗しました: {}", e))?;
                
            Some(thumbnail_path)
        } else {
            None
        };
        
        // ダウンロードしたテーマ情報を作成
        let theme_info = ThemeInfo {
            name: item.name,
            author: item.author,
            description: item.description,
            version: item.version,
            categories: self.convert_marketplace_item(item).categories,
            file_path,
            thumbnail_path,
            downloads: Some(item.downloads),
            rating: Some(item.rating),
            updated_at: SystemTime::now(),
            marketplace_id: Some(marketplace_id.to_string()),
            metadata: HashMap::new(),
        };
        
        // キャッシュに追加
        {
            let mut cache = self.theme_cache.write().unwrap();
            cache.insert(theme_info.name.clone(), theme_info.clone());
        }
        
        Ok(theme_info)
    }
    
    /// テーマをインストール
    pub fn install_theme(&self, theme_info: &ThemeInfo) -> Result<(), String> {
        // テーマファイルを読み込む
        let theme = self.theme_engine.load_theme(&theme_info.file_path)?;
        
        // テーマエンジンにインストール
        self.theme_engine.install_theme(theme);
        
        Ok(())
    }
    
    /// テーマを適用
    pub fn apply_theme(&self, theme_name: &str, effect: &str) -> Result<(), String> {
        // テーマを設定
        self.theme_engine.set_theme_with_blend(theme_name, effect)
    }
    
    /// テーマの評価を送信
    pub async fn rate_theme(&self, marketplace_id: &str, rating: f32) -> Result<(), String> {
        if rating < 0.0 || rating > 5.0 {
            return Err("評価は0から5の間である必要があります".to_string());
        }
        
        // 評価APIのURL
        let url = format!("{}/{}/rate", self.marketplace_url, marketplace_id);
        
        // 評価データ
        let data = serde_json::json!({
            "rating": rating
        });
        
        // POSTリクエストを送信
        let response = self.http_client.post(&url)
            .json(&data)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("評価の送信に失敗しました: {}", e))?;
            
        if !response.status().is_success() {
            return Err(format!("評価の送信に失敗しました: {}", response.status()));
        }
        
        Ok(())
    }
    
    /// テーマを共有
    pub async fn share_theme(&self, theme_info: &ThemeInfo) -> Result<String, String> {
        // テーマが存在するか確認
        if !theme_info.file_path.exists() {
            return Err("テーマファイルが見つかりません".to_string());
        }
        
        // テーマファイルを読み込む
        let theme_content = fs::read(&theme_info.file_path)
            .map_err(|e| format!("テーマファイルの読み込みに失敗しました: {}", e))?;
            
        // サムネイルがあれば読み込む
        let thumbnail_content = if let Some(ref path) = theme_info.thumbnail_path {
            if path.exists() {
                Some(fs::read(path).map_err(|e| format!("サムネイルの読み込みに失敗しました: {}", e))?)
            } else {
                None
            }
        } else {
            None
        };
        
        // マルチパートフォームデータを作成
        let form = reqwest::multipart::Form::new()
            .text("name", theme_info.name.clone())
            .text("author", theme_info.author.clone())
            .text("description", theme_info.description.clone())
            .text("version", theme_info.version.clone())
            .text("categories", serde_json::to_string(&theme_info.categories).unwrap())
            .part("theme_file", reqwest::multipart::Part::bytes(theme_content)
                .file_name(theme_info.file_path.file_name().unwrap_or_default().to_string_lossy().to_string())
                .mime_str("application/json").unwrap());
            
        // サムネイルがあれば追加
        let form = if let Some(content) = thumbnail_content {
            form.part("thumbnail", reqwest::multipart::Part::bytes(content)
                .mime_str("image/png").unwrap())
        } else {
            form
        };
        
        // 共有APIのURL
        let url = format!("{}/share", self.marketplace_url);
        
        // POSTリクエストを送信
        let response = self.http_client.post(&url)
            .multipart(form)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| format!("テーマの共有に失敗しました: {}", e))?;
            
        if !response.status().is_success() {
            return Err(format!("テーマの共有に失敗しました: {}", response.status()));
        }
        
        // レスポンスからIDを抽出
        let data: serde_json::Value = response.json()
            .await
            .map_err(|e| format!("レスポンスの解析に失敗しました: {}", e))?;
            
        let id = data["id"].as_str()
            .ok_or_else(|| "不正なレスポンス形式です".to_string())?
            .to_string();
            
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::tempdir;
    
    #[test]
    fn test_theme_info_loading() {
        let engine = Arc::new(crate::ui::theming::engine::ThemeEngine::new());
        let library = ThemeLibrary::new(engine);
        
        // テスト用ディレクトリを作成
        let temp_dir = tempdir().unwrap();
        let theme_path = temp_dir.path().join("test_theme.json");
        
        // テストテーマを作成
        let theme = crate::ui::theming::engine::Theme {
            name: "TestTheme".to_string(),
            author: Some("TestAuthor".to_string()),
            description: Some("Test Description".to_string()),
            version: Some("1.0.0".to_string()),
            mode: crate::ui::theming::engine::ThemeMode::Dark,
            ..crate::ui::theming::engine::Theme::default()
        };
        
        // テーマをJSONに変換して保存
        let json = serde_json::to_string_pretty(&theme).unwrap();
        fs::write(&theme_path, json).unwrap();
        
        // テーマ情報を読み込み
        let info = library.load_theme_info(&theme_path).unwrap();
        
        // 基本情報が正しいか確認
        assert_eq!(info.name, "TestTheme");
        assert_eq!(info.author, "TestAuthor");
        assert_eq!(info.description, "Test Description");
        assert_eq!(info.version, "1.0.0");
        
        // カテゴリが正しいか確認
        assert!(info.categories.contains(&ThemeCategory::Dark));
    }
    
    #[test]
    fn test_marketplace_item_conversion() {
        let engine = Arc::new(crate::ui::theming::engine::ThemeEngine::new());
        let library = ThemeLibrary::new(engine);
        
        // テスト用マーケットプレイスアイテムを作成
        let item = MarketplaceItem {
            id: "test123".to_string(),
            name: "Marketplace Theme".to_string(),
            author: "Market Author".to_string(),
            description: "A theme from marketplace".to_string(),
            version: "2.0.0".to_string(),
            categories: vec!["dark".to_string(), "minimal".to_string()],
            thumbnail_url: "https://example.com/thumbnail.png".to_string(),
            download_url: "https://example.com/theme.json".to_string(),
            downloads: 1234,
            rating: 4.5,
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            metadata: HashMap::new(),
        };
        
        // マーケットプレイスアイテムをThemeInfoに変換
        let info = library.convert_marketplace_item(item);
        
        // 基本情報が正しいか確認
        assert_eq!(info.name, "Marketplace Theme");
        assert_eq!(info.author, "Market Author");
        assert_eq!(info.description, "A theme from marketplace");
        assert_eq!(info.version, "2.0.0");
        
        // カテゴリが正しいか確認
        assert!(info.categories.contains(&ThemeCategory::Dark));
        assert!(info.categories.contains(&ThemeCategory::Minimal));
        
        // マーケットプレイス固有の情報が正しいか確認
        assert_eq!(info.downloads, Some(1234));
        assert_eq!(info.rating, Some(4.5));
        assert_eq!(info.marketplace_id, Some("test123".to_string()));
    }
} 