// 互換性モジュールのテスト

use lumos_desktop::integration::compat::{
    CompatManager, CompatManagerConfig, CompatMode, ApiVersion, CompatibleVersion,
    CompatError, CompatibilityLayer,
};
use lumos_desktop::integration::compat::layers::{
    AetherOS1_0Layer, AetherOS1_5Layer, AetherOS2_0Layer, AetherOS2_5Layer,
};
use lumos_desktop::core::system::process_manager::ProcessManager;
use lumos_desktop::core::window_manager::WindowManager;
use lumos_desktop::core::settings::SettingsManager;

use std::sync::Arc;
use serde_json::json;

// モック実装
// 実際のテストでは、これらのモック実装を適切に実装する必要があります
struct MockProcessManager;
struct MockWindowManager;
struct MockSettingsManager;

impl ProcessManager {
    fn new_mock() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl WindowManager {
    fn new_mock() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl SettingsManager {
    fn new_mock() -> Arc<Self> {
        Arc::new(Self)
    }
}

// テスト用の簡易的な互換性レイヤー
struct TestCompatLayer {
    version: CompatibleVersion,
}

impl TestCompatLayer {
    fn new(version: CompatibleVersion) -> Self {
        Self { version }
    }
}

impl CompatibilityLayer for TestCompatLayer {
    fn version(&self) -> CompatibleVersion {
        self.version.clone()
    }
    
    fn translate_api_call(&self, name: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, CompatError> {
        Ok(json!({
            "test_layer": format!("{}", self.version),
            "api_call": name,
            "args": args,
        }))
    }
    
    fn translate_resource(&self, resource_type: &str, data: &[u8]) -> Result<Vec<u8>, CompatError> {
        // テスト用に先頭に識別子を追加
        let mut result = format!("TEST_{}:", self.version).as_bytes().to_vec();
        result.extend_from_slice(data);
        Ok(result)
    }
    
    fn translate_event(&self, event_name: &str, event_data: &serde_json::Value) -> Result<serde_json::Value, CompatError> {
        Ok(json!({
            "test_layer": format!("{}", self.version),
            "event": event_name,
            "data": event_data,
        }))
    }
    
    fn initialize(&mut self) -> Result<(), CompatError> {
        println!("初期化: {}", self.version);
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), CompatError> {
        println!("クリーンアップ: {}", self.version);
        Ok(())
    }
}

#[test]
fn test_api_version_compatibility() {
    let v1 = ApiVersion::new(2, 0, 0);
    let v2 = ApiVersion::new(2, 1, 0);
    let v3 = ApiVersion::new(1, 5, 0);
    
    assert!(v1.is_compatible_with(&v1)); // 同じバージョン
    assert!(v2.is_compatible_with(&v1)); // メジャーが同じでマイナーが新しい
    assert!(!v1.is_compatible_with(&v2)); // メジャーが同じだがマイナーが古い
    assert!(!v1.is_compatible_with(&v3)); // メジャーが異なる
}

#[test]
fn test_compat_manager_initialization() {
    let process_manager = ProcessManager::new_mock();
    let window_manager = WindowManager::new_mock();
    let settings_manager = SettingsManager::new_mock();
    
    let config = CompatManagerConfig {
        default_mode: CompatMode::Partial,
        simulation_delay_ms: 10,
        resource_cache_size: 100,
        enable_logging: true,
        api_version_override: Some(ApiVersion::new(3, 1, 0)),
        layer_priority: 50,
    };
    
    let compat_manager = CompatManager::new(
        process_manager,
        window_manager,
        settings_manager,
        Some(config),
    );
    
    let current_api = compat_manager.get_current_api_version();
    assert_eq!(current_api.major, 3);
    assert_eq!(current_api.minor, 1);
    assert_eq!(current_api.patch, 0);
}

#[test]
fn test_register_compat_layers() {
    let process_manager = ProcessManager::new_mock();
    let window_manager = WindowManager::new_mock();
    let settings_manager = SettingsManager::new_mock();
    
    let mut compat_manager = CompatManager::new(
        process_manager,
        window_manager,
        settings_manager,
        None,
    );
    
    // テスト用レイヤーを登録
    let layer1 = TestCompatLayer::new(CompatibleVersion::AetherOS1_0);
    let layer2 = TestCompatLayer::new(CompatibleVersion::AetherOS2_0);
    
    assert!(compat_manager.register_compat_layer(Box::new(layer1)).is_ok());
    assert!(compat_manager.register_compat_layer(Box::new(layer2)).is_ok());
    
    // 同じバージョンのレイヤーを登録しようとするとエラー
    let layer_duplicate = TestCompatLayer::new(CompatibleVersion::AetherOS1_0);
    assert!(compat_manager.register_compat_layer(Box::new(layer_duplicate)).is_err());
}

#[test]
fn test_aetheros_1_0_layer_api_translation() {
    let layer = AetherOS1_0Layer::new();
    
    // ウィンドウ作成のテスト
    let window_args = vec![
        json!("My Window"),
        json!(800),
        json!(600),
        json!(true),
    ];
    
    let result = layer.translate_api_call("window.create", &window_args).unwrap();
    
    assert_eq!(result["title"], "My Window");
    assert_eq!(result["width"], 800);
    assert_eq!(result["height"], 600);
    assert_eq!(result["resizable"], true);
    
    // HTTP GETリクエストのテスト
    let http_args = vec![
        json!("https://api.example.com/data"),
        json!({"Authorization": "Bearer token123"}),
    ];
    
    let result = layer.translate_api_call("net.httpGet", &http_args).unwrap();
    
    assert_eq!(result["method"], "GET");
    assert_eq!(result["url"], "https://api.example.com/data");
    assert_eq!(result["headers"]["Authorization"], "Bearer token123");
}

#[test]
fn test_aetheros_2_0_layer_api_translation() {
    let layer = AetherOS2_0Layer::new();
    
    // ウィンドウ作成のテスト
    let window_args = vec![
        json!("Advanced Window"),
        json!(1024),
        json!(768),
        json!(false),
    ];
    
    let result = layer.translate_api_call("windowManager.createApplicationWindow", &window_args).unwrap();
    
    assert_eq!(result["title"], "Advanced Window");
    assert_eq!(result["width"], 1024);
    assert_eq!(result["height"], 768);
    assert_eq!(result["resizable"], false);
    assert_eq!(result["alwaysOnTop"], false);
    assert_eq!(result["centered"], true);
    
    // メモリ情報取得のテスト
    let memory_args = vec![json!(true)];
    
    let result = layer.translate_api_call("systemInfo.getMemoryInfo", &memory_args).unwrap();
    
    assert_eq!(result["includeSwap"], true);
    assert_eq!(result["detailed"], true);
    assert_eq!(result["refreshCache"], true);
}

// さらに多くのテストケースを追加することができます

fn main() {
    println!("互換性モジュールのテスト実行");
    
    // テストの手動実行
    test_api_version_compatibility();
    test_compat_manager_initialization();
    test_register_compat_layers();
    test_aetheros_1_0_layer_api_translation();
    test_aetheros_2_0_layer_api_translation();
    
    println!("すべてのテストが成功しました");
} 