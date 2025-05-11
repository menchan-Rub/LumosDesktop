// LumosDesktop 設定モジュールテスト

#[cfg(test)]
mod settings_tests {
    use std::path::PathBuf;
    use tempfile::tempdir;
    
    // 設定モジュールをインポート
    use lumos_desktop::core::settings::{
        SettingsManager, SettingsManagerConfig, SettingsError,
        registry::{SettingsRegistry, SettingsValue},
        profile_manager::{ProfileManager, UserProfile, ProfileType},
        schema::{SchemaManager, SchemaConstraint, SchemaType, SettingsSchema},
        sync_agent::{SyncAgent, SyncConfig, SyncProvider, SyncDirection, SyncStatus}
    };
    
    // 基本的な設定の作成と取得をテスト
    #[test]
    fn test_basic_settings() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config");
        
        // 設定マネージャーを作成
        let mut manager_config = SettingsManagerConfig::default();
        manager_config.base_directory = config_path.clone();
        manager_config.default_settings_file = config_path.join("defaults.json");
        manager_config.schema_directory = config_path.join("schema");
        manager_config.profiles_directory = config_path.join("profiles");
        
        let mut manager = SettingsManager::with_config(manager_config);
        manager.initialize()?;
        
        // 設定を設定
        manager.set("app.window.width", 800)?;
        manager.set("app.window.height", 600)?;
        manager.set("app.window.title", "テストウィンドウ")?;
        manager.set("app.theme", "dark")?;
        
        // 設定を取得
        let width: i32 = manager.get("app.window.width")?;
        let height: i32 = manager.get("app.window.height")?;
        let title: String = manager.get("app.window.title")?;
        let theme: String = manager.get("app.theme")?;
        
        // 値を検証
        assert_eq!(width, 800);
        assert_eq!(height, 600);
        assert_eq!(title, "テストウィンドウ");
        assert_eq!(theme, "dark");
        
        // 存在しない設定
        let result: Result<String, _> = manager.get("app.non_existent");
        assert!(result.is_err());
        
        // 設定を保存
        manager.save()?;
        
        Ok(())
    }
    
    // スキーマ検証をテスト
    #[test]
    fn test_schema_validation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let schema_dir = temp_dir.path().join("schema");
        
        // スキーママネージャーを作成
        let mut schema_manager = SchemaManager::new(&schema_dir);
        schema_manager.initialize()?;
        
        // スキーマを作成
        let schema = SettingsSchema::new("app_settings", "1.0")
            .description("アプリケーション設定")
            .constraint("app.window.width", 
                SchemaConstraint::integer()
                    .minimum(100.0)
                    .maximum(4000.0)
                    .description("ウィンドウの幅")
            )
            .constraint("app.window.height", 
                SchemaConstraint::integer()
                    .minimum(100.0)
                    .maximum(4000.0)
                    .description("ウィンドウの高さ")
            )
            .constraint("app.window.title", 
                SchemaConstraint::string()
                    .min_length(1)
                    .max_length(100)
                    .description("ウィンドウのタイトル")
            )
            .constraint("app.theme", 
                SchemaConstraint::enum_type(vec![
                    serde_json::json!("light"),
                    serde_json::json!("dark"),
                    serde_json::json!("system")
                ])
                .description("アプリケーションのテーマ")
            );
        
        // スキーマを登録
        schema_manager.register_schema(schema)?;
        
        // 有効な値の検証
        schema_manager.validate_value("app.window.width", &800)?;
        schema_manager.validate_value("app.window.height", &600)?;
        schema_manager.validate_value("app.window.title", &"テストウィンドウ")?;
        schema_manager.validate_value("app.theme", &"dark")?;
        
        // 無効な値の検証
        assert!(schema_manager.validate_value("app.window.width", &50).is_err());
        assert!(schema_manager.validate_value("app.window.height", &5000).is_err());
        assert!(schema_manager.validate_value("app.window.title", &"").is_err());
        assert!(schema_manager.validate_value("app.theme", &"invalid").is_err());
        
        Ok(())
    }
    
    // プロファイル管理をテスト
    #[test]
    fn test_profile_management() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let profiles_dir = temp_dir.path().join("profiles");
        
        // プロファイルマネージャーを作成
        let mut profile_manager = ProfileManager::new(&profiles_dir);
        profile_manager.initialize()?;
        
        // デフォルトプロファイルが作成されているか確認
        assert!(profile_manager.has_default_profile());
        let default_profile_id = profile_manager.get_default_profile_id().unwrap();
        
        // 新しいプロファイルを作成
        let profile_id = profile_manager.create_profile("テストプロファイル", ProfileType::User)?;
        
        // プロファイルを取得して設定を追加
        let profile = profile_manager.get_profile_mut(&profile_id).unwrap();
        profile.set("app.window.width", 1024)?;
        profile.set("app.window.height", 768)?;
        
        // プロファイルから設定を取得
        let width: i32 = profile.get("app.window.width")?;
        let height: i32 = profile.get("app.window.height")?;
        
        assert_eq!(width, 1024);
        assert_eq!(height, 768);
        
        // プロファイルをアクティブに設定
        profile_manager.set_active_profile(profile_id.clone())?;
        assert_eq!(profile_manager.get_active_profile_id().unwrap(), profile_id);
        
        // すべてのプロファイルを確認
        let profiles = profile_manager.get_all_profiles();
        assert_eq!(profiles.len(), 2); // デフォルトとテスト
        
        // プロファイルを保存
        profile_manager.save_all()?;
        
        Ok(())
    }
    
    // 同期エージェントをテスト
    #[test]
    fn test_sync_agent() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let sync_dir = temp_dir.path().join("sync");
        
        std::fs::create_dir_all(&sync_dir)?;
        
        // 同期設定を作成
        let mut sync_config = SyncConfig::default();
        sync_config.enabled = true;
        sync_config.provider = SyncProvider::FileSystem(sync_dir);
        sync_config.direction = SyncDirection::Bidirectional;
        sync_config.auto_sync = true;
        
        // 同期エージェントを作成
        let mut sync_agent = SyncAgent::with_config(sync_config);
        sync_agent.initialize()?;
        
        // 初期状態を確認
        assert_eq!(sync_agent.get_status(), SyncStatus::Idle);
        assert!(sync_agent.get_config().enabled);
        
        // 同期を実行
        let result = sync_agent.sync()?;
        assert!(result.success);
        
        // 同期を無効化
        sync_agent.disable();
        assert!(!sync_agent.get_config().enabled);
        assert_eq!(sync_agent.get_status(), SyncStatus::Paused);
        
        // 無効化された状態で同期を試みる
        let result = sync_agent.sync();
        assert!(result.is_err());
        
        Ok(())
    }
} 