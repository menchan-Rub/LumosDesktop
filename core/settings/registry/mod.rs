// LumosDesktop 設定レジストリモジュール
// 設定値を階層的に管理し、永続化する機能を提供

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, json};

use crate::core::settings::SettingsError;

/// 設定レジストリが管理する値の型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingsValue {
    /// 文字列値
    String(String),
    /// 整数値
    Integer(i64),
    /// 浮動小数点値
    Float(f64),
    /// 真偽値
    Boolean(bool),
    /// 配列
    Array(Vec<SettingsValue>),
    /// オブジェクト（ネストした設定）
    Object(HashMap<String, SettingsValue>),
    /// null値
    Null,
}

impl From<JsonValue> for SettingsValue {
    fn from(value: JsonValue) -> Self {
        match value {
            JsonValue::Null => SettingsValue::Null,
            JsonValue::Bool(b) => SettingsValue::Boolean(b),
            JsonValue::Number(n) => {
                if n.is_i64() {
                    SettingsValue::Integer(n.as_i64().unwrap())
                } else {
                    SettingsValue::Float(n.as_f64().unwrap())
                }
            },
            JsonValue::String(s) => SettingsValue::String(s),
            JsonValue::Array(arr) => {
                SettingsValue::Array(arr.into_iter().map(SettingsValue::from).collect())
            },
            JsonValue::Object(obj) => {
                let mut map = HashMap::new();
                for (key, val) in obj {
                    map.insert(key, SettingsValue::from(val));
                }
                SettingsValue::Object(map)
            }
        }
    }
}

impl From<SettingsValue> for JsonValue {
    fn from(value: SettingsValue) -> Self {
        match value {
            SettingsValue::Null => JsonValue::Null,
            SettingsValue::Boolean(b) => JsonValue::Bool(b),
            SettingsValue::Integer(i) => json!(i),
            SettingsValue::Float(f) => json!(f),
            SettingsValue::String(s) => JsonValue::String(s),
            SettingsValue::Array(arr) => {
                JsonValue::Array(arr.into_iter().map(JsonValue::from).collect())
            },
            SettingsValue::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (key, val) in obj {
                    map.insert(key, JsonValue::from(val));
                }
                JsonValue::Object(map)
            }
        }
    }
}

/// 設定キー
pub type SettingsKey = String;

/// 設定パス（ドットで区切られた階層パス）
pub type SettingsPath = String;

/// 設定ノード
#[derive(Debug, Clone)]
pub struct SettingsNode {
    /// ノードの値
    value: SettingsValue,
    /// 変更済みフラグ
    dirty: bool,
}

impl SettingsNode {
    /// 新しい設定ノードを作成
    pub fn new(value: SettingsValue) -> Self {
        Self {
            value,
            dirty: false,
        }
    }
    
    /// ノードの値を取得
    pub fn get_value(&self) -> &SettingsValue {
        &self.value
    }
    
    /// ノードの値を設定
    pub fn set_value(&mut self, value: SettingsValue) {
        self.value = value;
        self.dirty = true;
    }
    
    /// ノードが変更されているか
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    /// ノードをクリーンにする
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
    
    /// ノードを変更済みとしてマークする
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

/// 設定トランザクション
///
/// 複数の設定変更をアトミックに適用するためのトランザクション
pub struct SettingsTransaction {
    /// 変更内容
    changes: HashMap<SettingsPath, SettingsValue>,
    /// 削除内容
    deletions: Vec<SettingsPath>,
}

impl SettingsTransaction {
    /// 新しいトランザクションを作成
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
            deletions: Vec::new(),
        }
    }
    
    /// トランザクションに変更を追加
    pub fn set<T: Serialize>(&mut self, path: &str, value: T) -> Result<&mut Self, SettingsError> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| SettingsError::TypeError(format!("シリアル化エラー: {}", e)))?;
        
        self.changes.insert(path.to_string(), SettingsValue::from(json_value));
        Ok(self)
    }
    
    /// トランザクションに削除を追加
    pub fn delete(&mut self, path: &str) -> &mut Self {
        self.deletions.push(path.to_string());
        self
    }
    
    /// 変更を取得
    pub fn get_changes(&self) -> &HashMap<SettingsPath, SettingsValue> {
        &self.changes
    }
    
    /// 削除を取得
    pub fn get_deletions(&self) -> &Vec<SettingsPath> {
        &self.deletions
    }
    
    /// トランザクションがからかどうか
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty() && self.deletions.is_empty()
    }
    
    /// トランザクションをクリア
    pub fn clear(&mut self) {
        self.changes.clear();
        self.deletions.clear();
    }
}

/// 設定レジストリ
///
/// 階層的な設定値を管理し、永続化する
pub struct SettingsRegistry {
    /// ルートノード
    root: SettingsNode,
    /// 設定ファイルのパス
    settings_file: Option<PathBuf>,
    /// 変更済みフラグ
    dirty: bool,
}

impl SettingsRegistry {
    /// 新しい設定レジストリを作成
    pub fn new() -> Self {
        Self {
            root: SettingsNode::new(SettingsValue::Object(HashMap::new())),
            settings_file: None,
            dirty: false,
        }
    }
    
    /// 設定ファイルを指定して新しい設定レジストリを作成
    pub fn with_file<P: AsRef<Path>>(file_path: P) -> Self {
        Self {
            root: SettingsNode::new(SettingsValue::Object(HashMap::new())),
            settings_file: Some(file_path.as_ref().to_path_buf()),
            dirty: false,
        }
    }
    
    /// 設定レジストリを初期化
    pub fn initialize(&mut self) -> Result<(), SettingsError> {
        // 既存の設定ファイルからロードを試みる
        if let Some(file_path) = &self.settings_file {
            if file_path.exists() {
                self.load(file_path)?;
            }
        }
        
        Ok(())
    }
    
    /// 設定ファイルを設定
    pub fn set_settings_file<P: AsRef<Path>>(&mut self, file_path: P) {
        self.settings_file = Some(file_path.as_ref().to_path_buf());
    }
    
    /// 設定ファイルを取得
    pub fn get_settings_file(&self) -> Option<&Path> {
        self.settings_file.as_deref()
    }
    
    /// 設定が変更されているか
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.root.is_dirty()
    }
    
    /// 設定ファイルから設定をロード
    pub fn load<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), SettingsError> {
        let file_path = file_path.as_ref();
        
        // ファイルが存在しなければエラー
        if !file_path.exists() {
            return Err(SettingsError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("設定ファイルが見つかりません: {}", file_path.display())
            )));
        }
        
        // ファイルを読み込み
        let content = fs::read_to_string(file_path)
            .map_err(|e| SettingsError::Io(e))?;
        
        // JSONとしてパース
        let json_value: JsonValue = serde_json::from_str(&content)
            .map_err(|e| SettingsError::TypeError(format!("JSONパースエラー: {}", e)))?;
        
        // SettingsValueに変換
        let settings_value = SettingsValue::from(json_value);
        
        // ルートノードを更新
        self.root = SettingsNode::new(settings_value);
        self.dirty = false;
        
        Ok(())
    }
    
    /// デフォルト設定をロード
    pub fn load_defaults<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), SettingsError> {
        let file_path = file_path.as_ref();
        
        // ファイルが存在しなければエラー
        if !file_path.exists() {
            return Err(SettingsError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("デフォルト設定ファイルが見つかりません: {}", file_path.display())
            )));
        }
        
        // ファイルを読み込み
        let content = fs::read_to_string(file_path)
            .map_err(|e| SettingsError::Io(e))?;
        
        // JSONとしてパース
        let json_value: JsonValue = serde_json::from_str(&content)
            .map_err(|e| SettingsError::TypeError(format!("JSONパースエラー: {}", e)))?;
        
        // SettingsValueに変換
        let defaults = SettingsValue::from(json_value);
        
        // デフォルト値をマージ（既存の値を上書きしない）
        self.merge_defaults(defaults)?;
        
        Ok(())
    }
    
    /// デフォルト値をマージ（既存の値を上書きしない）
    fn merge_defaults(&mut self, defaults: SettingsValue) -> Result<(), SettingsError> {
        match defaults {
            SettingsValue::Object(default_map) => {
                // 現在のルートがオブジェクトでなければエラー
                if let SettingsValue::Object(ref mut current_map) = self.root.value {
                    // デフォルト値を追加（既存の値は上書きしない）
                    for (key, value) in default_map {
                        if !current_map.contains_key(&key) {
                            current_map.insert(key, value);
                            self.dirty = true;
                        } else if let SettingsValue::Object(_) = value {
                            // オブジェクトの場合は再帰的にマージ
                            if let Some(SettingsValue::Object(_)) = current_map.get(&key) {
                                let path = key.clone();
                                let current_value = self.get_raw(&path)?;
                                let merged = self.merge_objects(current_value, value)?;
                                current_map.insert(key, merged);
                                self.dirty = true;
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err(SettingsError::TypeError("ルートノードがオブジェクトではありません".to_string()))
                }
            },
            _ => Err(SettingsError::TypeError("デフォルト設定はオブジェクトでなければなりません".to_string()))
        }
    }
    
    /// オブジェクトをマージ
    fn merge_objects(&self, current: SettingsValue, default: SettingsValue) -> Result<SettingsValue, SettingsError> {
        match (current, default) {
            (SettingsValue::Object(mut current_map), SettingsValue::Object(default_map)) => {
                for (key, value) in default_map {
                    if !current_map.contains_key(&key) {
                        current_map.insert(key, value);
                    } else if let SettingsValue::Object(_) = value {
                        if let Some(current_value) = current_map.get(&key) {
                            if let SettingsValue::Object(_) = current_value {
                                let merged = self.merge_objects(current_value.clone(), value)?;
                                current_map.insert(key, merged);
                            }
                        }
                    }
                }
                Ok(SettingsValue::Object(current_map))
            },
            _ => Err(SettingsError::TypeError("オブジェクト同士でなければマージできません".to_string()))
        }
    }
    
    /// 設定を保存
    pub fn save(&self) -> Result<(), SettingsError> {
        if let Some(file_path) = &self.settings_file {
            self.save_to(file_path)
        } else {
            Err(SettingsError::Other("設定ファイルが指定されていません".to_string()))
        }
    }
    
    /// 設定を指定したファイルに保存
    pub fn save_to<P: AsRef<Path>>(&self, file_path: P) -> Result<(), SettingsError> {
        let file_path = file_path.as_ref();
        
        // パスのディレクトリが存在しなければ作成
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| SettingsError::Io(e))?;
            }
        }
        
        // SettingsValueをJSONに変換
        let json_value = JsonValue::from(self.root.get_value().clone());
        
        // 整形したJSONとして保存
        let content = serde_json::to_string_pretty(&json_value)
            .map_err(|e| SettingsError::TypeError(format!("JSONシリアル化エラー: {}", e)))?;
        
        fs::write(file_path, content)
            .map_err(|e| SettingsError::Io(e))?;
        
        Ok(())
    }
    
    /// 設定値を取得
    pub fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, SettingsError> {
        // パスからキーの列を取得
        let keys = self.split_path(path);
        
        // ルートノードから始めて、パスをたどる
        let mut current = &self.root.value;
        
        for key in &keys {
            match current {
                SettingsValue::Object(map) => {
                    if let Some(value) = map.get(key) {
                        current = value;
                    } else {
                        return Err(SettingsError::KeyNotFound(path.to_string()));
                    }
                },
                _ => return Err(SettingsError::TypeError(format!("パス'{}'の親がオブジェクトではありません", path))),
            }
        }
        
        // 値をシリアライズして目的の型に変換
        let json_value = JsonValue::from(current.clone());
        serde_json::from_value(json_value)
            .map_err(|e| SettingsError::TypeError(format!("型変換エラー: {}", e)))
    }
    
    /// 設定値を生のSettingsValue型で取得
    pub fn get_raw(&self, path: &str) -> Result<SettingsValue, SettingsError> {
        // パスからキーの列を取得
        let keys = self.split_path(path);
        
        // ルートノードから始めて、パスをたどる
        let mut current = &self.root.value;
        
        for key in &keys {
            match current {
                SettingsValue::Object(map) => {
                    if let Some(value) = map.get(key) {
                        current = value;
                    } else {
                        return Err(SettingsError::KeyNotFound(path.to_string()));
                    }
                },
                _ => return Err(SettingsError::TypeError(format!("パス'{}'の親がオブジェクトではありません", path))),
            }
        }
        
        Ok(current.clone())
    }
    
    /// 設定値を設定
    pub fn set<T: Serialize>(&mut self, path: &str, value: T) -> Result<(), SettingsError> {
        // 値をJSONにシリアライズ
        let json_value = serde_json::to_value(value)
            .map_err(|e| SettingsError::TypeError(format!("シリアル化エラー: {}", e)))?;
        
        // SettingsValueに変換
        let settings_value = SettingsValue::from(json_value);
        
        // パスからキーの列を取得
        let keys = self.split_path(path);
        
        // ルートノードから始めて、パスをたどりながら必要に応じてノードを作成
        let mut current = &mut self.root.value;
        
        for (i, key) in keys.iter().enumerate() {
            if i == keys.len() - 1 {
                // 最後のキーなら値を設定
                match current {
                    SettingsValue::Object(map) => {
                        map.insert(key.clone(), settings_value);
                        self.dirty = true;
                        return Ok(());
                    },
                    _ => return Err(SettingsError::TypeError(format!("パス'{}'の親がオブジェクトではありません", path))),
                }
            } else {
                // 途中のキーならオブジェクトを作成または取得
                match current {
                    SettingsValue::Object(map) => {
                        if !map.contains_key(key) {
                            // キーが存在しなければ空のオブジェクトを作成
                            map.insert(key.clone(), SettingsValue::Object(HashMap::new()));
                            self.dirty = true;
                        }
                        
                        // 次のレベルへ
                        match map.get_mut(key) {
                            Some(ref mut next) => {
                                current = next;
                            },
                            None => return Err(SettingsError::Other("予期せぬエラー".to_string())),
                        }
                    },
                    _ => return Err(SettingsError::TypeError(format!("パス'{}'の親がオブジェクトではありません", path))),
                }
            }
        }
        
        Ok(())
    }
    
    /// 設定値を削除
    pub fn delete(&mut self, path: &str) -> Result<(), SettingsError> {
        // パスからキーの列を取得
        let keys = self.split_path(path);
        
        if keys.is_empty() {
            return Err(SettingsError::KeyNotFound("空のパス".to_string()));
        }
        
        // 最後のキーを取得
        let last_key = keys.last().unwrap();
        
        // 親パスを構築
        let parent_keys = &keys[0..keys.len() - 1];
        
        // ルートノードから始めて、親パスをたどる
        let mut current = &mut self.root.value;
        
        for key in parent_keys {
            match current {
                SettingsValue::Object(map) => {
                    if let Some(next) = map.get_mut(key) {
                        current = next;
                    } else {
                        return Err(SettingsError::KeyNotFound(path.to_string()));
                    }
                },
                _ => return Err(SettingsError::TypeError(format!("パス'{}'の親がオブジェクトではありません", path))),
            }
        }
        
        // 最後のキーを削除
        match current {
            SettingsValue::Object(map) => {
                if map.remove(last_key).is_some() {
                    self.dirty = true;
                    Ok(())
                } else {
                    Err(SettingsError::KeyNotFound(path.to_string()))
                }
            },
            _ => Err(SettingsError::TypeError(format!("パス'{}'の親がオブジェクトではありません", path))),
        }
    }
    
    /// 設定値をリセット（デフォルト値に戻す）
    pub fn reset(&mut self, path: &str) -> Result<(), SettingsError> {
        // 現在はシンプルに削除するだけ
        // より高度な実装では、デフォルト値を保持して元に戻す
        self.delete(path)
    }
    
    /// トランザクションを適用
    pub fn apply_transaction(&mut self, transaction: &SettingsTransaction) -> Result<(), SettingsError> {
        // 変更を適用
        for (path, value) in transaction.get_changes() {
            // パスからキーの列を取得
            let keys = self.split_path(path);
            
            // ルートノードから始めて、パスをたどりながら必要に応じてノードを作成
            let mut current = &mut self.root.value;
            
            for (i, key) in keys.iter().enumerate() {
                if i == keys.len() - 1 {
                    // 最後のキーなら値を設定
                    match current {
                        SettingsValue::Object(map) => {
                            map.insert(key.clone(), value.clone());
                            self.dirty = true;
                        },
                        _ => return Err(SettingsError::TypeError(format!("パス'{}'の親がオブジェクトではありません", path))),
                    }
                } else {
                    // 途中のキーならオブジェクトを作成または取得
                    match current {
                        SettingsValue::Object(map) => {
                            if !map.contains_key(key) {
                                // キーが存在しなければ空のオブジェクトを作成
                                map.insert(key.clone(), SettingsValue::Object(HashMap::new()));
                                self.dirty = true;
                            }
                            
                            // 次のレベルへ
                            match map.get_mut(key) {
                                Some(ref mut next) => {
                                    current = next;
                                },
                                None => return Err(SettingsError::Other("予期せぬエラー".to_string())),
                            }
                        },
                        _ => return Err(SettingsError::TypeError(format!("パス'{}'の親がオブジェクトではありません", path))),
                    }
                }
            }
        }
        
        // 削除を適用
        for path in transaction.get_deletions() {
            // エラーは無視して続行
            let _ = self.delete(path);
        }
        
        Ok(())
    }
    
    /// パスを分割してキーの列に変換
    fn split_path(&self, path: &str) -> Vec<String> {
        path.split('.')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
    
    /// すべての設定キーを取得
    pub fn get_all_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        self.collect_keys(&self.root.value, "", &mut keys);
        keys
    }
    
    /// 再帰的にすべてのキーを収集
    fn collect_keys(&self, value: &SettingsValue, prefix: &str, keys: &mut Vec<String>) {
        match value {
            SettingsValue::Object(map) => {
                for (key, val) in map {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    
                    keys.push(new_prefix.clone());
                    self.collect_keys(val, &new_prefix, keys);
                }
            },
            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_settings_value_conversion() {
        // JSON -> SettingsValue
        let json = json!({
            "string": "value",
            "integer": 42,
            "float": 3.14,
            "boolean": true,
            "array": [1, 2, 3],
            "object": {
                "nested": "value"
            },
            "null": null
        });
        
        let settings_value = SettingsValue::from(json.clone());
        
        // SettingsValue -> JSON
        let back_to_json = JsonValue::from(settings_value);
        
        assert_eq!(json, back_to_json);
    }
    
    #[test]
    fn test_settings_registry_get_set() {
        let mut registry = SettingsRegistry::new();
        
        // 値を設定
        registry.set("app.window.width", 800).unwrap();
        registry.set("app.window.height", 600).unwrap();
        registry.set("app.window.title", "Test Window").unwrap();
        registry.set("app.theme", "dark").unwrap();
        
        // 値を取得
        let width: i32 = registry.get("app.window.width").unwrap();
        let height: i32 = registry.get("app.window.height").unwrap();
        let title: String = registry.get("app.window.title").unwrap();
        let theme: String = registry.get("app.theme").unwrap();
        
        assert_eq!(width, 800);
        assert_eq!(height, 600);
        assert_eq!(title, "Test Window");
        assert_eq!(theme, "dark");
        
        // 存在しないキー
        let result: Result<String, _> = registry.get("app.window.position");
        assert!(result.is_err());
        
        // キーを削除
        registry.delete("app.window.title").unwrap();
        let result: Result<String, _> = registry.get("app.window.title");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_settings_registry_save_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("settings.json");
        
        // 設定を作成して保存
        {
            let mut registry = SettingsRegistry::with_file(&file_path);
            registry.set("app.window.width", 800).unwrap();
            registry.set("app.window.height", 600).unwrap();
            registry.save().unwrap();
        }
        
        // 設定をロード
        {
            let mut registry = SettingsRegistry::new();
            registry.load(&file_path).unwrap();
            
            let width: i32 = registry.get("app.window.width").unwrap();
            let height: i32 = registry.get("app.window.height").unwrap();
            
            assert_eq!(width, 800);
            assert_eq!(height, 600);
        }
    }
    
    #[test]
    fn test_settings_transaction() {
        let mut registry = SettingsRegistry::new();
        
        // 初期値を設定
        registry.set("app.window.width", 800).unwrap();
        registry.set("app.window.height", 600).unwrap();
        
        // トランザクションを作成
        let mut transaction = SettingsTransaction::new();
        transaction.set("app.window.width", 1024).unwrap();
        transaction.set("app.window.title", "New Window").unwrap();
        transaction.delete("app.window.height");
        
        // トランザクションを適用
        registry.apply_transaction(&transaction).unwrap();
        
        // 結果を確認
        let width: i32 = registry.get("app.window.width").unwrap();
        let title: String = registry.get("app.window.title").unwrap();
        let result: Result<i32, _> = registry.get("app.window.height");
        
        assert_eq!(width, 1024);
        assert_eq!(title, "New Window");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_settings_registry_get_all_keys() {
        let mut registry = SettingsRegistry::new();
        
        registry.set("app.window.width", 800).unwrap();
        registry.set("app.window.height", 600).unwrap();
        registry.set("app.theme", "dark").unwrap();
        
        let keys = registry.get_all_keys();
        
        assert!(keys.contains(&"app".to_string()));
        assert!(keys.contains(&"app.window".to_string()));
        assert!(keys.contains(&"app.window.width".to_string()));
        assert!(keys.contains(&"app.window.height".to_string()));
        assert!(keys.contains(&"app.theme".to_string()));
    }
} 