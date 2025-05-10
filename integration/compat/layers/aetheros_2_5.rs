// AetherOS 2.5 互換性レイヤー
// AetherOS 2.5 APIとの互換性を提供します

use crate::integration::compat::{CompatibilityLayer, CompatError, CompatibleVersion};
use log::{debug, warn, info};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

/// AetherOS 2.5互換性レイヤー
pub struct AetherOS2_5Layer {
    // APIマッピングテーブル - 古いAPI名から新しいAPI名へのマッピング
    api_mappings: HashMap<String, String>,
    // 変換テーブル - 特殊な変換が必要なAPIの処理関数
    transformers: HashMap<String, fn(&[Value]) -> Result<Value, CompatError>>,
    // リソースマッピングテーブル
    resource_mappings: HashMap<String, String>,
    // 統計情報
    call_count: RwLock<HashMap<String, u64>>,
}

impl AetherOS2_5Layer {
    /// 新しいAetherOS 2.5互換性レイヤーを作成します
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
        // AetherOS 2.5と3.0ではAPIが非常に近いため、マッピングは最小限
        
        // 名前変更されたAPIのみマッピング
        self.api_mappings.insert(
            "windowManager.createStandardWindow".to_string(), 
            "windowManager.createWindow".to_string()
        );
        
        self.api_mappings.insert(
            "display.getDisplayProperties".to_string(), 
            "display.getScreenProperties".to_string()
        );
        
        self.api_mappings.insert(
            "network.websocket.create".to_string(), 
            "network.createWebSocketConnection".to_string()
        );
        
        self.api_mappings.insert(
            "systemInfo.getCpuUsage".to_string(), 
            "systemInfo.getCpuUtilization".to_string()
        );
        
        self.api_mappings.insert(
            "systemInfo.getGpuUsage".to_string(), 
            "systemInfo.getGpuUtilization".to_string()
        );
    }
    
    /// 特別な変換が必要なAPIのトランスフォーマーを設定
    fn setup_transformers(&mut self) {
        // ウィンドウ作成の変換
        self.transformers.insert("windowManager.createStandardWindow".to_string(), |args| {
            if args.is_empty() {
                return Err(CompatError::SystemCallError("ウィンドウ設定が指定されていません".to_string()));
            }
            
            let config = &args[0];
            
            // ほぼ同じだが、いくつかのプロパティ名が変更されている可能性がある
            let mut new_config = config.clone();
            
            if let Some(obj) = new_config.as_object_mut() {
                // プロパティ名の変更があれば対応
                if let Some(use_native_decorations) = obj.remove("useNativeDecorations") {
                    obj.insert("decorations".to_string(), use_native_decorations);
                }
                
                // 3.0で追加されたプロパティがなければデフォルト値を設定
                if !obj.contains_key("theme") {
                    obj.insert("theme".to_string(), serde_json::json!("system"));
                }
                
                if !obj.contains_key("minWidth") && obj.contains_key("width") {
                    if let Some(width) = obj.get("width") {
                        obj.insert("minWidth".to_string(), width.clone());
                    }
                }
                
                if !obj.contains_key("minHeight") && obj.contains_key("height") {
                    if let Some(height) = obj.get("height") {
                        obj.insert("minHeight".to_string(), height.clone());
                    }
                }
            }
            
            Ok(new_config)
        });
        
        // ディスプレイプロパティ取得の変換
        self.transformers.insert("display.getDisplayProperties".to_string(), |args| {
            let display_id = args.get(0).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            
            // 新しいAPIフォーマットに変換
            Ok(serde_json::json!({
                "displayId": display_id,
                "includeHdrCapabilities": true,
                "includeColorSpace": true
            }))
        });
        
        // WebSocket作成の変換
        self.transformers.insert("network.websocket.create".to_string(), |args| {
            if args.is_empty() {
                return Err(CompatError::SystemCallError("WebSocket設定が指定されていません".to_string()));
            }
            
            let config = &args[0];
            
            // プロパティ名の変更とデフォルト値の追加
            let mut new_config = config.clone();
            
            if let Some(obj) = new_config.as_object_mut() {
                // プロパティ名の変更があれば対応
                if let Some(auto_reconnect) = obj.remove("autoReconnect") {
                    obj.insert("reconnect".to_string(), auto_reconnect);
                }
                
                if let Some(max_reconnect_attempts) = obj.remove("maxRetries") {
                    obj.insert("maxReconnectAttempts".to_string(), max_reconnect_attempts);
                }
                
                // 3.0で追加されたプロパティがなければデフォルト値を設定
                if !obj.contains_key("reconnectInterval") {
                    obj.insert("reconnectInterval".to_string(), serde_json::json!(3000));
                }
                
                if !obj.contains_key("binaryType") {
                    obj.insert("binaryType".to_string(), serde_json::json!("arraybuffer"));
                }
            }
            
            Ok(new_config)
        });
    }
    
    /// リソースタイプのマッピングを設定
    fn setup_resource_mappings(&mut self) {
        // AetherOS 2.5と3.0ではリソースタイプは同じものがほとんど
        self.resource_mappings.insert(
            "audio/system-sound".to_string(), 
            "audio/system-notification".to_string()
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

impl CompatibilityLayer for AetherOS2_5Layer {
    fn version(&self) -> CompatibleVersion {
        CompatibleVersion::AetherOS2_5
    }
    
    fn translate_api_call(&self, name: &str, args: &[Value]) -> Result<Value, CompatError> {
        debug!("AetherOS2_5Layer: APIコール変換: {} (引数: {}個)", name, args.len());
        
        // 統計情報を更新
        self.increment_call_count(name);
        
        // 特別な変換が必要なAPIの場合
        if let Some(transformer) = self.transformers.get(name) {
            let result = transformer(args);
            match &result {
                Ok(_) => debug!("AetherOS2_5Layer: 特殊変換成功: {}", name),
                Err(e) => warn!("AetherOS2_5Layer: 特殊変換失敗: {} - エラー: {}", name, e),
            }
            return result;
        }
        
        // 標準的なAPIマッピング
        let new_name = self.translate_api_name(name);
        if new_name != name {
            debug!("AetherOS2_5Layer: API名変換: {} -> {}", name, new_name);
        }
        
        // 引数はそのまま渡す（必要に応じてここで引数の変換も行う）
        Ok(serde_json::json!({
            "api": new_name,
            "args": args,
        }))
    }
    
    fn translate_resource(&self, resource_type: &str, data: &[u8]) -> Result<Vec<u8>, CompatError> {
        let new_type = self.translate_resource_type(resource_type);
        debug!("AetherOS2_5Layer: リソース変換: {} -> {}, サイズ: {}バイト", 
               resource_type, new_type, data.len());
        
        // AetherOS 2.5のリソースは変換なしで使用可能
        // 特殊な変換が必要な場合はここに追加
        
        // 現在はデータをそのまま返す
        Ok(data.to_vec())
    }
    
    fn translate_event(&self, event_name: &str, event_data: &Value) -> Result<Value, CompatError> {
        debug!("AetherOS2_5Layer: イベント変換: {}", event_name);
        
        // イベント名のマッピング (AetherOS 2.5と3.0ではイベント名はほぼ同じ)
        let new_event_name = match event_name {
            "window.themeChanged" => "windowThemeChanged",
            "system.powerModeChanged" => "systemPowerModeChanged",
            _ => event_name,
        };
        
        if new_event_name != event_name {
            debug!("AetherOS2_5Layer: イベント名変換: {} -> {}", event_name, new_event_name);
        }
        
        // イベントデータは基本的にそのまま渡す
        // 必要に応じてここでイベントデータの変換を行う
        Ok(serde_json::json!({
            "name": new_event_name,
            "data": event_data,
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "source": "compat_layer",
            "priority": "normal"
        }))
    }
    
    fn initialize(&mut self) -> Result<(), CompatError> {
        info!("AetherOS2_5Layer: 初期化");
        // 特別な初期化が必要な場合はここに実装
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), CompatError> {
        info!("AetherOS2_5Layer: クリーンアップ");
        // リソース解放などが必要な場合はここに実装
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_translation() {
        let layer = AetherOS2_5Layer::new();
        
        // 基本的なAPI名の変換テスト
        assert_eq!(
            layer.translate_api_name("windowManager.createStandardWindow"),
            "windowManager.createWindow"
        );
        assert_eq!(
            layer.translate_api_name("systemInfo.getCpuUsage"),
            "systemInfo.getCpuUtilization"
        );
        
        // マッピングにないAPI名はそのまま
        assert_eq!(
            layer.translate_api_name("network.httpRequest"),
            "network.httpRequest"
        );
    }
    
    #[test]
    fn test_window_create_transformation() {
        let layer = AetherOS2_5Layer::new();
        
        // ウィンドウ作成の変換テスト
        let args = vec![
            serde_json::json!({
                "title": "Test Window",
                "width": 1024,
                "height": 768,
                "useNativeDecorations": true,
                "resizable": true
            }),
        ];
        
        let result = layer.translate_api_call("windowManager.createStandardWindow", &args).unwrap();
        
        // 結果の検証
        assert_eq!(result["title"], "Test Window");
        assert_eq!(result["width"], 1024);
        assert_eq!(result["height"], 768);
        assert_eq!(result["decorations"], true); // useNativeDecorationsからdecorationsに変換
        assert_eq!(result["resizable"], true);
        assert_eq!(result["theme"], "system"); // デフォルト値が追加
        assert_eq!(result["minWidth"], 1024); // widthからminWidthに複製
        assert_eq!(result["minHeight"], 768); // heightからminHeightに複製
    }
    
    #[test]
    fn test_websocket_transformation() {
        let layer = AetherOS2_5Layer::new();
        
        // WebSocket作成の変換テスト
        let args = vec![
            serde_json::json!({
                "url": "wss://example.com/socket",
                "protocols": ["v1", "v2"],
                "autoReconnect": true,
                "maxRetries": 5
            }),
        ];
        
        let result = layer.translate_api_call("network.websocket.create", &args).unwrap();
        
        // 結果の検証
        assert_eq!(result["url"], "wss://example.com/socket");
        assert_eq!(result["protocols"][0], "v1");
        assert_eq!(result["protocols"][1], "v2");
        assert_eq!(result["reconnect"], true); // autoReconnectからreconnectに変換
        assert_eq!(result["maxReconnectAttempts"], 5); // maxRetriesからmaxReconnectAttemptsに変換
        assert_eq!(result["reconnectInterval"], 3000); // デフォルト値が追加
        assert_eq!(result["binaryType"], "arraybuffer"); // デフォルト値が追加
    }
} 