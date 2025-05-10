// AetherOS 1.5 互換性レイヤー
// AetherOS 1.5 APIとの互換性を提供します

use crate::integration::compat::{CompatibilityLayer, CompatError, CompatibleVersion};
use log::{debug, warn, info};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

/// AetherOS 1.5互換性レイヤー
pub struct AetherOS1_5Layer {
    // APIマッピングテーブル - 古いAPI名から新しいAPI名へのマッピング
    api_mappings: HashMap<String, String>,
    // 変換テーブル - 特殊な変換が必要なAPIの処理関数
    transformers: HashMap<String, fn(&[Value]) -> Result<Value, CompatError>>,
    // リソースマッピングテーブル
    resource_mappings: HashMap<String, String>,
    // 統計情報
    call_count: RwLock<HashMap<String, u64>>,
}

impl AetherOS1_5Layer {
    /// 新しいAetherOS 1.5互換性レイヤーを作成します
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
            "display.getResolution".to_string(), 
            "display.getScreenDimensions".to_string()
        );
        self.api_mappings.insert(
            "display.getBrightness".to_string(), 
            "display.getBrightness".to_string()
        );
        self.api_mappings.insert(
            "display.setBrightness".to_string(), 
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
        self.api_mappings.insert(
            "window.setPosition".to_string(), 
            "windowManager.setWindowPosition".to_string()
        );
        self.api_mappings.insert(
            "window.setSize".to_string(), 
            "windowManager.setWindowSize".to_string()
        );
        
        // ファイルシステム関連
        self.api_mappings.insert(
            "fileSystem.readFile".to_string(), 
            "fileSystem.readFile".to_string()
        );
        self.api_mappings.insert(
            "fileSystem.writeFile".to_string(), 
            "fileSystem.writeFile".to_string()
        );
        self.api_mappings.insert(
            "fileSystem.deleteFile".to_string(), 
            "fileSystem.deleteFile".to_string()
        );
        self.api_mappings.insert(
            "fileSystem.listDirectory".to_string(), 
            "fileSystem.listDirectory".to_string()
        );
        self.api_mappings.insert(
            "fileSystem.createDirectory".to_string(), 
            "fileSystem.createDirectory".to_string()
        );
        self.api_mappings.insert(
            "fileSystem.deleteDirectory".to_string(), 
            "fileSystem.deleteDirectory".to_string()
        );
        
        // 設定関連
        self.api_mappings.insert(
            "settings.getValue".to_string(), 
            "settingsManager.getValue".to_string()
        );
        self.api_mappings.insert(
            "settings.setValue".to_string(), 
            "settingsManager.setValue".to_string()
        );
        self.api_mappings.insert(
            "settings.deleteValue".to_string(), 
            "settingsManager.deleteValue".to_string()
        );
        
        // システム関連
        self.api_mappings.insert(
            "system.powerOff".to_string(), 
            "powerManager.shutdown".to_string()
        );
        self.api_mappings.insert(
            "system.reboot".to_string(), 
            "powerManager.restart".to_string()
        );
        self.api_mappings.insert(
            "system.sleep".to_string(), 
            "powerManager.sleep".to_string()
        );
        self.api_mappings.insert(
            "system.getMemoryInfo".to_string(), 
            "systemInfo.getMemoryDetails".to_string()
        );
        self.api_mappings.insert(
            "system.getCpuInfo".to_string(), 
            "systemInfo.getCpuDetails".to_string()
        );
        
        // ネットワーク関連
        self.api_mappings.insert(
            "network.httpRequest".to_string(), 
            "network.httpRequest".to_string()
        );
        self.api_mappings.insert(
            "network.webSocket.connect".to_string(), 
            "network.createWebSocketConnection".to_string()
        );
    }
    
    /// 特別な変換が必要なAPIのトランスフォーマーを設定
    fn setup_transformers(&mut self) {
        // HTTP リクエストの変換
        self.transformers.insert("network.httpRequest".to_string(), |args| {
            if args.is_empty() {
                return Err(CompatError::SystemCallError("リクエスト設定が指定されていません".to_string()));
            }
            
            let config = &args[0];
            
            // 新しいAPIフォーマットに変換 (1.5と3.0では類似しているがタイムアウト値などが異なる)
            let mut new_config = config.clone();
            
            // タイムアウトのデフォルト値が異なる場合の調整
            if let Some(obj) = new_config.as_object_mut() {
                if !obj.contains_key("timeout") {
                    obj.insert("timeout".to_string(), serde_json::json!(30000));
                }
                
                // 1.5では"followRedirects"だったが3.0では"followRedirect"に変更された場合
                if let Some(follow_redirects) = obj.remove("followRedirects") {
                    obj.insert("followRedirect".to_string(), follow_redirects);
                }
            }
            
            Ok(new_config)
        });
        
        // WebSocket接続の変換
        self.transformers.insert("network.webSocket.connect".to_string(), |args| {
            if args.is_empty() {
                return Err(CompatError::SystemCallError("WebSocket URLが指定されていません".to_string()));
            }
            
            let url = match &args[0] {
                Value::String(url_str) => url_str.clone(),
                _ => return Err(CompatError::SystemCallError("URLは文字列である必要があります".to_string())),
            };
            
            // プロトコルと追加ヘッダー
            let protocols = args.get(1).cloned().unwrap_or(Value::Array(vec![]));
            let headers = args.get(2).cloned().unwrap_or(Value::Object(serde_json::Map::new()));
            
            // 新しいAPIフォーマットに変換
            Ok(serde_json::json!({
                "url": url,
                "protocols": protocols,
                "headers": headers,
                "reconnect": true,
                "reconnectInterval": 3000,
                "maxReconnectAttempts": 5,
            }))
        });
        
        // ウィンドウ作成の変換
        self.transformers.insert("window.create".to_string(), |args| {
            if args.is_empty() {
                return Err(CompatError::SystemCallError("ウィンドウ設定が指定されていません".to_string()));
            }
            
            let config = &args[0];
            
            // デフォルト値とプロパティ名の調整
            let mut new_config = config.clone();
            
            if let Some(obj) = new_config.as_object_mut() {
                // タイトルのデフォルト値設定
                if !obj.contains_key("title") {
                    obj.insert("title".to_string(), serde_json::json!("AetherOS Application"));
                }
                
                // 1.5では"frame"だったが3.0では"decorations"に変更された場合
                if let Some(frame) = obj.remove("frame") {
                    obj.insert("decorations".to_string(), frame);
                }
                
                // その他の互換性調整
                if !obj.contains_key("alwaysOnTop") {
                    obj.insert("alwaysOnTop".to_string(), serde_json::json!(false));
                }
            }
            
            Ok(new_config)
        });
        
        // メモリ情報取得の変換
        self.transformers.insert("system.getMemoryInfo".to_string(), |_args| {
            // 新しいAPIフォーマットでは追加情報が必要
            Ok(serde_json::json!({
                "includeSwap": true,
                "detailed": true
            }))
        });
    }
    
    /// リソースタイプのマッピングを設定
    fn setup_resource_mappings(&mut self) {
        self.resource_mappings.insert(
            "image/app-icon".to_string(), 
            "image/app-icon".to_string()
        );
        self.resource_mappings.insert(
            "audio/sound-effect".to_string(), 
            "audio/sound-effect".to_string()
        );
        self.resource_mappings.insert(
            "font/user".to_string(), 
            "font/custom".to_string()
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

impl CompatibilityLayer for AetherOS1_5Layer {
    fn version(&self) -> CompatibleVersion {
        CompatibleVersion::AetherOS1_5
    }
    
    fn translate_api_call(&self, name: &str, args: &[Value]) -> Result<Value, CompatError> {
        debug!("AetherOS1_5Layer: APIコール変換: {} (引数: {}個)", name, args.len());
        
        // 統計情報を更新
        self.increment_call_count(name);
        
        // 特別な変換が必要なAPIの場合
        if let Some(transformer) = self.transformers.get(name) {
            let result = transformer(args);
            match &result {
                Ok(_) => debug!("AetherOS1_5Layer: 特殊変換成功: {}", name),
                Err(e) => warn!("AetherOS1_5Layer: 特殊変換失敗: {} - エラー: {}", name, e),
            }
            return result;
        }
        
        // 標準的なAPIマッピング
        let new_name = self.translate_api_name(name);
        if new_name != name {
            debug!("AetherOS1_5Layer: API名変換: {} -> {}", name, new_name);
        }
        
        // 引数はそのまま渡す（必要に応じてここで引数の変換も行う）
        Ok(serde_json::json!({
            "api": new_name,
            "args": args,
        }))
    }
    
    fn translate_resource(&self, resource_type: &str, data: &[u8]) -> Result<Vec<u8>, CompatError> {
        let new_type = self.translate_resource_type(resource_type);
        debug!("AetherOS1_5Layer: リソース変換: {} -> {}, サイズ: {}バイト", 
               resource_type, new_type, data.len());
        
        // AetherOS 1.5のほとんどのリソースは変換なしで使用可能
        // 特殊な変換が必要な場合はここに追加
        
        // 現在はデータをそのまま返す
        Ok(data.to_vec())
    }
    
    fn translate_event(&self, event_name: &str, event_data: &Value) -> Result<Value, CompatError> {
        debug!("AetherOS1_5Layer: イベント変換: {}", event_name);
        
        // イベント名のマッピング
        let new_event_name = match event_name {
            "window.resize" => "windowResized",
            "window.close" => "windowClosed",
            "window.focus" => "windowFocused",
            "window.blur" => "windowBlurred",
            "app.exit" => "applicationExit",
            "system.memoryWarning" => "systemMemoryWarning",
            "system.batteryLow" => "systemBatteryLow",
            _ => event_name,
        };
        
        if new_event_name != event_name {
            debug!("AetherOS1_5Layer: イベント名変換: {} -> {}", event_name, new_event_name);
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
        info!("AetherOS1_5Layer: 初期化");
        // 特別な初期化が必要な場合はここに実装
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), CompatError> {
        info!("AetherOS1_5Layer: クリーンアップ");
        // リソース解放などが必要な場合はここに実装
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_translation() {
        let layer = AetherOS1_5Layer::new();
        
        // 基本的なAPI名の変換テスト
        assert_eq!(
            layer.translate_api_name("display.getResolution"),
            "display.getScreenDimensions"
        );
        assert_eq!(
            layer.translate_api_name("system.powerOff"),
            "powerManager.shutdown"
        );
        
        // マッピングにないAPI名はそのまま
        assert_eq!(
            layer.translate_api_name("unknown.api"),
            "unknown.api"
        );
    }
    
    #[test]
    fn test_http_request_transformation() {
        let layer = AetherOS1_5Layer::new();
        
        // HTTPリクエストの変換テスト
        let args = vec![
            serde_json::json!({
                "method": "GET",
                "url": "https://example.com/api",
                "headers": {"Authorization": "Bearer token123"},
                "followRedirects": true
            }),
        ];
        
        let result = layer.translate_api_call("network.httpRequest", &args).unwrap();
        
        // 結果の検証
        assert_eq!(result["url"], "https://example.com/api");
        assert_eq!(result["headers"]["Authorization"], "Bearer token123");
        assert_eq!(result["followRedirect"], true); // 名前が変更されていることを確認
        assert_eq!(result["timeout"], 30000);
    }
    
    #[test]
    fn test_window_create_transformation() {
        let layer = AetherOS1_5Layer::new();
        
        // ウィンドウ作成の変換テスト
        let args = vec![
            serde_json::json!({
                "width": 1024,
                "height": 768,
                "frame": false
            }),
        ];
        
        let result = layer.translate_api_call("window.create", &args).unwrap();
        
        // 結果の検証
        assert_eq!(result["title"], "AetherOS Application"); // デフォルト値が設定されている
        assert_eq!(result["width"], 1024);
        assert_eq!(result["height"], 768);
        assert_eq!(result["decorations"], false); // "frame"が"decorations"に変換されている
        assert_eq!(result["alwaysOnTop"], false); // デフォルト値が追加されている
    }
} 