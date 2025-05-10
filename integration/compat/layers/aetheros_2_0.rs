// AetherOS 2.0 互換性レイヤー
// AetherOS 2.0 APIとの互換性を提供します

use crate::integration::compat::{CompatibilityLayer, CompatError, CompatibleVersion};
use log::{debug, warn, info};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

/// AetherOS 2.0互換性レイヤー
pub struct AetherOS2_0Layer {
    // APIマッピングテーブル - 古いAPI名から新しいAPI名へのマッピング
    api_mappings: HashMap<String, String>,
    // 変換テーブル - 特殊な変換が必要なAPIの処理関数
    transformers: HashMap<String, fn(&[Value]) -> Result<Value, CompatError>>,
    // リソースマッピングテーブル
    resource_mappings: HashMap<String, String>,
    // 統計情報
    call_count: RwLock<HashMap<String, u64>>,
}

impl AetherOS2_0Layer {
    /// 新しいAetherOS 2.0互換性レイヤーを作成します
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
        // AetherOS 2.0と3.0ではAPIがかなり近いため、マッピングは少ない
        
        // ディスプレイ関連
        self.api_mappings.insert(
            "display.getScreenInfo".to_string(), 
            "display.getScreenDimensions".to_string()
        );
        
        // ウィンドウ関連
        self.api_mappings.insert(
            "windowManager.createApplicationWindow".to_string(), 
            "windowManager.createWindow".to_string()
        );
        
        // システム関連
        self.api_mappings.insert(
            "systemInfo.getProcessorInfo".to_string(), 
            "systemInfo.getCpuDetails".to_string()
        );
        self.api_mappings.insert(
            "systemInfo.getMemoryInfo".to_string(), 
            "systemInfo.getMemoryDetails".to_string()
        );
        self.api_mappings.insert(
            "powerManager.powerOff".to_string(), 
            "powerManager.shutdown".to_string()
        );
        self.api_mappings.insert(
            "powerManager.reboot".to_string(), 
            "powerManager.restart".to_string()
        );
        
        // ネットワーク関連
        self.api_mappings.insert(
            "network.createWebsocket".to_string(), 
            "network.createWebSocketConnection".to_string()
        );
        
        // センサー関連
        self.api_mappings.insert(
            "sensors.accelerometer.getData".to_string(), 
            "sensors.getAccelerometerData".to_string()
        );
        self.api_mappings.insert(
            "sensors.gyroscope.getData".to_string(), 
            "sensors.getGyroscopeData".to_string()
        );
        self.api_mappings.insert(
            "sensors.magnetometer.getData".to_string(), 
            "sensors.getMagnetometerData".to_string()
        );
    }
    
    /// 特別な変換が必要なAPIのトランスフォーマーを設定
    fn setup_transformers(&mut self) {
        // スクリーン情報取得の変換
        self.transformers.insert("display.getScreenInfo".to_string(), |args| {
            // AetherOS 2.0では複合的な情報を返していたが、3.0では分割された
            Ok(serde_json::json!({
                "includeRefreshRate": true,
                "includeScaleFactor": true
            }))
        });
        
        // アプリケーションウィンドウ作成の変換
        self.transformers.insert("windowManager.createApplicationWindow".to_string(), |args| {
            if args.len() < 3 {
                return Err(CompatError::SystemCallError("ウィンドウパラメータが不足しています".to_string()));
            }
            
            let title = match &args[0] {
                Value::String(title_str) => title_str.clone(),
                _ => return Err(CompatError::SystemCallError("タイトルは文字列である必要があります".to_string())),
            };
            
            let width = match args[1].as_u64() {
                Some(w) => w as u32,
                None => return Err(CompatError::SystemCallError("幅は数値である必要があります".to_string())),
            };
            
            let height = match args[2].as_u64() {
                Some(h) => h as u32,
                None => return Err(CompatError::SystemCallError("高さは数値である必要があります".to_string())),
            };
            
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
                "maximized": false,
                "centered": true
            }))
        });
        
        // センサーデータ取得の変換
        self.transformers.insert("sensors.accelerometer.getData".to_string(), |_args| {
            // AetherOS 3.0では設定パラメータが追加された
            Ok(serde_json::json!({
                "samplingRate": "normal",
                "filterEnabled": true
            }))
        });
        
        // システムメモリ情報取得の変換
        self.transformers.insert("systemInfo.getMemoryInfo".to_string(), |args| {
            let detailed = args.get(0).and_then(|v| v.as_bool()).unwrap_or(false);
            
            // 新しいAPIフォーマットに変換
            Ok(serde_json::json!({
                "includeSwap": true,
                "detailed": detailed,
                "refreshCache": true
            }))
        });
    }
    
    /// リソースタイプのマッピングを設定
    fn setup_resource_mappings(&mut self) {
        // AetherOS 2.0と3.0ではリソースタイプはほぼ同じ
        self.resource_mappings.insert(
            "theme/color-scheme".to_string(), 
            "theme/color-palette".to_string()
        );
        self.resource_mappings.insert(
            "audio/notification".to_string(), 
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

impl CompatibilityLayer for AetherOS2_0Layer {
    fn version(&self) -> CompatibleVersion {
        CompatibleVersion::AetherOS2_0
    }
    
    fn translate_api_call(&self, name: &str, args: &[Value]) -> Result<Value, CompatError> {
        debug!("AetherOS2_0Layer: APIコール変換: {} (引数: {}個)", name, args.len());
        
        // 統計情報を更新
        self.increment_call_count(name);
        
        // 特別な変換が必要なAPIの場合
        if let Some(transformer) = self.transformers.get(name) {
            let result = transformer(args);
            match &result {
                Ok(_) => debug!("AetherOS2_0Layer: 特殊変換成功: {}", name),
                Err(e) => warn!("AetherOS2_0Layer: 特殊変換失敗: {} - エラー: {}", name, e),
            }
            return result;
        }
        
        // 標準的なAPIマッピング
        let new_name = self.translate_api_name(name);
        if new_name != name {
            debug!("AetherOS2_0Layer: API名変換: {} -> {}", name, new_name);
        }
        
        // 引数はそのまま渡す（必要に応じてここで引数の変換も行う）
        Ok(serde_json::json!({
            "api": new_name,
            "args": args,
        }))
    }
    
    fn translate_resource(&self, resource_type: &str, data: &[u8]) -> Result<Vec<u8>, CompatError> {
        let new_type = self.translate_resource_type(resource_type);
        debug!("AetherOS2_0Layer: リソース変換: {} -> {}, サイズ: {}バイト", 
               resource_type, new_type, data.len());
        
        // AetherOS 2.0のほとんどのリソースは変換なしで使用可能
        // 特殊な変換が必要な場合はここに追加
        
        // 現在はデータをそのまま返す
        Ok(data.to_vec())
    }
    
    fn translate_event(&self, event_name: &str, event_data: &Value) -> Result<Value, CompatError> {
        debug!("AetherOS2_0Layer: イベント変換: {}", event_name);
        
        // イベント名のマッピング (AetherOS 2.0と3.0ではイベント名はほぼ同じ)
        let new_event_name = match event_name {
            "displayConfigChanged" => "displaySettingsChanged",
            "systemShuttingDown" => "systemShutdownInitiated",
            "systemRestarting" => "systemRestartInitiated",
            _ => event_name,
        };
        
        if new_event_name != event_name {
            debug!("AetherOS2_0Layer: イベント名変換: {} -> {}", event_name, new_event_name);
        }
        
        // イベントデータは基本的にそのまま渡す
        // 必要に応じてここでイベントデータの変換を行う
        Ok(serde_json::json!({
            "name": new_event_name,
            "data": event_data,
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "source": "compat_layer",
        }))
    }
    
    fn initialize(&mut self) -> Result<(), CompatError> {
        info!("AetherOS2_0Layer: 初期化");
        // 特別な初期化が必要な場合はここに実装
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), CompatError> {
        info!("AetherOS2_0Layer: クリーンアップ");
        // リソース解放などが必要な場合はここに実装
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_translation() {
        let layer = AetherOS2_0Layer::new();
        
        // 基本的なAPI名の変換テスト
        assert_eq!(
            layer.translate_api_name("display.getScreenInfo"),
            "display.getScreenDimensions"
        );
        assert_eq!(
            layer.translate_api_name("systemInfo.getProcessorInfo"),
            "systemInfo.getCpuDetails"
        );
        
        // マッピングにないAPI名はそのまま
        assert_eq!(
            layer.translate_api_name("unknown.api"),
            "unknown.api"
        );
    }
    
    #[test]
    fn test_window_create_transformation() {
        let layer = AetherOS2_0Layer::new();
        
        // ウィンドウ作成の変換テスト
        let args = vec![
            serde_json::json!("Test Application"),
            serde_json::json!(1024),
            serde_json::json!(768),
            serde_json::json!(true),
        ];
        
        let result = layer.translate_api_call("windowManager.createApplicationWindow", &args).unwrap();
        
        // 結果の検証
        assert_eq!(result["title"], "Test Application");
        assert_eq!(result["width"], 1024);
        assert_eq!(result["height"], 768);
        assert_eq!(result["resizable"], true);
        assert_eq!(result["decorations"], true);
        assert_eq!(result["centered"], true);
    }
    
    #[test]
    fn test_sensor_data_transformation() {
        let layer = AetherOS2_0Layer::new();
        
        // センサーデータ取得の変換テスト
        let args = vec![];
        
        let result = layer.translate_api_call("sensors.accelerometer.getData", &args).unwrap();
        
        // 結果の検証
        assert_eq!(result["samplingRate"], "normal");
        assert_eq!(result["filterEnabled"], true);
    }
} 