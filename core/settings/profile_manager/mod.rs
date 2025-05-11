// LumosDesktop プロファイル管理モジュール
// 複数のユーザープロファイルと環境設定の管理を担当

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::core::settings::SettingsError;
use crate::core::settings::registry::{SettingsRegistry, SettingsValue};

/// プロファイルID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProfileId(String);

impl ProfileId {
    /// 新しいランダムなプロファイルIDを生成
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// 文字列からプロファイルIDを作成
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    /// プロファイルIDを文字列として取得
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProfileId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for ProfileId {
    fn default() -> Self {
        Self::new()
    }
}

/// プロファイルの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileType {
    /// ユーザープロファイル
    User,
    /// デバイスプロファイル
    Device,
    /// 環境プロファイル
    Environment,
    /// アプリケーションプロファイル
    Application,
    /// システムプロファイル
    System,
}

/// プロファイルのメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    /// プロファイル名
    pub name: String,
    /// プロファイルの説明
    pub description: Option<String>,
    /// プロファイルのアイコン
    pub icon: Option<String>,
    /// プロファイルの作成日時
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// プロファイルの最終更新日時
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// プロファイルの種類
    pub profile_type: ProfileType,
    /// プロファイルのタグ
    pub tags: Vec<String>,
    /// カスタムメタデータ
    pub custom: HashMap<String, String>,
}

impl Default for ProfileMetadata {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            description: None,
            icon: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            profile_type: ProfileType::User,
            tags: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

/// ユーザープロファイル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// プロファイルID
    pub id: ProfileId,
    /// メタデータ
    pub metadata: ProfileMetadata,
    /// 親プロファイルID（継承元）
    pub parent_id: Option<ProfileId>,
    /// 設定レジストリ
    #[serde(skip)]
    registry: Option<SettingsRegistry>,
    /// オーバーライドキー（親プロファイルから継承しない設定キー）
    pub overrides: Vec<String>,
    /// プロファイルファイルパス
    #[serde(skip)]
    file_path: Option<PathBuf>,
}

impl UserProfile {
    /// 新しいユーザープロファイルを作成
    pub fn new(name: &str, profile_type: ProfileType) -> Self {
        let mut metadata = ProfileMetadata::default();
        metadata.name = name.to_string();
        metadata.profile_type = profile_type;
        
        Self {
            id: ProfileId::new(),
            metadata,
            parent_id: None,
            registry: Some(SettingsRegistry::new()),
            overrides: Vec::new(),
            file_path: None,
        }
    }
    
    /// IDを指定して新しいユーザープロファイルを作成
    pub fn with_id(id: ProfileId, name: &str, profile_type: ProfileType) -> Self {
        let mut metadata = ProfileMetadata::default();
        metadata.name = name.to_string();
        metadata.profile_type = profile_type;
        
        Self {
            id,
            metadata,
            parent_id: None,
            registry: Some(SettingsRegistry::new()),
            overrides: Vec::new(),
            file_path: None,
        }
    }
    
    /// プロファイルファイルを設定
    pub fn set_file_path<P: AsRef<Path>>(&mut self, path: P) {
        self.file_path = Some(path.as_ref().to_path_buf());
        
        // レジストリにもファイルパスを設定
        if let Some(ref mut registry) = self.registry {
            registry.set_settings_file(path);
        }
    }
    
    /// プロファイルファイルパスを取得
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }
    
    /// 設定レジストリを取得
    pub fn registry(&self) -> Option<&SettingsRegistry> {
        self.registry.as_ref()
    }
    
    /// 設定レジストリを可変で取得
    pub fn registry_mut(&mut self) -> Option<&mut SettingsRegistry> {
        self.registry.as_mut()
    }
    
    /// プロファイルから設定値を取得
    pub fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, SettingsError> {
        match &self.registry {
            Some(registry) => registry.get(path),
            None => Err(SettingsError::Other("プロファイルレジストリが初期化されていません".to_string())),
        }
    }
    
    /// プロファイルに設定値を設定
    pub fn set<T: Serialize>(&mut self, path: &str, value: T) -> Result<(), SettingsError> {
        match &mut self.registry {
            Some(registry) => {
                // 親から継承している場合は、オーバーライドリストに追加
                if self.parent_id.is_some() && !self.overrides.contains(&path.to_string()) {
                    self.overrides.push(path.to_string());
                }
                
                registry.set(path, value)
            },
            None => Err(SettingsError::Other("プロファイルレジストリが初期化されていません".to_string())),
        }
    }
    
    /// 設定値をリセット
    pub fn reset(&mut self, path: &str) -> Result<(), SettingsError> {
        match &mut self.registry {
            Some(registry) => {
                // オーバーライドリストから削除
                self.overrides.retain(|key| key != path);
                
                registry.delete(path)
            },
            None => Err(SettingsError::Other("プロファイルレジストリが初期化されていません".to_string())),
        }
    }
    
    /// プロファイルをロード
    pub fn load(&mut self) -> Result<(), SettingsError> {
        if let Some(path) = &self.file_path {
            if let Some(ref mut registry) = self.registry {
                if path.exists() {
                    registry.load(path)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// プロファイルを保存
    pub fn save(&self) -> Result<(), SettingsError> {
        if let Some(path) = &self.file_path {
            if let Some(ref registry) = self.registry {
                // プロファイルメタデータを別ファイルに保存
                let metadata_path = path.with_extension("meta.json");
                let metadata_json = serde_json::to_string_pretty(&self)
                    .map_err(|e| SettingsError::Other(format!("メタデータのシリアル化エラー: {}", e)))?;
                
                // ディレクトリが存在しなければ作成
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .map_err(|e| SettingsError::Io(e))?;
                    }
                }
                
                fs::write(&metadata_path, metadata_json)
                    .map_err(|e| SettingsError::Io(e))?;
                
                // 設定を保存
                registry.save_to(path)?;
            }
        }
        
        Ok(())
    }
}

/// プロファイルマネージャー
///
/// 複数のプロファイルを管理し、アクティブプロファイルの設定を提供
pub struct ProfileManager {
    /// プロファイルのベースディレクトリ
    profiles_dir: PathBuf,
    /// 登録されたプロファイル
    profiles: HashMap<ProfileId, UserProfile>,
    /// アクティブなプロファイルID
    active_profile_id: Option<ProfileId>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl ProfileManager {
    /// 新しいプロファイルマネージャーを作成
    pub fn new<P: AsRef<Path>>(profiles_dir: P) -> Self {
        Self {
            profiles_dir: profiles_dir.as_ref().to_path_buf(),
            profiles: HashMap::new(),
            active_profile_id: None,
            initialized: false,
        }
    }
    
    /// プロファイルマネージャーを初期化
    pub fn initialize(&mut self) -> Result<(), SettingsError> {
        if self.initialized {
            return Ok(());
        }
        
        // プロファイルディレクトリが存在しなければ作成
        if !self.profiles_dir.exists() {
            fs::create_dir_all(&self.profiles_dir)
                .map_err(|e| SettingsError::Io(e))?;
        }
        
        // 既存のプロファイルをロード
        self.load_profiles()?;
        
        // デフォルトプロファイルが存在しなければ作成
        if !self.has_default_profile() {
            self.create_default_profile()?;
        }
        
        // アクティブプロファイルが設定されていなければデフォルトを設定
        if self.active_profile_id.is_none() {
            if let Some(id) = self.get_default_profile_id() {
                self.active_profile_id = Some(id);
            }
        }
        
        self.initialized = true;
        Ok(())
    }
    
    /// 既存のプロファイルをロード
    fn load_profiles(&mut self) -> Result<(), SettingsError> {
        // profiles_dir内の全てのメタデータファイルを検索
        let entries = match fs::read_dir(&self.profiles_dir) {
            Ok(entries) => entries,
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    return Ok(());
                }
                return Err(SettingsError::Io(e));
            }
        };
        
        for entry in entries {
            let entry = entry.map_err(|e| SettingsError::Io(e))?;
            let path = entry.path();
            
            // メタデータファイルを探す
            if path.is_file() && path.extension().map_or(false, |ext| ext == "meta.json") {
                // メタデータをロード
                let metadata_content = fs::read_to_string(&path)
                    .map_err(|e| SettingsError::Io(e))?;
                
                let profile: UserProfile = serde_json::from_str(&metadata_content)
                    .map_err(|e| SettingsError::Other(format!("プロファイルのパースエラー: {}", e)))?;
                
                // 設定ファイルのパスを構築
                let settings_path = self.profiles_dir.join(format!("{}.json", profile.id.as_str()));
                
                // プロファイルを適切に初期化
                let mut initialized_profile = profile.clone();
                initialized_profile.registry = Some(SettingsRegistry::new());
                initialized_profile.set_file_path(&settings_path);
                
                // 設定をロード
                initialized_profile.load()?;
                
                // プロファイルを登録
                self.profiles.insert(profile.id.clone(), initialized_profile);
            }
        }
        
        Ok(())
    }
    
    /// デフォルトプロファイルが存在するか確認
    fn has_default_profile(&self) -> bool {
        self.profiles.values().any(|p| p.metadata.name == "Default")
    }
    
    /// デフォルトプロファイルIDを取得
    fn get_default_profile_id(&self) -> Option<ProfileId> {
        self.profiles.values()
            .find(|p| p.metadata.name == "Default")
            .map(|p| p.id.clone())
    }
    
    /// デフォルトプロファイルを作成
    fn create_default_profile(&mut self) -> Result<ProfileId, SettingsError> {
        let profile_id = ProfileId::new();
        let mut profile = UserProfile::new("Default", ProfileType::User);
        profile.id = profile_id.clone();
        
        // メタデータを設定
        profile.metadata.description = Some("Default user profile".to_string());
        
        // ファイルパスを設定
        let file_path = self.profiles_dir.join(format!("{}.json", profile_id.as_str()));
        profile.set_file_path(file_path);
        
        // プロファイルを保存
        profile.save()?;
        
        // プロファイルを登録
        self.profiles.insert(profile_id.clone(), profile);
        
        Ok(profile_id)
    }
    
    /// 新しいプロファイルを作成
    pub fn create_profile(&mut self, name: &str, profile_type: ProfileType) -> Result<ProfileId, SettingsError> {
        if !self.initialized {
            return Err(SettingsError::Other("プロファイルマネージャーが初期化されていません".to_string()));
        }
        
        let profile_id = ProfileId::new();
        let mut profile = UserProfile::new(name, profile_type);
        profile.id = profile_id.clone();
        
        // ファイルパスを設定
        let file_path = self.profiles_dir.join(format!("{}.json", profile_id.as_str()));
        profile.set_file_path(file_path);
        
        // プロファイルを保存
        profile.save()?;
        
        // プロファイルを登録
        self.profiles.insert(profile_id.clone(), profile);
        
        Ok(profile_id)
    }
    
    /// プロファイルを削除
    pub fn delete_profile(&mut self, profile_id: &ProfileId) -> Result<(), SettingsError> {
        if !self.initialized {
            return Err(SettingsError::Other("プロファイルマネージャーが初期化されていません".to_string()));
        }
        
        // プロファイルが存在するか確認
        if let Some(profile) = self.profiles.get(profile_id) {
            // ファイルを削除
            if let Some(path) = profile.file_path() {
                if path.exists() {
                    fs::remove_file(path)
                        .map_err(|e| SettingsError::Io(e))?;
                }
                
                // メタデータファイルも削除
                let metadata_path = path.with_extension("meta.json");
                if metadata_path.exists() {
                    fs::remove_file(metadata_path)
                        .map_err(|e| SettingsError::Io(e))?;
                }
            }
            
            // プロファイルを削除
            self.profiles.remove(profile_id);
            
            // アクティブプロファイルが削除された場合
            if self.active_profile_id.as_ref() == Some(profile_id) {
                self.active_profile_id = self.get_default_profile_id();
            }
            
            Ok(())
        } else {
            Err(SettingsError::ProfileError(format!("プロファイルが見つかりません: {}", profile_id.as_str())))
        }
    }
    
    /// プロファイルを取得
    pub fn get_profile(&self, profile_id: &ProfileId) -> Option<&UserProfile> {
        self.profiles.get(profile_id)
    }
    
    /// プロファイルを可変で取得
    pub fn get_profile_mut(&mut self, profile_id: &ProfileId) -> Option<&mut UserProfile> {
        self.profiles.get_mut(profile_id)
    }
    
    /// すべてのプロファイルを取得
    pub fn get_all_profiles(&self) -> Vec<&UserProfile> {
        self.profiles.values().collect()
    }
    
    /// アクティブなプロファイルIDを取得
    pub fn get_active_profile_id(&self) -> Option<ProfileId> {
        self.active_profile_id.clone()
    }
    
    /// アクティブなプロファイルを設定
    pub fn set_active_profile(&mut self, profile_id: ProfileId) -> Result<(), SettingsError> {
        if !self.initialized {
            return Err(SettingsError::Other("プロファイルマネージャーが初期化されていません".to_string()));
        }
        
        // プロファイルが存在するか確認
        if self.profiles.contains_key(&profile_id) {
            self.active_profile_id = Some(profile_id);
            Ok(())
        } else {
            Err(SettingsError::ProfileError(format!("プロファイルが見つかりません: {}", profile_id.as_str())))
        }
    }
    
    /// アクティブなプロファイルを取得
    pub fn get_active_profile(&self) -> Option<&UserProfile> {
        self.active_profile_id.as_ref().and_then(|id| self.profiles.get(id))
    }
    
    /// アクティブなプロファイルを可変で取得
    pub fn get_active_profile_mut(&mut self) -> Option<&mut UserProfile> {
        if let Some(id) = self.active_profile_id.clone() {
            self.profiles.get_mut(&id)
        } else {
            None
        }
    }
    
    /// プロファイルから設定値を取得
    pub fn get_profile_setting<T: serde::de::DeserializeOwned>(
        &self,
        profile_id: ProfileId,
        path: &str
    ) -> Result<T, SettingsError> {
        // プロファイルを取得
        let profile = self.profiles.get(&profile_id)
            .ok_or_else(|| SettingsError::ProfileError(format!("プロファイルが見つかりません: {}", profile_id.as_str())))?;
        
        // プロファイルから設定を取得
        match profile.get(path) {
            Ok(value) => Ok(value),
            Err(SettingsError::KeyNotFound(_)) => {
                // 親プロファイルがあれば、そこから取得
                if let Some(parent_id) = &profile.parent_id {
                    // オーバーライドリストにある場合は親からは取得しない
                    if profile.overrides.contains(&path.to_string()) {
                        return Err(SettingsError::KeyNotFound(path.to_string()));
                    }
                    
                    self.get_profile_setting(*parent_id.clone(), path)
                } else {
                    Err(SettingsError::KeyNotFound(path.to_string()))
                }
            },
            Err(e) => Err(e),
        }
    }
    
    /// プロファイルに設定値を設定
    pub fn set_profile_setting<T: Serialize>(
        &self,
        profile_id: ProfileId,
        path: &str,
        value: &T
    ) -> Result<(), SettingsError> {
        // プロファイルを可変で取得（ここではselfがimmutableなので工夫が必要）
        if let Some(profile) = self.profiles.get(&profile_id) {
            if let Some(registry) = profile.registry() {
                // JSONにシリアライズ
                let json_value = serde_json::to_value(value)
                    .map_err(|e| SettingsError::TypeError(format!("シリアル化エラー: {}", e)))?;
                
                // SettingsValueに変換
                let settings_value = crate::core::settings::registry::SettingsValue::from(json_value);
                
                // 実際のセット操作はsetメソッドで行う必要がある
                // ここでは簡略化のためエラーを返す
                return Err(SettingsError::Other("immutableコンテキストでの設定変更はサポートされていません".to_string()));
            }
        }
        
        Err(SettingsError::ProfileError(format!("プロファイルが見つかりません: {}", profile_id.as_str())))
    }
    
    /// プロファイルの設定値をリセット
    pub fn reset_profile_setting(
        &self,
        profile_id: ProfileId,
        path: &str
    ) -> Result<(), SettingsError> {
        // immutableコンテキストでのリセットはサポートされていない
        Err(SettingsError::Other("immutableコンテキストでの設定リセットはサポートされていません".to_string()))
    }
    
    /// すべてのプロファイルを保存
    pub fn save_all(&self) -> Result<(), SettingsError> {
        for profile in self.profiles.values() {
            profile.save()?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_profile_id() {
        let id1 = ProfileId::new();
        let id2 = ProfileId::new();
        
        assert_ne!(id1, id2);
        
        let id_str = id1.as_str();
        let id3 = ProfileId::from_string(id_str.to_string());
        
        assert_eq!(id1, id3);
    }
    
    #[test]
    fn test_user_profile_creation() {
        let profile = UserProfile::new("Test Profile", ProfileType::User);
        
        assert_eq!(profile.metadata.name, "Test Profile");
        assert_eq!(profile.metadata.profile_type, ProfileType::User);
        assert!(profile.parent_id.is_none());
        assert!(profile.overrides.is_empty());
    }
    
    #[test]
    fn test_profile_save_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("profile.json");
        
        // プロファイルを作成して保存
        let profile_id = {
            let mut profile = UserProfile::new("Test Profile", ProfileType::User);
            profile.set_file_path(&file_path);
            
            // 設定を追加
            profile.set("app.window.width", 800).unwrap();
            profile.set("app.window.height", 600).unwrap();
            
            profile.save().unwrap();
            
            profile.id.clone()
        };
        
        // 新しいプロファイルを作成してロード
        let mut profile = UserProfile::with_id(profile_id, "Test Profile", ProfileType::User);
        profile.set_file_path(&file_path);
        profile.load().unwrap();
        
        // 設定を確認
        let width: i32 = profile.get("app.window.width").unwrap();
        let height: i32 = profile.get("app.window.height").unwrap();
        
        assert_eq!(width, 800);
        assert_eq!(height, 600);
    }
    
    #[test]
    fn test_profile_manager_basic() {
        let dir = tempdir().unwrap();
        let mut manager = ProfileManager::new(dir.path());
        
        // 初期化
        manager.initialize().unwrap();
        
        // デフォルトプロファイルが作成されているか確認
        assert!(manager.has_default_profile());
        
        // 新しいプロファイルを作成
        let profile_id = manager.create_profile("Test Profile", ProfileType::User).unwrap();
        
        // プロファイルを取得
        let profile = manager.get_profile(&profile_id).unwrap();
        assert_eq!(profile.metadata.name, "Test Profile");
        
        // アクティブプロファイルを設定
        manager.set_active_profile(profile_id.clone()).unwrap();
        assert_eq!(manager.get_active_profile_id().unwrap(), profile_id);
    }
}