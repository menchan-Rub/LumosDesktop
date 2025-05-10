// LumosDesktop システムモジュール
// システムレベルのサービスとハードウェア監視機能を提供します

//! # システムモジュール
//!
//! このモジュールはLumosDesktopのシステムレベルの機能を提供します。
//! ハードウェア監視、電源管理、通知サービス、セキュリティコンテキスト、
//! 触覚フィードバックなどのシステムレベルの機能が含まれています。
//!
//! システムモジュールは、アプリケーションとオペレーティングシステムの
//! 間のインターフェースとして機能し、ハードウェアリソースへの
//! 統一されたアクセスを提供します。

pub mod hardware_monitor;
pub mod power_interface;
pub mod notification_service;
pub mod security_context;
pub mod haptics;

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::time::Duration;

// 主要なモジュールの公開型をre-export
pub use hardware_monitor::{
    HardwareMonitor,
    HardwareMonitorConfig,
    HistoryDataType,
    MonitoringData,
    DataPoint
};

pub use hardware_monitor::resource_alert::{
    ResourceAlert,
    AlertType,
    AlertLevel,
    AlertManager
};

pub use power_interface::{
    PowerManager,
    PowerState,
    PowerProfile,
    PowerEvent,
    BatteryState
};

pub use notification_service::{
    NotificationService,
    Notification,
    NotificationPriority,
    NotificationCategory
};

pub use security_context::{
    SecurityContext,
    SecurityLevel,
    PermissionSet,
    Permission
};

pub use haptics::{
    HapticFeedback,
    HapticEffect,
    HapticIntensity
};

/// システムモジュールのエラー型
#[derive(Debug)]
pub enum SystemError {
    /// ハードウェアモニターエラー
    HardwareMonitor(String),
    /// 電源管理エラー
    PowerInterface(String),
    /// 通知サービスエラー
    NotificationService(String),
    /// セキュリティコンテキストエラー
    SecurityContext(String),
    /// 触覚フィードバックエラー
    Haptics(String),
    /// 設定読み込みエラー
    Configuration(String),
    /// 初期化エラー
    Initialization(String),
    /// その他のエラー
    Other(String),
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::HardwareMonitor(msg) => write!(f, "ハードウェアモニターエラー: {}", msg),
            SystemError::PowerInterface(msg) => write!(f, "電源管理エラー: {}", msg),
            SystemError::NotificationService(msg) => write!(f, "通知サービスエラー: {}", msg),
            SystemError::SecurityContext(msg) => write!(f, "セキュリティコンテキストエラー: {}", msg),
            SystemError::Haptics(msg) => write!(f, "触覚フィードバックエラー: {}", msg),
            SystemError::Configuration(msg) => write!(f, "設定読み込みエラー: {}", msg),
            SystemError::Initialization(msg) => write!(f, "初期化エラー: {}", msg),
            SystemError::Other(msg) => write!(f, "システムエラー: {}", msg),
        }
    }
}

impl Error for SystemError {}

/// システムモジュールのリソースタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemResourceType {
    /// CPU
    Cpu,
    /// メモリ
    Memory,
    /// ストレージ
    Storage,
    /// ネットワーク
    Network,
    /// バッテリー
    Battery,
    /// センサー
    Sensor,
    /// セキュリティ
    Security,
    /// その他
    Other,
}

/// システムモジュールの設定
#[derive(Debug, Clone)]
pub struct SystemConfig {
    /// ハードウェアモニターの設定
    pub hardware_monitor: hardware_monitor::HardwareMonitorConfig,
    /// 電源管理の設定
    pub power_interface: HashMap<String, String>,
    /// 通知サービスの設定
    pub notification_service: HashMap<String, String>,
    /// セキュリティコンテキストの設定
    pub security_context: HashMap<String, String>,
    /// 触覚フィードバックの設定
    pub haptics: HashMap<String, String>,
    /// システム全体のポーリング間隔（ミリ秒）
    pub polling_interval_ms: u64,
    /// システムレベルのログ有効化フラグ
    pub enable_logging: bool,
    /// カスタム設定
    pub custom_settings: HashMap<String, String>,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            hardware_monitor: hardware_monitor::HardwareMonitorConfig::default(),
            power_interface: HashMap::new(),
            notification_service: HashMap::new(),
            security_context: HashMap::new(),
            haptics: HashMap::new(),
            polling_interval_ms: 5000, // 5秒
            enable_logging: true,
            custom_settings: HashMap::new(),
        }
    }
}

/// システムサブシステムのステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsystemStatus {
    /// 正常に動作中
    Running,
    /// 初期化中
    Initializing,
    /// 停止中
    Stopped,
    /// エラー発生
    Error,
    /// アイドル状態
    Idle,
    /// 一時停止中
    Suspended,
}

impl fmt::Display for SubsystemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubsystemStatus::Running => write!(f, "実行中"),
            SubsystemStatus::Initializing => write!(f, "初期化中"),
            SubsystemStatus::Stopped => write!(f, "停止"),
            SubsystemStatus::Error => write!(f, "エラー"),
            SubsystemStatus::Idle => write!(f, "アイドル"),
            SubsystemStatus::Suspended => write!(f, "一時停止"),
        }
    }
}

/// システムマネージャー
///
/// システム全体のサブシステムを管理し、各モジュール間の連携を調整します。
pub struct SystemManager {
    /// 設定
    config: SystemConfig,
    /// ハードウェアモニター
    hardware_monitor: Option<Arc<Mutex<HardwareMonitor>>>,
    /// 電源管理
    power_manager: Option<Arc<Mutex<power_interface::PowerManager>>>,
    /// 通知サービス
    notification_service: Option<Arc<Mutex<notification_service::NotificationService>>>,
    /// セキュリティコンテキスト
    security_context: Option<Arc<RwLock<security_context::SecurityContext>>>,
    /// 触覚フィードバック
    haptic_feedback: Option<Arc<Mutex<haptics::HapticFeedback>>>,
    /// サブシステムのステータス
    subsystem_status: Arc<RwLock<HashMap<&'static str, SubsystemStatus>>>,
    /// システムイベントハンドラ
    event_handlers: HashMap<String, Box<dyn Fn(&str) -> Result<(), SystemError> + Send + Sync>>,
    /// セッション開始時間
    #[allow(dead_code)]
    start_time: std::time::Instant,
}

impl SystemManager {
    /// 新しいシステムマネージャーを作成
    pub fn new() -> Self {
        Self {
            config: SystemConfig::default(),
            hardware_monitor: None,
            power_manager: None,
            notification_service: None,
            security_context: None,
            haptic_feedback: None,
            subsystem_status: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }
    
    /// 設定を指定してシステムマネージャーを作成
    pub fn with_config(config: SystemConfig) -> Self {
        Self {
            config,
            hardware_monitor: None,
            power_manager: None,
            notification_service: None,
            security_context: None,
            haptic_feedback: None,
            subsystem_status: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }
    
    /// システムマネージャーを初期化
    pub fn initialize(&mut self) -> Result<(), SystemError> {
        // システムサブシステムを初期化
        self.initialize_hardware_monitor()?;
        self.initialize_power_manager()?;
        self.initialize_notification_service()?;
        self.initialize_security_context()?;
        self.initialize_haptic_feedback()?;
        
        // システムイベントハンドラを設定
        self.setup_event_handlers()?;
        
        // サブシステム間の相互接続を設定
        self.connect_subsystems()?;
        
        Ok(())
    }
    
    /// ハードウェアモニターを初期化
    fn initialize_hardware_monitor(&mut self) -> Result<(), SystemError> {
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("hardware_monitor", SubsystemStatus::Initializing);
        }
        
        // ハードウェアモニターを作成
        let monitor = HardwareMonitor::with_config(self.config.hardware_monitor.clone());
        let monitor = Arc::new(Mutex::new(monitor));
        self.hardware_monitor = Some(monitor);
        
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("hardware_monitor", SubsystemStatus::Running);
        }
        
        Ok(())
    }
    
    /// 電源管理を初期化
    fn initialize_power_manager(&mut self) -> Result<(), SystemError> {
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("power_manager", SubsystemStatus::Initializing);
        }
        
        // 電源管理を作成
        let power_manager = power_interface::PowerManager::new();
        let power_manager = Arc::new(Mutex::new(power_manager));
        self.power_manager = Some(power_manager);
        
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("power_manager", SubsystemStatus::Running);
        }
        
        Ok(())
    }
    
    /// 通知サービスを初期化
    fn initialize_notification_service(&mut self) -> Result<(), SystemError> {
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("notification_service", SubsystemStatus::Initializing);
        }
        
        // 通知サービスを作成
        let notification_service = notification_service::NotificationService::new();
        let notification_service = Arc::new(Mutex::new(notification_service));
        self.notification_service = Some(notification_service);
        
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("notification_service", SubsystemStatus::Running);
        }
        
        Ok(())
    }
    
    /// セキュリティコンテキストを初期化
    fn initialize_security_context(&mut self) -> Result<(), SystemError> {
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("security_context", SubsystemStatus::Initializing);
        }
        
        // セキュリティコンテキストを作成
        let security_context = security_context::SecurityContext::new();
        let security_context = Arc::new(RwLock::new(security_context));
        self.security_context = Some(security_context);
        
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("security_context", SubsystemStatus::Running);
        }
        
        Ok(())
    }
    
    /// 触覚フィードバックを初期化
    fn initialize_haptic_feedback(&mut self) -> Result<(), SystemError> {
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("haptic_feedback", SubsystemStatus::Initializing);
        }
        
        // 触覚フィードバックを作成
        let haptic_feedback = haptics::HapticFeedback::new();
        let haptic_feedback = Arc::new(Mutex::new(haptic_feedback));
        self.haptic_feedback = Some(haptic_feedback);
        
        // ステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            status.insert("haptic_feedback", SubsystemStatus::Running);
        }
        
        Ok(())
    }
    
    /// システムイベントハンドラを設定
    fn setup_event_handlers(&mut self) -> Result<(), SystemError> {
        // ハードウェアモニターからの高温アラートをハンドル
        let power_manager_clone = self.power_manager.clone();
        let notification_service_clone = self.notification_service.clone();
        
        let high_temp_handler: Box<dyn Fn(&str) -> Result<(), SystemError> + Send + Sync> = 
            Box::new(move |message| {
                // 電源プロファイルを省電力モードに変更
                if let Some(power_manager) = &power_manager_clone {
                    let mut power_manager = power_manager.lock().unwrap();
                    power_manager.set_power_profile(PowerProfile::PowerSaver)
                        .map_err(|e| SystemError::PowerInterface(e.to_string()))?;
                }
                
                // 通知を送信
                if let Some(notification_service) = &notification_service_clone {
                    let mut notification_service = notification_service.lock().unwrap();
                    let notification = Notification::new(
                        "システム",
                        &format!("温度警告: {}", message),
                        NotificationCategory::System,
                        NotificationPriority::High
                    );
                    notification_service.send(notification)
                        .map_err(|e| SystemError::NotificationService(e.to_string()))?;
                }
                
                Ok(())
            });
        
        self.event_handlers.insert("high_temperature".to_string(), high_temp_handler);
        
        // 低バッテリーアラートをハンドル
        let power_manager_clone = self.power_manager.clone();
        let notification_service_clone = self.notification_service.clone();
        
        let low_battery_handler: Box<dyn Fn(&str) -> Result<(), SystemError> + Send + Sync> = 
            Box::new(move |message| {
                // 電源プロファイルを省電力モードに変更
                if let Some(power_manager) = &power_manager_clone {
                    let mut power_manager = power_manager.lock().unwrap();
                    power_manager.set_power_profile(PowerProfile::PowerSaver)
                        .map_err(|e| SystemError::PowerInterface(e.to_string()))?;
                }
                
                // 通知を送信
                if let Some(notification_service) = &notification_service_clone {
                    let mut notification_service = notification_service.lock().unwrap();
                    let notification = Notification::new(
                        "システム",
                        &format!("バッテリー警告: {}", message),
                        NotificationCategory::System,
                        NotificationPriority::High
                    );
                    notification_service.send(notification)
                        .map_err(|e| SystemError::NotificationService(e.to_string()))?;
                }
                
                Ok(())
            });
        
        self.event_handlers.insert("low_battery".to_string(), low_battery_handler);
        
        Ok(())
    }
    
    /// サブシステム間の相互接続を設定
    fn connect_subsystems(&mut self) -> Result<(), SystemError> {
        // ハードウェアモニターとリソースアラートの連携
        if let Some(hardware_monitor) = &self.hardware_monitor {
            let mut monitor = hardware_monitor.lock().unwrap();
            
            // 通知サービスへのアラート転送を設定
            let notification_service_clone = self.notification_service.clone();
            if let Some(notification_service) = &notification_service_clone {
                monitor.register_alert_callback(move |alert| {
                    if alert.should_notify() {
                        if let Some(notification_service) = &notification_service {
                            let mut service = notification_service.lock().unwrap();
                            
                            // アラートの重要度に基づいて通知優先度を決定
                            let priority = match alert.level {
                                AlertLevel::Info => NotificationPriority::Low,
                                AlertLevel::Warning => NotificationPriority::Medium,
                                AlertLevel::Critical => NotificationPriority::High,
                                AlertLevel::Emergency => NotificationPriority::Critical,
                            };
                            
                            let notification = Notification::new(
                                &format!("システム: {}", alert.alert_type),
                                &alert.message,
                                NotificationCategory::System,
                                priority
                            );
                            
                            let _ = service.send(notification);
                        }
                    }
                });
            }
            
            // リソース使用状況に応じた電源管理の調整
            let power_manager_clone = self.power_manager.clone();
            if let Some(power_manager) = &power_manager_clone {
                monitor.register_alert_callback(move |alert| {
                    match alert.alert_type {
                        AlertType::HighCpuUsage | AlertType::HighTemperature => {
                            if alert.level >= AlertLevel::Warning {
                                if let Some(power_manager) = &power_manager {
                                    let mut manager = power_manager.lock().unwrap();
                                    let _ = manager.set_power_profile(PowerProfile::PowerSaver);
                                }
                            }
                        },
                        AlertType::LowBattery | AlertType::CriticalBattery => {
                            if let Some(power_manager) = &power_manager {
                                let mut manager = power_manager.lock().unwrap();
                                let _ = manager.set_power_profile(PowerProfile::PowerSaver);
                                
                                // 危機的なバッテリー残量の場合はさらに省電力設定を適用
                                if alert.level == AlertLevel::Critical {
                                    let _ = manager.enable_extreme_power_saving();
                                }
                            }
                        },
                        _ => {}
                    }
                });
            }
            
            // モニタリングを開始
            if let Err(e) = monitor.start() {
                return Err(SystemError::HardwareMonitor(e));
            }
        }
        
        // 電源管理と通知サービスの連携
        if let (Some(power_manager), Some(notification_service)) = 
            (&self.power_manager, &self.notification_service) {
            
            let notification_service_clone = notification_service.clone();
            let mut manager = power_manager.lock().unwrap();
            
            manager.set_power_event_callback(move |event| {
                let mut service = notification_service_clone.lock().unwrap();
                
                match event {
                    PowerEvent::BatteryLow(percentage) => {
                        let notification = Notification::new(
                            "電源",
                            &format!("バッテリー残量が低下しています: {}%", percentage),
                            NotificationCategory::System,
                            NotificationPriority::Medium
                        );
                        let _ = service.send(notification);
                    },
                    PowerEvent::BatteryCritical(percentage) => {
                        let notification = Notification::new(
                            "電源",
                            &format!("バッテリー残量が危機的に低下しています: {}%。すぐに充電してください。", percentage),
                            NotificationCategory::System,
                            NotificationPriority::Critical
                        );
                        let _ = service.send(notification);
                    },
                    PowerEvent::PowerSourceChanged(source) => {
                        let message = match source {
                            PowerState::AC => "AC電源に接続されました",
                            PowerState::Battery => "バッテリー駆動に切り替わりました",
                            PowerState::UPS => "UPS電源に切り替わりました",
                            PowerState::Unknown => "電源状態が変更されました",
                        };
                        
                        let notification = Notification::new(
                            "電源",
                            message,
                            NotificationCategory::System,
                            NotificationPriority::Low
                        );
                        let _ = service.send(notification);
                    },
                    _ => {}
                }
            });
        }
        
        Ok(())
    }
    
    /// ハードウェアモニターを取得
    pub fn get_hardware_monitor(&self) -> Option<Arc<Mutex<HardwareMonitor>>> {
        self.hardware_monitor.clone()
    }
    
    /// 電源管理を取得
    pub fn get_power_manager(&self) -> Option<Arc<Mutex<power_interface::PowerManager>>> {
        self.power_manager.clone()
    }
    
    /// 通知サービスを取得
    pub fn get_notification_service(&self) -> Option<Arc<Mutex<notification_service::NotificationService>>> {
        self.notification_service.clone()
    }
    
    /// セキュリティコンテキストを取得
    pub fn get_security_context(&self) -> Option<Arc<RwLock<security_context::SecurityContext>>> {
        self.security_context.clone()
    }
    
    /// 触覚フィードバックを取得
    pub fn get_haptic_feedback(&self) -> Option<Arc<Mutex<haptics::HapticFeedback>>> {
        self.haptic_feedback.clone()
    }
    
    /// サブシステムのステータスを取得
    pub fn get_subsystem_status(&self, name: &str) -> Option<SubsystemStatus> {
        let status = self.subsystem_status.read().unwrap();
        status.get(name).copied()
    }
    
    /// すべてのサブシステムのステータスを取得
    pub fn get_all_subsystem_status(&self) -> HashMap<&'static str, SubsystemStatus> {
        let status = self.subsystem_status.read().unwrap();
        status.clone()
    }
    
    /// システムイベントを発行
    pub fn emit_event(&self, event: &str, message: &str) -> Result<(), SystemError> {
        if let Some(handler) = self.event_handlers.get(event) {
            handler(message)?;
        }
        Ok(())
    }
    
    /// ハードウェアリソースの使用状況をチェック
    pub fn check_resource_usage(&self) -> Result<HashMap<SystemResourceType, f64>, SystemError> {
        let mut usage = HashMap::new();
        
        if let Some(hardware_monitor) = &self.hardware_monitor {
            let monitor = hardware_monitor.lock().unwrap();
            let data = monitor.get_current_data();
            
            // CPU使用率
            if let Some(cpu_info) = &data.cpu_info {
                usage.insert(SystemResourceType::Cpu, cpu_info.usage.total);
            }
            
            // メモリ使用率
            if let Some(memory_info) = &data.memory_info {
                usage.insert(SystemResourceType::Memory, memory_info.usage.used_percent);
            }
            
            // バッテリー残量
            if let Some(battery_info) = &data.battery_info {
                usage.insert(SystemResourceType::Battery, battery_info.percentage);
            }
        }
        
        Ok(usage)
    }
    
    /// システム全体のステータスを取得
    pub fn get_system_status(&self) -> Result<HashMap<String, String>, SystemError> {
        let mut status = HashMap::new();
        
        // サブシステムのステータス
        for (name, subsystem_status) in self.subsystem_status.read().unwrap().iter() {
            status.insert(format!("subsystem.{}", name), format!("{}", subsystem_status));
        }
        
        // リソース使用状況
        if let Ok(resource_usage) = self.check_resource_usage() {
            for (resource, usage) in resource_usage {
                status.insert(format!("resource.{:?}", resource), format!("{:.1}%", usage));
            }
        }
        
        // 電源状態
        if let Some(power_manager) = &self.power_manager {
            let manager = power_manager.lock().unwrap();
            status.insert("power.state".to_string(), format!("{:?}", manager.get_power_state()));
            status.insert("power.profile".to_string(), format!("{:?}", manager.get_power_profile()));
            
            if let Some(battery) = manager.get_battery_state() {
                status.insert("battery.percentage".to_string(), format!("{:.1}%", battery.percentage));
                status.insert("battery.time_remaining".to_string(), 
                             format!("{} 分", battery.time_remaining_minutes.unwrap_or(0)));
            }
        }
        
        Ok(status)
    }
    
    /// システムをシャットダウン
    pub fn shutdown(&mut self) -> Result<(), SystemError> {
        // ハードウェアモニターを停止
        if let Some(hardware_monitor) = &self.hardware_monitor {
            let monitor = hardware_monitor.lock().unwrap();
            if let Err(e) = monitor.stop() {
                return Err(SystemError::HardwareMonitor(e));
            }
        }
        
        // 通知サービスを停止
        if let Some(notification_service) = &self.notification_service {
            let mut service = notification_service.lock().unwrap();
            if let Err(e) = service.shutdown() {
                return Err(SystemError::NotificationService(e.to_string()));
            }
        }
        
        // すべてのサブシステムのステータスを更新
        {
            let mut status = self.subsystem_status.write().unwrap();
            for (_, subsystem_status) in status.iter_mut() {
                *subsystem_status = SubsystemStatus::Stopped;
            }
        }
        
        Ok(())
    }
    
    /// 定期的なポーリングを開始（バックグランドスレッド）
    pub fn start_polling(&self) -> Result<std::thread::JoinHandle<()>, SystemError> {
        let polling_interval = Duration::from_millis(self.config.polling_interval_ms);
        let hardware_monitor = self.hardware_monitor.clone();
        let power_manager = self.power_manager.clone();
        let subsystem_status = self.subsystem_status.clone();
        
        let handle = std::thread::spawn(move || {
            loop {
                // サブシステムの状態をチェック
                {
                    let status = subsystem_status.read().unwrap();
                    let all_running = status.values()
                        .all(|s| *s == SubsystemStatus::Running || *s == SubsystemStatus::Idle);
                    
                    if !all_running {
                        // 何かが停止またはエラー状態の場合は終了
                        break;
                    }
                }
                
                // ハードウェアリソースをチェック
                if let Some(monitor) = &hardware_monitor {
                    let data = monitor.lock().unwrap().get_current_data();
                    
                    // CPU温度をチェック
                    if let Some(cpu_info) = &data.cpu_info {
                        if let Some(temp) = cpu_info.temperature {
                            if temp > 80.0 {  // 80℃を超えると危険
                                // 電源管理を調整
                                if let Some(power_mgr) = &power_manager {
                                    let mut manager = power_mgr.lock().unwrap();
                                    let _ = manager.set_power_profile(PowerProfile::PowerSaver);
                                }
                            }
                        }
                    }
                }
                
                // ポーリング間隔を待機
                std::thread::sleep(polling_interval);
            }
        });
        
        Ok(handle)
    }
}

/// モジュールのユニットテスト
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_config_default() {
        let config = SystemConfig::default();
        assert!(config.enable_logging);
        assert_eq!(config.polling_interval_ms, 5000);
    }
    
    #[test]
    fn test_system_manager_creation() {
        let manager = SystemManager::new();
        assert!(manager.hardware_monitor.is_none());
        assert!(manager.power_manager.is_none());
        assert!(manager.notification_service.is_none());
        assert!(manager.security_context.is_none());
        assert!(manager.haptic_feedback.is_none());
    }
    
    #[test]
    fn test_subsystem_status_display() {
        assert_eq!(format!("{}", SubsystemStatus::Running), "実行中");
        assert_eq!(format!("{}", SubsystemStatus::Stopped), "停止");
        assert_eq!(format!("{}", SubsystemStatus::Error), "エラー");
        assert_eq!(format!("{}", SubsystemStatus::Initializing), "初期化中");
        assert_eq!(format!("{}", SubsystemStatus::Idle), "アイドル");
        assert_eq!(format!("{}", SubsystemStatus::Suspended), "一時停止");
    }
    
    #[test]
    fn test_system_error_display() {
        let err = SystemError::HardwareMonitor("テストエラー".to_string());
        assert_eq!(format!("{}", err), "ハードウェアモニターエラー: テストエラー");
        
        let err = SystemError::PowerInterface("電源テスト".to_string());
        assert_eq!(format!("{}", err), "電源管理エラー: 電源テスト");
        
        let err = SystemError::Other("その他のエラー".to_string());
        assert_eq!(format!("{}", err), "システムエラー: その他のエラー");
    }
} 