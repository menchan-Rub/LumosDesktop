// AetherOS 1.0 互換性レイヤー
// AetherOS 1.0 APIとの互換性を提供します

use crate::integration::compat::{CompatibilityLayer, CompatError, CompatibleVersion};
use log::{debug, warn, info};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

/// AetherOS 1.0互換性レイヤー
pub struct AetherOS1_0Layer {
    // APIマッピングテーブル - 古いAPI名から新しいAPI名へのマッピング
    api_mappings: HashMap<String, String>,
    // 変換テーブル - 特殊な変換が必要なAPIの処理関数
    transformers: HashMap<String, fn(&[Value]) -> Result<Value, CompatError>>,
    // リソースマッピングテーブル
    resource_mappings: HashMap<String, String>,
    // 統計情報
    call_count: RwLock<HashMap<String, u64>>,
}

impl AetherOS1_0Layer {
    /// 新しいAetherOS 1.0互換性レイヤーを作成します
    pub fn new() -> Self {
        let mut layer = Self {
            api_mappings: HashMap::new(),
            transformers: HashMap::new(),
            resource_mappings: HashMap::new(),
            call_count: RwLock::new(HashMap::new()),
        };
        
        // 初期化
        layer.setup_api_mappings();
        layer.setup_transformers();
        layer.setup_resource_mappings();
        
        layer
    }
    
    /// API名のマッピングを設定
    fn setup_api_mappings(&mut self) {
        // 画面関連
        self.api_mappings.insert(
            "screen.getSize".to_string(), 
            "display.getScreenDimensions".to_string()
        );
        self.api_mappings.insert(
            "screen.getBrightness".to_string(), 
            "display.getBrightness".to_string()
        );
        self.api_mappings.insert(
            "screen.setBrightness".to_string(), 
            "display.setBrightness".to_string()
        );
        
        // ウィンドウ関連
        self.api_mappings.insert(
            "window.create".to_string(), 
            "windowManager.createWindow".to_string()
        );
        self.api_mappings.insert(
            "window.close".to_string(), 
            "windowManager.closeWindow".to_string()
        );
        self.api_mappings.insert(
            "window.minimize".to_string(), 
            "windowManager.minimizeWindow".to_string()
        );
        self.api_mappings.insert(
            "window.maximize".to_string(), 
            "windowManager.maximizeWindow".to_string()
        );
        self.api_mappings.insert(
            "window.restore".to_string(), 
            "windowManager.restoreWindow".to_string()
        );
        
        // ファイルシステム関連
        self.api_mappings.insert(
            "fs.readFile".to_string(), 
            "fileSystem.readFile".to_string()
        );
        self.api_mappings.insert(
            "fs.writeFile".to_string(), 
            "fileSystem.writeFile".to_string()
        );
        self.api_mappings.insert(
            "fs.deleteFile".to_string(), 
            "fileSystem.deleteFile".to_string()
        );
        self.api_mappings.insert(
            "fs.listDirectory".to_string(), 
            "fileSystem.listDirectory".to_string()
        );
        
        // 設定関連
        self.api_mappings.insert(
            "settings.get".to_string(), 
            "settingsManager.getValue".to_string()
        );
        self.api_mappings.insert(
            "settings.set".to_string(), 
            "settingsManager.setValue".to_string()
        );
        
        // システム関連
        self.api_mappings.insert(
            "system.shutdown".to_string(), 
            "powerManager.shutdown".to_string()
        );
        self.api_mappings.insert(
            "system.restart".to_string(), 
            "powerManager.restart".to_string()
        );
        self.api_mappings.insert(
            "system.sleep".to_string(), 
            "powerManager.sleep".to_string()
        );
        
        // ネットワーク関連
        self.api_mappings.insert(
            "net.httpGet".to_string(), 
            "network.httpRequest".to_string()
        );
        self.api_mappings.insert(
            "net.httpPost".to_string(), 
            "network.httpRequest".to_string()
        );
    }
    
    /// 特別な変換が必要なAPIのトランスフォーマーを設定
    fn setup_transformers(&mut self) {
        // HTTP GETリクエストの変換
        self.transformers.insert("net.httpGet".to_string(), |args| {
            if args.is_empty() {
                return Err(CompatError::SystemCallError("URLが指定されていません".to_string()));
            }
            
            let url = match &args[0] {
                Value::String(url_str) => url_str.clone(),
                _ => return Err(CompatError::SystemCallError("URLは文字列である必要があります".to_string())),
            };
            
            // 新しいAPIフォーマットに変換
            Ok(serde_json::json!({
                "method": "GET",
                "url": url,
                "headers": args.get(1).cloned().unwrap_or(Value::Object(serde_json::Map::new())),
                "timeout": 30000,
            }))
        });
        
        // HTTP POSTリクエストの変換
        self.transformers.insert("net.httpPost".to_string(), |args| {
            if args.len() < 2 {
                return Err(CompatError::SystemCallError("URLとボディが必要です".to_string()));
            }
            
            let url = match &args[0] {
                Value::String(url_str) => url_str.clone(),
                _ => return Err(CompatError::SystemCallError("URLは文字列である必要があります".to_string())),
            };
            
            let body = args[1].clone();
            
            // 新しいAPIフォーマットに変換
            Ok(serde_json::json!({
                "method": "POST",
                "url": url,
                "headers": args.get(2).cloned().unwrap_or(Value::Object(serde_json::Map::new())),
                "body": body,
                "timeout": 30000,
            }))
        });
        
        // ウィンドウ作成の変換
        self.transformers.insert("window.create".to_string(), |args| {
            if args.is_empty() {
                return Err(CompatError::SystemCallError("ウィンドウタイトルが指定されていません".to_string()));
            }
            
            let title = match &args[0] {
                Value::String(title_str) => title_str.clone(),
                _ => return Err(CompatError::SystemCallError("タイトルは文字列である必要があります".to_string())),
            };
            
            // デフォルト値
            let width = args.get(1).and_then(|v| v.as_u64()).unwrap_or(800) as u32;
            let height = args.get(2).and_then(|v| v.as_u64()).unwrap_or(600) as u32;
            let resizable = args.get(3).and_then(|v| v.as_bool()).unwrap_or(true);
            
            // 新しいAPIフォーマットに変換
            Ok(serde_json::json!({
                "title": title,
                "width": width,
                "height": height,
                "resizable": resizable,
                "decorations": true,
                "alwaysOnTop": false,
                "transparent": false,
            }))
        });
        
        // 設定取得の変換
        self.transformers.insert("settings.get".to_string(), |args| {
            if args.is_empty() {
                return Err(CompatError::SystemCallError("設定キーが指定されていません".to_string()));
            }
            
            let key = match &args[0] {
                Value::String(key_str) => key_str.clone(),
                _ => return Err(CompatError::SystemCallError("設定キーは文字列である必要があります".to_string())),
            };
            
            // AetherOS 1.0の旧設定キーを新しいフォーマットに変換
            let new_key = if key.starts_with("sys.") {
                key.replacen("sys.", "system.", 1)
            } else if key.starts_with("app.") {
                key.clone()
            } else {
                format!("user.{}", key)
            };
            
            Ok(serde_json::json!({
                "key": new_key,
                "defaultValue": args.get(1).cloned().unwrap_or(Value::Null),
            }))
        });
    }
    
    /// リソースタイプのマッピングを設定
    fn setup_resource_mappings(&mut self) {
        self.resource_mappings.insert(
            "image/icon".to_string(), 
            "image/app-icon".to_string()
        );
        self.resource_mappings.insert(
            "audio/effect".to_string(), 
            "audio/sound-effect".to_string()
        );
        self.resource_mappings.insert(
            "font/standard".to_string(), 
            "font/system".to_string()
        );
    }
    
    /// 古いAPIコール名を新しいものに変換
    fn translate_api_name(&self, name: &str) -> String {
        self.api_mappings.get(name).cloned().unwrap_or_else(|| name.to_string())
    }
    
    /// 古いリソースタイプを新しいものに変換
    fn translate_resource_type(&self, resource_type: &str) -> String {
        self.resource_mappings.get(resource_type).cloned().unwrap_or_else(|| resource_type.to_string())
    }
    
    /// APIコール回数をインクリメント
    fn increment_call_count(&self, name: &str) {
        let mut call_count = self.call_count.write().unwrap();
        *call_count.entry(name.to_string()).or_insert(0) += 1;
    }
}

impl CompatibilityLayer for AetherOS1_0Layer {
    fn version(&self) -> CompatibleVersion {
        CompatibleVersion::AetherOS1_0
    }
    
    fn translate_api_call(&self, name: &str, args: &[Value]) -> Result<Value, CompatError> {
        debug!("AetherOS1_0Layer: APIコール変換: {} (引数: {}個)", name, args.len());
        
        // 統計情報を更新
        self.increment_call_count(name);
        
        // 特別な変換が必要なAPIの場合
        if let Some(transformer) = self.transformers.get(name) {
            let result = transformer(args);
            match &result {
                Ok(_) => debug!("AetherOS1_0Layer: 特殊変換成功: {}", name),
                Err(e) => warn!("AetherOS1_0Layer: 特殊変換失敗: {} - エラー: {}", name, e),
            }
            return result;
        }
        
        // 標準的なAPIマッピング
        let new_name = self.translate_api_name(name);
        if new_name != name {
            debug!("AetherOS1_0Layer: API名変換: {} -> {}", name, new_name);
        }
        
        // 引数はそのまま渡す（必要に応じてここで引数の変換も行う）
        Ok(serde_json::json!({
            "api": new_name,
            "args": args,
        }))
    }
    
    fn translate_resource(&self, resource_type: &str, data: &[u8]) -> Result<Vec<u8>, CompatError> {
        let new_type = self.translate_resource_type(resource_type);
        debug!("AetherOS1_0Layer: リソース変換: {} -> {}, サイズ: {}バイト", 
               resource_type, new_type, data.len());
        
        // AetherOS 1.0のほとんどのリソースは変換なしで使用可能
        // 特殊な変換が必要な場合はここに追加
        
        // 現在はデータをそのまま返す
        Ok(data.to_vec())
    }
    
    fn translate_event(&self, event_name: &str, event_data: &Value) -> Result<Value, CompatError> {
        debug!("AetherOS1_0Layer: イベント変換: {}", event_name);
        
        // イベント名のマッピング
        let new_event_name = match event_name {
            "window.resize" => "windowResized",
            "window.close" => "windowClosed",
            "app.exit" => "applicationExit",
            "system.lowMemory" => "systemMemoryWarning",
            _ => event_name,
        };
        
        if new_event_name != event_name {
            debug!("AetherOS1_0Layer: イベント名変換: {} -> {}", event_name, new_event_name);
        }
        
        // イベントデータは基本的にそのまま渡す
        // 必要に応じてここでイベントデータの変換を行う
        Ok(serde_json::json!({
            "name": new_event_name,
            "data": event_data,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }))
    }
    
    fn initialize(&mut self) -> Result<(), CompatError> {
        info!("AetherOS1_0Layer: 初期化");
        // 特別な初期化が必要な場合はここに実装
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), CompatError> {
        info!("AetherOS1_0Layer: クリーンアップ");
        // リソース解放などが必要な場合はここに実装
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_translation() {
        let layer = AetherOS1_0Layer::new();
        
        // 基本的なAPI名の変換テスト
        assert_eq!(
            layer.translate_api_name("screen.getSize"),
            "display.getScreenDimensions"
        );
        assert_eq!(
            layer.translate_api_name("fs.readFile"),
            "fileSystem.readFile"
        );
        
        // マッピングにないAPI名はそのまま
        assert_eq!(
            layer.translate_api_name("unknown.api"),
            "unknown.api"
        );
    }
    
    #[test]
    fn test_http_get_transformation() {
        let layer = AetherOS1_0Layer::new();
        
        // HTTP GETリクエストの変換テスト
        let args = vec![
            serde_json::json!("https://example.com/api"),
            serde_json::json!({"Authorization": "Bearer token123"}),
        ];
        
        let result = layer.translate_api_call("net.httpGet", &args).unwrap();
        
        // 結果の検証
        assert_eq!(result["method"], "GET");
        assert_eq!(result["url"], "https://example.com/api");
        assert_eq!(result["headers"]["Authorization"], "Bearer token123");
        assert_eq!(result["timeout"], 30000);
    }
    
    #[test]
    fn test_window_create_transformation() {
        let layer = AetherOS1_0Layer::new();
        
        // ウィンドウ作成の変換テスト
        let args = vec![
            serde_json::json!("Test Window"),
            serde_json::json!(1024),
            serde_json::json!(768),
            serde_json::json!(false),
        ];
        
        let result = layer.translate_api_call("window.create", &args).unwrap();
        
        // 結果の検証
        assert_eq!(result["title"], "Test Window");
        assert_eq!(result["width"], 1024);
        assert_eq!(result["height"], 768);
        assert_eq!(result["resizable"], false);
        assert_eq!(result["decorations"], true);
    }
} 