// LumosDesktop 設定スキーマモジュール
// 設定の型定義と検証を担当

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, json};
use regex::Regex;

use crate::core::settings::SettingsError;
use crate::core::settings::registry::SettingsValue;

/// スキーマタイプ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    /// 文字列型
    String,
    /// 整数型
    Integer,
    /// 浮動小数点型
    Number,
    /// 真偽値型
    Boolean,
    /// 配列型
    Array,
    /// オブジェクト型
    Object,
    /// 列挙型
    Enum,
    /// 任意の型
    Any,
}

/// スキーマ制約
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConstraint {
    /// 型
    #[serde(rename = "type")]
    pub type_name: SchemaType,
    /// 説明
    pub description: Option<String>,
    /// デフォルト値
    pub default: Option<JsonValue>,
    /// 必須
    #[serde(default)]
    pub required: bool,
    /// 最小値（数値型）
    pub minimum: Option<f64>,
    /// 最大値（数値型）
    pub maximum: Option<f64>,
    /// 最小長（文字列型）
    pub min_length: Option<usize>,
    /// 最大長（文字列型）
    pub max_length: Option<usize>,
    /// パターン（文字列型）
    pub pattern: Option<String>,
    /// 列挙値（enum型）
    pub enum_values: Option<Vec<JsonValue>>,
    /// 配列アイテムの型（配列型）
    pub items: Option<Box<SchemaConstraint>>,
    /// プロパティ（オブジェクト型）
    pub properties: Option<HashMap<String, SchemaConstraint>>,
    /// 追加プロパティを許可（オブジェクト型）
    #[serde(default = "default_true")]
    pub additional_properties: bool,
    /// 読み取り専用
    #[serde(default)]
    pub read_only: bool,
    /// フォーマット（文字列型、例: date, email, uri等）
    pub format: Option<String>,
}

fn default_true() -> bool {
    true
}

impl SchemaConstraint {
    /// 新しいスキーマ制約を作成
    pub fn new(type_name: SchemaType) -> Self {
        Self {
            type_name,
            description: None,
            default: None,
            required: false,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
            items: None,
            properties: None,
            additional_properties: true,
            read_only: false,
            format: None,
        }
    }
    
    /// 文字列型の制約を作成
    pub fn string() -> Self {
        Self::new(SchemaType::String)
    }
    
    /// 整数型の制約を作成
    pub fn integer() -> Self {
        Self::new(SchemaType::Integer)
    }
    
    /// 浮動小数点型の制約を作成
    pub fn number() -> Self {
        Self::new(SchemaType::Number)
    }
    
    /// 真偽値型の制約を作成
    pub fn boolean() -> Self {
        Self::new(SchemaType::Boolean)
    }
    
    /// 配列型の制約を作成
    pub fn array(items: SchemaConstraint) -> Self {
        let mut schema = Self::new(SchemaType::Array);
        schema.items = Some(Box::new(items));
        schema
    }
    
    /// オブジェクト型の制約を作成
    pub fn object() -> Self {
        let mut schema = Self::new(SchemaType::Object);
        schema.properties = Some(HashMap::new());
        schema
    }
    
    /// 列挙型の制約を作成
    pub fn enum_type(values: Vec<JsonValue>) -> Self {
        let mut schema = Self::new(SchemaType::Enum);
        schema.enum_values = Some(values);
        schema
    }
    
    /// 任意の型の制約を作成
    pub fn any() -> Self {
        Self::new(SchemaType::Any)
    }
    
    /// 説明を設定
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// デフォルト値を設定
    pub fn default<T: Serialize>(mut self, value: T) -> Self {
        self.default = Some(json!(value));
        self
    }
    
    /// 必須フラグを設定
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    
    /// 最小値を設定（数値型）
    pub fn minimum(mut self, minimum: f64) -> Self {
        self.minimum = Some(minimum);
        self
    }
    
    /// 最大値を設定（数値型）
    pub fn maximum(mut self, maximum: f64) -> Self {
        self.maximum = Some(maximum);
        self
    }
    
    /// 最小長を設定（文字列型）
    pub fn min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }
    
    /// 最大長を設定（文字列型）
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }
    
    /// パターンを設定（文字列型）
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }
    
    /// 読み取り専用フラグを設定
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    
    /// フォーマットを設定（文字列型）
    pub fn format(mut self, format: &str) -> Self {
        self.format = Some(format.to_string());
        self
    }
    
    /// プロパティを追加（オブジェクト型）
    pub fn property(mut self, name: &str, constraint: SchemaConstraint) -> Self {
        if self.type_name == SchemaType::Object {
            let properties = self.properties.get_or_insert_with(HashMap::new);
            properties.insert(name.to_string(), constraint);
        }
        self
    }
}

/// 検証結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// 検証結果
    pub is_valid: bool,
    /// エラーメッセージ
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// 成功した検証結果を作成
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }
    
    /// 失敗した検証結果を作成
    pub fn failure(error: String) -> Self {
        Self {
            is_valid: false,
            errors: vec![error],
        }
    }
    
    /// 失敗した検証結果を複数のエラーから作成
    pub fn failures(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }
    
    /// エラーを追加
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }
    
    /// 結果をマージ
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }
}

/// 設定スキーマ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSchema {
    /// スキーマ名
    pub name: String,
    /// バージョン
    pub version: String,
    /// 説明
    pub description: Option<String>,
    /// ルート制約
    pub root: SchemaConstraint,
    /// 設定パス別の制約
    pub properties: HashMap<String, SchemaConstraint>,
}

impl SettingsSchema {
    /// 新しい設定スキーマを作成
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: None,
            root: SchemaConstraint::object(),
            properties: HashMap::new(),
        }
    }
    
    /// 説明を設定
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// 制約を追加
    pub fn constraint(mut self, path: &str, constraint: SchemaConstraint) -> Self {
        self.properties.insert(path.to_string(), constraint);
        self
    }
    
    /// 値を検証
    pub fn validate<T: Serialize>(&self, path: &str, value: &T) -> ValidationResult {
        // 値をJSONにシリアライズ
        let json_value = match serde_json::to_value(value) {
            Ok(value) => value,
            Err(e) => return ValidationResult::failure(format!("シリアル化エラー: {}", e)),
        };
        
        // パスに対応する制約を取得
        if let Some(constraint) = self.properties.get(path) {
            self.validate_value_against_constraint(&json_value, constraint, path)
        } else {
            // 制約がなければ、ルート制約で検証
            self.validate_value_against_constraint(&json_value, &self.root, path)
        }
    }
    
    /// 制約に対して値を検証
    fn validate_value_against_constraint(&self, value: &JsonValue, constraint: &SchemaConstraint, path: &str) -> ValidationResult {
        match constraint.type_name {
            SchemaType::String => self.validate_string(value, constraint, path),
            SchemaType::Integer => self.validate_integer(value, constraint, path),
            SchemaType::Number => self.validate_number(value, constraint, path),
            SchemaType::Boolean => self.validate_boolean(value, constraint, path),
            SchemaType::Array => self.validate_array(value, constraint, path),
            SchemaType::Object => self.validate_object(value, constraint, path),
            SchemaType::Enum => self.validate_enum(value, constraint, path),
            SchemaType::Any => ValidationResult::success(),
        }
    }
    
    /// 文字列型の値を検証
    fn validate_string(&self, value: &JsonValue, constraint: &SchemaConstraint, path: &str) -> ValidationResult {
        if !value.is_string() {
            return ValidationResult::failure(format!("'{}': 文字列型が必要ですが、{}が与えられました", path, value));
        }
        
        let string_value = value.as_str().unwrap();
        let mut result = ValidationResult::success();
        
        // 最小長の検証
        if let Some(min_length) = constraint.min_length {
            if string_value.len() < min_length {
                result.add_error(format!("'{}': 文字列の長さが最小値{}未満です", path, min_length));
            }
        }
        
        // 最大長の検証
        if let Some(max_length) = constraint.max_length {
            if string_value.len() > max_length {
                result.add_error(format!("'{}': 文字列の長さが最大値{}を超えています", path, max_length));
            }
        }
        
        // パターンの検証
        if let Some(pattern) = &constraint.pattern {
            match Regex::new(pattern) {
                Ok(regex) => {
                    if !regex.is_match(string_value) {
                        result.add_error(format!("'{}': 文字列がパターン'{}'に一致しません", path, pattern));
                    }
                },
                Err(e) => {
                    result.add_error(format!("'{}': 無効な正規表現パターン: {}", path, e));
                }
            }
        }
        
        // フォーマットの検証
        if let Some(format) = &constraint.format {
            match format.as_str() {
                "email" => {
                    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
                    if !email_regex.is_match(string_value) {
                        result.add_error(format!("'{}': 有効なメールアドレス形式ではありません", path));
                    }
                },
                "uri" => {
                    if url::Url::parse(string_value).is_err() {
                        result.add_error(format!("'{}': 有効なURI形式ではありません", path));
                    }
                },
                "date" => {
                    let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
                    if !date_regex.is_match(string_value) {
                        result.add_error(format!("'{}': 有効な日付形式（YYYY-MM-DD）ではありません", path));
                    }
                },
                "date-time" => {
                    let datetime_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$").unwrap();
                    if !datetime_regex.is_match(string_value) {
                        result.add_error(format!("'{}': 有効な日時形式（ISO 8601）ではありません", path));
                    }
                },
                _ => {
                    result.add_error(format!("'{}': 未知のフォーマット'{}'です", path, format));
                }
            }
        }
        
        result
    }
    
    /// 整数型の値を検証
    fn validate_integer(&self, value: &JsonValue, constraint: &SchemaConstraint, path: &str) -> ValidationResult {
        if !value.is_i64() && !value.is_u64() {
            return ValidationResult::failure(format!("'{}': 整数型が必要ですが、{}が与えられました", path, value));
        }
        
        let mut result = ValidationResult::success();
        
        // 数値として共通の検証
        if value.is_number() {
            let number_value = value.as_f64().unwrap();
            
            // 最小値の検証
            if let Some(minimum) = constraint.minimum {
                if number_value < minimum {
                    result.add_error(format!("'{}': 値{}が最小値{}未満です", path, number_value, minimum));
                }
            }
            
            // 最大値の検証
            if let Some(maximum) = constraint.maximum {
                if number_value > maximum {
                    result.add_error(format!("'{}': 値{}が最大値{}を超えています", path, number_value, maximum));
                }
            }
        }
        
        result
    }
    
    /// 数値型の値を検証
    fn validate_number(&self, value: &JsonValue, constraint: &SchemaConstraint, path: &str) -> ValidationResult {
        if !value.is_number() {
            return ValidationResult::failure(format!("'{}': 数値型が必要ですが、{}が与えられました", path, value));
        }
        
        let number_value = value.as_f64().unwrap();
        let mut result = ValidationResult::success();
        
        // 最小値の検証
        if let Some(minimum) = constraint.minimum {
            if number_value < minimum {
                result.add_error(format!("'{}': 値{}が最小値{}未満です", path, number_value, minimum));
            }
        }
        
        // 最大値の検証
        if let Some(maximum) = constraint.maximum {
            if number_value > maximum {
                result.add_error(format!("'{}': 値{}が最大値{}を超えています", path, number_value, maximum));
            }
        }
        
        result
    }
    
    /// 真偽値型の値を検証
    fn validate_boolean(&self, value: &JsonValue, constraint: &SchemaConstraint, path: &str) -> ValidationResult {
        if !value.is_boolean() {
            return ValidationResult::failure(format!("'{}': 真偽値型が必要ですが、{}が与えられました", path, value));
        }
        
        ValidationResult::success()
    }
    
    /// 配列型の値を検証
    fn validate_array(&self, value: &JsonValue, constraint: &SchemaConstraint, path: &str) -> ValidationResult {
        if !value.is_array() {
            return ValidationResult::failure(format!("'{}': 配列型が必要ですが、{}が与えられました", path, value));
        }
        
        let array_value = value.as_array().unwrap();
        let mut result = ValidationResult::success();
        
        // 配列アイテムの検証
        if let Some(items) = &constraint.items {
            for (i, item) in array_value.iter().enumerate() {
                let item_path = format!("{}[{}]", path, i);
                let item_result = self.validate_value_against_constraint(item, items, &item_path);
                result.merge(item_result);
            }
        }
        
        result
    }
    
    /// オブジェクト型の値を検証
    fn validate_object(&self, value: &JsonValue, constraint: &SchemaConstraint, path: &str) -> ValidationResult {
        if !value.is_object() {
            return ValidationResult::failure(format!("'{}': オブジェクト型が必要ですが、{}が与えられました", path, value));
        }
        
        let object_value = value.as_object().unwrap();
        let mut result = ValidationResult::success();
        
        // プロパティの検証
        if let Some(properties) = &constraint.properties {
            // 必須プロパティの検証
            for (prop_name, prop_constraint) in properties {
                if prop_constraint.required && !object_value.contains_key(prop_name) {
                    result.add_error(format!("'{}': 必須プロパティ'{}'がありません", path, prop_name));
                }
                
                // プロパティ値の検証
                if let Some(prop_value) = object_value.get(prop_name) {
                    let prop_path = if path.is_empty() {
                        prop_name.clone()
                    } else {
                        format!("{}.{}", path, prop_name)
                    };
                    
                    let prop_result = self.validate_value_against_constraint(prop_value, prop_constraint, &prop_path);
                    result.merge(prop_result);
                }
            }
            
            // 追加プロパティの検証
            if !constraint.additional_properties {
                for prop_name in object_value.keys() {
                    if !properties.contains_key(prop_name) {
                        result.add_error(format!("'{}': 追加プロパティ'{}'は許可されていません", path, prop_name));
                    }
                }
            }
        }
        
        result
    }
    
    /// 列挙型の値を検証
    fn validate_enum(&self, value: &JsonValue, constraint: &SchemaConstraint, path: &str) -> ValidationResult {
        if let Some(enum_values) = &constraint.enum_values {
            if !enum_values.contains(value) {
                return ValidationResult::failure(format!(
                    "'{}': 値{}は許可された値{:?}の一つでなければなりません",
                    path, value, enum_values
                ));
            }
        } else {
            return ValidationResult::failure(format!("'{}': 列挙値が定義されていません", path));
        }
        
        ValidationResult::success()
    }
}

/// スキーママネージャー
///
/// 設定スキーマを管理し、設定値を検証
pub struct SchemaManager {
    /// スキーマディレクトリ
    schema_dir: PathBuf,
    /// 登録されたスキーマ
    schemas: HashMap<String, SettingsSchema>,
    /// パスとスキーマのマッピング
    path_schemas: HashMap<String, String>,
    /// 初期化済みフラグ
    initialized: bool,
}

impl SchemaManager {
    /// 新しいスキーママネージャーを作成
    pub fn new<P: AsRef<Path>>(schema_dir: P) -> Self {
        Self {
            schema_dir: schema_dir.as_ref().to_path_buf(),
            schemas: HashMap::new(),
            path_schemas: HashMap::new(),
            initialized: false,
        }
    }
    
    /// スキーママネージャーを初期化
    pub fn initialize(&mut self) -> Result<(), SettingsError> {
        if self.initialized {
            return Ok(());
        }
        
        // スキーマディレクトリが存在しなければ作成
        if !self.schema_dir.exists() {
            fs::create_dir_all(&self.schema_dir)
                .map_err(|e| SettingsError::Io(e))?;
        }
        
        // 既存のスキーマをロード
        self.load_schemas()?;
        
        self.initialized = true;
        Ok(())
    }
    
    /// 既存のスキーマをロード
    fn load_schemas(&mut self) -> Result<(), SettingsError> {
        // schema_dir内の全てのJSONファイルを検索
        let entries = match fs::read_dir(&self.schema_dir) {
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
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                // スキーマファイルをロード
                let content = fs::read_to_string(&path)
                    .map_err(|e| SettingsError::Io(e))?;
                
                let schema: SettingsSchema = serde_json::from_str(&content)
                    .map_err(|e| SettingsError::SchemaError(format!("スキーマのパースエラー: {}", e)))?;
                
                // スキーマを登録
                let schema_name = schema.name.clone();
                
                // パスとスキーマのマッピングを追加
                for path in schema.properties.keys() {
                    self.path_schemas.insert(path.clone(), schema_name.clone());
                }
                
                self.schemas.insert(schema_name, schema);
            }
        }
        
        Ok(())
    }
    
    /// スキーマを登録
    pub fn register_schema(&mut self, schema: SettingsSchema) -> Result<(), SettingsError> {
        if !self.initialized {
            return Err(SettingsError::Other("スキーママネージャーが初期化されていません".to_string()));
        }
        
        let schema_name = schema.name.clone();
        
        // パスとスキーマのマッピングを追加
        for path in schema.properties.keys() {
            self.path_schemas.insert(path.clone(), schema_name.clone());
        }
        
        // スキーマを保存
        let file_path = self.schema_dir.join(format!("{}.json", schema_name));
        let schema_json = serde_json::to_string_pretty(&schema)
            .map_err(|e| SettingsError::SchemaError(format!("スキーマのシリアル化エラー: {}", e)))?;
        
        fs::write(&file_path, schema_json)
            .map_err(|e| SettingsError::Io(e))?;
        
        // スキーマを登録
        self.schemas.insert(schema_name, schema);
        
        Ok(())
    }
    
    /// スキーマを取得
    pub fn get_schema(&self, name: &str) -> Option<&SettingsSchema> {
        self.schemas.get(name)
    }
    
    /// パスに関連付けられたスキーマを取得
    pub fn get_schema_for_path(&self, path: &str) -> Option<&SettingsSchema> {
        // 完全一致するパスを検索
        if let Some(schema_name) = self.path_schemas.get(path) {
            return self.schemas.get(schema_name);
        }
        
        // 親パスを探索
        let parts: Vec<&str> = path.split('.').collect();
        
        for i in (0..parts.len()).rev() {
            let parent_path = parts[0..i].join(".");
            if let Some(schema_name) = self.path_schemas.get(&parent_path) {
                return self.schemas.get(schema_name);
            }
        }
        
        None
    }
    
    /// 値を検証
    pub fn validate_value<T: Serialize>(&self, path: &str, value: &T) -> Result<(), SettingsError> {
        if !self.initialized {
            return Err(SettingsError::Other("スキーママネージャーが初期化されていません".to_string()));
        }
        
        // パスに関連付けられたスキーマを取得
        if let Some(schema) = self.get_schema_for_path(path) {
            let result = schema.validate(path, value);
            
            if !result.is_valid {
                return Err(SettingsError::ValidationError(result.errors.join(", ")));
            }
        }
        
        Ok(())
    }
    
    /// すべてのスキーマを取得
    pub fn get_all_schemas(&self) -> Vec<&SettingsSchema> {
        self.schemas.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_schema_validation_string() {
        let constraint = SchemaConstraint::string()
            .min_length(3)
            .max_length(10)
            .pattern(r"^[a-z]+$");
        
        let schema = SettingsSchema::new("test", "1.0")
            .constraint("test.string", constraint);
        
        // 有効な値
        let result = schema.validate("test.string", &"abcdef");
        assert!(result.is_valid);
        
        // 無効な値 (短すぎる)
        let result = schema.validate("test.string", &"ab");
        assert!(!result.is_valid);
        
        // 無効な値 (長すぎる)
        let result = schema.validate("test.string", &"abcdefghijk");
        assert!(!result.is_valid);
        
        // 無効な値 (パターンに一致しない)
        let result = schema.validate("test.string", &"Abcdef");
        assert!(!result.is_valid);
        
        // 型が異なる
        let result = schema.validate("test.string", &123);
        assert!(!result.is_valid);
    }
    
    #[test]
    fn test_schema_validation_number() {
        let constraint = SchemaConstraint::number()
            .minimum(0.0)
            .maximum(100.0);
        
        let schema = SettingsSchema::new("test", "1.0")
            .constraint("test.number", constraint);
        
        // 有効な値
        let result = schema.validate("test.number", &50.0);
        assert!(result.is_valid);
        
        // 境界値
        let result = schema.validate("test.number", &0.0);
        assert!(result.is_valid);
        
        let result = schema.validate("test.number", &100.0);
        assert!(result.is_valid);
        
        // 無効な値 (小さすぎる)
        let result = schema.validate("test.number", &-1.0);
        assert!(!result.is_valid);
        
        // 無効な値 (大きすぎる)
        let result = schema.validate("test.number", &101.0);
        assert!(!result.is_valid);
        
        // 型が異なる
        let result = schema.validate("test.number", &"string");
        assert!(!result.is_valid);
    }
    
    #[test]
    fn test_schema_validation_object() {
        let string_constraint = SchemaConstraint::string().min_length(1);
        let number_constraint = SchemaConstraint::number().minimum(0.0);
        
        let object_constraint = SchemaConstraint::object()
            .property("name", string_constraint)
            .property("age", number_constraint);
        
        let schema = SettingsSchema::new("test", "1.0")
            .constraint("test.object", object_constraint);
        
        // 有効な値
        let value = json!({
            "name": "John",
            "age": 30
        });
        let result = schema.validate("test.object", &value);
        assert!(result.is_valid);
        
        // 一部の値が無効
        let value = json!({
            "name": "",
            "age": 30
        });
        let result = schema.validate("test.object", &value);
        assert!(!result.is_valid);
        
        let value = json!({
            "name": "John",
            "age": -1
        });
        let result = schema.validate("test.object", &value);
        assert!(!result.is_valid);
        
        // 型が異なる
        let result = schema.validate("test.object", &"string");
        assert!(!result.is_valid);
    }
    
    #[test]
    fn test_schema_validation_array() {
        let item_constraint = SchemaConstraint::string().min_length(1);
        let array_constraint = SchemaConstraint::array(item_constraint);
        
        let schema = SettingsSchema::new("test", "1.0")
            .constraint("test.array", array_constraint);
        
        // 有効な値
        let value = json!(["a", "bc", "def"]);
        let result = schema.validate("test.array", &value);
        assert!(result.is_valid);
        
        // 一部の値が無効
        let value = json!(["a", "", "def"]);
        let result = schema.validate("test.array", &value);
        assert!(!result.is_valid);
        
        // 型が異なる
        let result = schema.validate("test.array", &"string");
        assert!(!result.is_valid);
    }
    
    #[test]
    fn test_schema_validation_enum() {
        let enum_constraint = SchemaConstraint::enum_type(vec![
            json!("red"),
            json!("green"),
            json!("blue")
        ]);
        
        let schema = SettingsSchema::new("test", "1.0")
            .constraint("test.enum", enum_constraint);
        
        // 有効な値
        let result = schema.validate("test.enum", &"red");
        assert!(result.is_valid);
        
        // 無効な値
        let result = schema.validate("test.enum", &"yellow");
        assert!(!result.is_valid);
        
        // 型が異なる
        let result = schema.validate("test.enum", &123);
        assert!(!result.is_valid);
    }
} 