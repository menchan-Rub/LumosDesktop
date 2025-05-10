//! 電源インターフェースモジュール
//!
//! このモジュールは、電源管理機能へのインターフェースを提供します。
//! バッテリー状態の監視、電源プランの管理、システムの電源状態の制御などの機能を含みます。

pub mod events;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use log::{debug, error, info, warn};
use std::fmt;
use std::path::Path;

/// 電源の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerSource {
    /// AC電源
    AC,
    /// バッテリー
    Battery,
    /// 無停電電源装置
    UPS,
    /// 不明または未サポート
    Unknown,
}

impl fmt::Display for PowerSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PowerSource::AC => write!(f, "AC電源"),
            PowerSource::Battery => write!(f, "バッテリー"),
            PowerSource::UPS => write!(f, "UPS"),
            PowerSource::Unknown => write!(f, "不明"),
        }
    }
}

/// 充電状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChargingState {
    /// 充電中
    Charging,
    /// 放電中
    Discharging,
    /// 満充電（電源接続中）
    Full,
    /// バッテリーなし
    NotPresent,
    /// 不明または未サポート
    Unknown,
}

/// バッテリーの健康状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BatteryHealth {
    /// 良好
    Good,
    /// 劣化中
    Degrading,
    /// 交換推奨
    Poor,
    /// 不明または未サポート
    Unknown,
}

/// 電源プラン
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PowerPlan {
    /// 高パフォーマンス
    HighPerformance,
    /// バランス
    Balanced,
    /// 省電力
    PowerSaver,
    /// カスタム
    Custom(String),
}

impl fmt::Display for PowerPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PowerPlan::HighPerformance => write!(f, "パフォーマンス"),
            PowerPlan::Balanced => write!(f, "バランス"),
            PowerPlan::PowerSaver => write!(f, "省電力"),
            PowerPlan::Custom(ref name) => write!(f, "カスタム ({})", name),
        }
    }
}

/// バッテリー情報
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    /// バッテリーのID
    pub id: String,
    /// バッテリー名
    pub name: String,
    /// メーカー
    pub manufacturer: Option<String>,
    /// モデル
    pub model: Option<String>,
    /// シリアル番号
    pub serial_number: Option<String>,
    /// 技術（Li-ionなど）
    pub technology: Option<String>,
    /// 製造日
    pub manufacture_date: Option<chrono::NaiveDate>,
    /// 現在の容量（mWh）
    pub current_capacity: Option<u32>,
    /// 設計容量（mWh）
    pub design_capacity: Option<u32>,
    /// 最大容量（mWh）
    pub max_capacity: Option<u32>,
    /// 現在の充電レベル（%）
    pub level: u8,
    /// 充電状態
    pub charging_state: ChargingState,
    /// 充電サイクル回数
    pub cycle_count: Option<u32>,
    /// 健康状態
    pub health: BatteryHealth,
    /// 電圧（mV）
    pub voltage: Option<u32>,
    /// 現在の電流（mA）
    pub current: Option<i32>,
    /// 電力（mW）
    pub power: Option<i32>,
    /// 残り時間の推定（分）
    pub time_remaining: Option<u32>,
    /// 最後の更新時刻
    pub last_updated: Instant,
}

impl BatteryInfo {
    /// 新しいバッテリー情報を作成
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            manufacturer: None,
            model: None,
            serial_number: None,
            technology: None,
            manufacture_date: None,
            current_capacity: None,
            design_capacity: None,
            max_capacity: None,
            level: 0,
            charging_state: ChargingState::Unknown,
            cycle_count: None,
            health: BatteryHealth::Unknown,
            voltage: None,
            current: None,
            power: None,
            time_remaining: None,
            last_updated: Instant::now(),
        }
    }
    
    /// 健康状態のパーセンテージを取得（設計容量に対する現在の容量の割合）
    pub fn health_percentage(&self) -> Option<u8> {
        match (self.current_capacity, self.design_capacity) {
            (Some(current), Some(design)) if design > 0 => {
                let percentage = (current as f64 / design as f64 * 100.0).round() as u8;
                Some(percentage.min(100))
            },
            _ => None,
        }
    }
    
    /// バッテリーの健康状態を計算して設定
    pub fn update_health(&mut self) {
        let health = match self.health_percentage() {
            Some(percentage) if percentage >= 80 => BatteryHealth::Good,
            Some(percentage) if percentage >= 50 => BatteryHealth::Degrading,
            Some(_) => BatteryHealth::Poor,
            None => BatteryHealth::Unknown,
        };
        self.health = health;
    }
    
    /// バッテリーが危険なレベルか（通常20%以下）
    pub fn is_low(&self) -> bool {
        self.level <= 20 && self.charging_state == ChargingState::Discharging
    }
    
    /// バッテリーが非常に危険なレベルか（通常5%以下）
    pub fn is_critical(&self) -> bool {
        self.level <= 5 && self.charging_state == ChargingState::Discharging
    }
}

/// UPS情報
#[derive(Debug, Clone)]
pub struct UPSInfo {
    /// UPSのID
    pub id: String,
    /// UPS名
    pub name: String,
    /// メーカー
    pub manufacturer: Option<String>,
    /// モデル
    pub model: Option<String>,
    /// 現在のバッテリーレベル（%）
    pub battery_level: u8,
    /// AC電源が接続されているか
    pub on_line: bool,
    /// UPSがバッテリーで動作しているか
    pub on_battery: bool,
    /// バッテリーの残り時間（分）
    pub time_remaining: Option<u32>,
    /// 負荷（%）
    pub load_percentage: Option<u8>,
    /// 最後の更新時刻
    pub last_updated: Instant,
}

/// 電源インターフェースの設定
#[derive(Debug, Clone)]
pub struct PowerInterfaceConfig {
    /// バッテリー監視の間隔（ミリ秒）
    pub battery_poll_interval_ms: u64,
    /// UPS監視の間隔（ミリ秒）
    pub ups_poll_interval_ms: u64,
    /// 低バッテリー警告のしきい値（%）
    pub low_battery_threshold: u8,
    /// 危険なバッテリーレベルのしきい値（%）
    pub critical_battery_threshold: u8,
    /// 自動的に省電力モードを有効にするバッテリーレベル（%）
    pub auto_power_saving_threshold: Option<u8>,
    /// 危険なバッテリーレベルでの自動アクション
    pub critical_battery_action: CriticalBatteryAction,
}

impl Default for PowerInterfaceConfig {
    fn default() -> Self {
        Self {
            battery_poll_interval_ms: 10000, // 10秒
            ups_poll_interval_ms: 30000,     // 30秒
            low_battery_threshold: 20,
            critical_battery_threshold: 5,
            auto_power_saving_threshold: Some(30),
            critical_battery_action: CriticalBatteryAction::Hibernate,
        }
    }
}

/// 危険なバッテリーレベルでの自動アクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriticalBatteryAction {
    /// 何もしない
    None,
    /// 通知のみ
    NotifyOnly,
    /// スリープ
    Sleep,
    /// 休止状態
    Hibernate,
    /// シャットダウン
    Shutdown,
}

/// 電源インターフェース
pub struct PowerInterface {
    /// 現在の電源の種類
    current_power_source: RwLock<PowerSource>,
    
    /// バッテリー情報のマップ
    batteries: RwLock<HashMap<String, BatteryInfo>>,
    
    /// UPS情報のマップ
    ups_devices: RwLock<HashMap<String, UPSInfo>>,
    
    /// 現在の電源プラン
    current_power_plan: RwLock<PowerPlan>,
    
    /// 省電力モードが有効か
    power_saving_enabled: RwLock<bool>,
    
    /// 設定
    config: RwLock<PowerInterfaceConfig>,
    
    /// モニタリングが有効か
    monitoring_enabled: Mutex<bool>,
    
    /// イベントハンドラー
    event_handlers: Mutex<Vec<Box<dyn Fn(&events::PowerEvent) + Send + Sync>>>,
    
    /// プラットフォーム固有の実装
    #[cfg(target_os = "linux")]
    platform: linux::LinuxPowerInterface,
    #[cfg(target_os = "macos")]
    platform: macos::MacOSPowerInterface,
    #[cfg(target_os = "windows")]
    platform: windows::WindowsPowerInterface,
}

impl PowerInterface {
    /// 新しい電源インターフェースを作成
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        let platform = linux::LinuxPowerInterface::new();
        #[cfg(target_os = "macos")]
        let platform = macos::MacOSPowerInterface::new();
        #[cfg(target_os = "windows")]
        let platform = windows::WindowsPowerInterface::new();
        
        Self {
            current_power_source: RwLock::new(PowerSource::Unknown),
            batteries: RwLock::new(HashMap::new()),
            ups_devices: RwLock::new(HashMap::new()),
            current_power_plan: RwLock::new(PowerPlan::Balanced),
            power_saving_enabled: RwLock::new(false),
            config: RwLock::new(PowerInterfaceConfig::default()),
            monitoring_enabled: Mutex::new(false),
            event_handlers: Mutex::new(Vec::new()),
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            platform,
        }
    }
    
    /// 監視を開始
    pub fn start_monitoring(&self) -> Result<(), String> {
        {
            let mut monitoring = self.monitoring_enabled.lock().unwrap();
            if *monitoring {
                return Ok(());
            }
            *monitoring = true;
        }
        
        // 初期状態を取得
        self.refresh_power_source();
        self.refresh_batteries();
        self.refresh_ups_devices();
        self.refresh_power_plan();
        
        // プラットフォーム固有の初期化
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        self.platform.initialize();
        
        info!("電源インターフェースの監視を開始しました");
        Ok(())
    }
    
    /// 監視を停止
    pub fn stop_monitoring(&self) -> Result<(), String> {
        {
            let mut monitoring = self.monitoring_enabled.lock().unwrap();
            if !*monitoring {
                return Ok(());
            }
            *monitoring = false;
        }
        
        // プラットフォーム固有のクリーンアップ
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        self.platform.cleanup();
        
        info!("電源インターフェースの監視を停止しました");
        Ok(())
    }
    
    /// 電源の種類を更新
    pub fn refresh_power_source(&self) -> PowerSource {
        let new_source = self.get_current_power_source();
        let old_source = *self.current_power_source.read().unwrap();
        
        if new_source != old_source {
            // 電源の変更を検出
            *self.current_power_source.write().unwrap() = new_source;
            
            // イベントを発行
            let event = events::PowerEvent::PowerSourceChanged {
                old: old_source,
                new: new_source,
            };
            self.emit_power_event(&event);
            
            // 電源プランを更新
            self.update_power_plan_based_on_source(new_source);
            
            // AC電源の接続/切断イベントを発行
            match (old_source, new_source) {
                (PowerSource::Battery, PowerSource::AC) => {
                    self.emit_power_event(&events::PowerEvent::ACAdapterConnected);
                },
                (PowerSource::AC, PowerSource::Battery) => {
                    self.emit_power_event(&events::PowerEvent::ACAdapterDisconnected);
                },
                _ => {}
            }
        }
        
        new_source
    }
    
    /// 現在の電源の種類を取得
    pub fn get_current_power_source(&self) -> PowerSource {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let source = self.platform.get_power_source();
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let source = PowerSource::Unknown;
        
        source
    }
    
    /// 電源の種類に基づいて電源プランを更新
    fn update_power_plan_based_on_source(&self, source: PowerSource) {
        match source {
            PowerSource::AC => {
                // AC電源の場合、バランスプランに設定
                self.set_power_plan(PowerPlan::Balanced);
            },
            PowerSource::Battery => {
                // バッテリーの場合、省電力プランに設定
                self.set_power_plan(PowerPlan::PowerSaver);
                
                // 自動省電力モードを有効化（設定されている場合）
                let config = self.config.read().unwrap();
                if let Some(threshold) = config.auto_power_saving_threshold {
                    // いずれかのバッテリーがしきい値を下回っているか確認
                    let batteries = self.batteries.read().unwrap();
                    let should_enable = batteries.values().any(|bat| bat.level <= threshold);
                    
                    if should_enable && !*self.power_saving_enabled.read().unwrap() {
                        self.set_power_saving_enabled(true);
                    }
                }
            },
            _ => {}
        }
    }
    
    /// バッテリー情報を更新
    pub fn refresh_batteries(&self) -> HashMap<String, BatteryInfo> {
        // 既存のバッテリー情報を保存
        let old_batteries = self.batteries.read().unwrap().clone();
        
        // 新しいバッテリー情報を取得
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let new_batteries = self.platform.get_batteries();
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let new_batteries = HashMap::new();
        
        // バッテリー情報を更新
        {
            let mut batteries = self.batteries.write().unwrap();
            *batteries = new_batteries.clone();
        }
        
        // イベントを発行
        self.detect_battery_events(&old_batteries, &new_batteries);
        
        new_batteries
    }
    
    /// バッテリーイベントを検出して発行
    fn detect_battery_events(&self, old_batteries: &HashMap<String, BatteryInfo>, new_batteries: &HashMap<String, BatteryInfo>) {
        // 設定を取得
        let config = self.config.read().unwrap();
        
        // 新しいバッテリーの検出
        for (id, battery) in new_batteries {
            if !old_batteries.contains_key(id) {
                // 新しいバッテリーが接続された
                let event = events::PowerEvent::BatteryConnected {
                    battery_id: id.clone(),
                    name: battery.name.clone(),
                };
                self.emit_power_event(&event);
                continue;
            }
            
            let old_battery = &old_batteries[id];
            
            // バッテリーレベルの変化
            if battery.level != old_battery.level {
                let event = events::PowerEvent::BatteryLevelChanged {
                    battery_id: id.clone(),
                    old: old_battery.level,
                    new: battery.level,
                };
                self.emit_power_event(&event);
                
                // 低バッテリー警告
                if battery.level <= config.low_battery_threshold && old_battery.level > config.low_battery_threshold {
                    let event = events::PowerEvent::LowBatteryWarning {
                        battery_id: id.clone(),
                        level: battery.level,
                    };
                    self.emit_power_event(&event);
                }
                
                // 危険なバッテリーレベル警告
                if battery.level <= config.critical_battery_threshold && old_battery.level > config.critical_battery_threshold {
                    let event = events::PowerEvent::CriticalBatteryWarning {
                        battery_id: id.clone(),
                        level: battery.level,
                    };
                    self.emit_power_event(&event);
                    
                    // 危険なバッテリーレベルでのアクション
                    self.handle_critical_battery(config.critical_battery_action);
                }
                
                // 満充電の検出
                if battery.charging_state == ChargingState::Full && old_battery.charging_state != ChargingState::Full {
                    let event = events::PowerEvent::BatteryFullyCharged {
                        battery_id: id.clone(),
                    };
                    self.emit_power_event(&event);
                }
            }
            
            // 充電状態の変化
            if battery.charging_state != old_battery.charging_state {
                let event = events::PowerEvent::ChargingStateChanged {
                    battery_id: id.clone(),
                    old: old_battery.charging_state,
                    new: battery.charging_state,
                };
                self.emit_power_event(&event);
            }
            
            // 健康状態の変化
            if battery.health != old_battery.health {
                let event = events::PowerEvent::BatteryHealthChanged {
                    battery_id: id.clone(),
                    old: old_battery.health,
                    new: battery.health,
                };
                self.emit_power_event(&event);
            }
        }
        
        // 切断されたバッテリーの検出
        for id in old_batteries.keys() {
            if !new_batteries.contains_key(id) {
                // バッテリーが切断された
                let event = events::PowerEvent::BatteryDisconnected {
                    battery_id: id.clone(),
                };
                self.emit_power_event(&event);
            }
        }
    }
    
    /// 危険なバッテリーレベルでのアクション
    fn handle_critical_battery(&self, action: CriticalBatteryAction) {
        match action {
            CriticalBatteryAction::None => {
                // 何もしない
            },
            CriticalBatteryAction::NotifyOnly => {
                // 通知は既に発行済み
            },
            CriticalBatteryAction::Sleep => {
                info!("危険なバッテリーレベルのため、システムをスリープ状態にします");
                self.system_sleep();
            },
            CriticalBatteryAction::Hibernate => {
                info!("危険なバッテリーレベルのため、システムを休止状態にします");
                self.system_hibernate();
            },
            CriticalBatteryAction::Shutdown => {
                info!("危険なバッテリーレベルのため、システムをシャットダウンします");
                self.system_shutdown();
            },
        }
    }
    
    /// UPS情報を更新
    pub fn refresh_ups_devices(&self) -> HashMap<String, UPSInfo> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let ups_devices = self.platform.get_ups_devices();
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let ups_devices = HashMap::new();
        
        // UPS情報を更新
        {
            let mut devices = self.ups_devices.write().unwrap();
            *devices = ups_devices.clone();
        }
        
        ups_devices
    }
    
    /// 電源プランを取得
    pub fn get_power_plan(&self) -> PowerPlan {
        self.current_power_plan.read().unwrap().clone()
    }
    
    /// 電源プランを設定
    pub fn set_power_plan(&self, plan: PowerPlan) -> bool {
        let old_plan = self.current_power_plan.read().unwrap().clone();
        if old_plan == plan {
            return true;
        }
        
        // プラットフォーム固有の実装
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let success = self.platform.set_power_plan(&plan);
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let success = true;
        
        if success {
            // 電源プランを更新
            *self.current_power_plan.write().unwrap() = plan.clone();
            
            // イベントを発行
            let event = events::PowerEvent::PowerPlanChanged {
                old: old_plan,
                new: plan,
            };
            self.emit_power_event(&event);
        }
        
        success
    }
    
    /// 省電力モードが有効か
    pub fn is_power_saving_enabled(&self) -> bool {
        *self.power_saving_enabled.read().unwrap()
    }
    
    /// 省電力モードを設定
    pub fn set_power_saving_enabled(&self, enabled: bool) -> bool {
        let was_enabled = *self.power_saving_enabled.read().unwrap();
        if was_enabled == enabled {
            return true;
        }
        
        // プラットフォーム固有の実装
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let success = self.platform.set_power_saving_enabled(enabled);
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let success = true;
        
        if success {
            *self.power_saving_enabled.write().unwrap() = enabled;
        }
        
        success
    }
    
    /// バッテリー情報を取得
    pub fn get_battery_info(&self, battery_id: &str) -> Option<BatteryInfo> {
        let batteries = self.batteries.read().unwrap();
        batteries.get(battery_id).cloned()
    }
    
    /// すべてのバッテリー情報を取得
    pub fn get_all_batteries(&self) -> HashMap<String, BatteryInfo> {
        self.batteries.read().unwrap().clone()
    }
    
    /// UPS情報を取得
    pub fn get_ups_info(&self, ups_id: &str) -> Option<UPSInfo> {
        let ups_devices = self.ups_devices.read().unwrap();
        ups_devices.get(ups_id).cloned()
    }
    
    /// すべてのUPS情報を取得
    pub fn get_all_ups_devices(&self) -> HashMap<String, UPSInfo> {
        self.ups_devices.read().unwrap().clone()
    }
    
    /// 設定を取得
    pub fn get_config(&self) -> PowerInterfaceConfig {
        self.config.read().unwrap().clone()
    }
    
    /// 設定を更新
    pub fn set_config(&self, config: PowerInterfaceConfig) {
        *self.config.write().unwrap() = config;
    }
    
    /// システムをスリープ状態にする
    pub fn system_sleep(&self) -> bool {
        debug!("システムをスリープ状態にします");
        
        // スリープ前イベントを発行
        self.emit_power_event(&events::PowerEvent::SystemSleepPending);
        
        // プラットフォーム固有の実装
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let success = self.platform.system_sleep();
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let success = false;
        
        if !success {
            error!("システムをスリープ状態にできませんでした");
        }
        
        success
    }
    
    /// システムを休止状態にする
    pub fn system_hibernate(&self) -> bool {
        debug!("システムを休止状態にします");
        
        // 休止前イベントを発行
        self.emit_power_event(&events::PowerEvent::SystemHibernatePending);
        
        // プラットフォーム固有の実装
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let success = self.platform.system_hibernate();
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let success = false;
        
        if !success {
            error!("システムを休止状態にできませんでした");
        }
        
        success
    }
    
    /// システムをシャットダウンする
    pub fn system_shutdown(&self) -> bool {
        debug!("システムをシャットダウンします");
        
        // シャットダウン前イベントを発行
        self.emit_power_event(&events::PowerEvent::SystemShutdownPending);
        
        // プラットフォーム固有の実装
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let success = self.platform.system_shutdown();
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let success = false;
        
        if !success {
            error!("システムをシャットダウンできませんでした");
        }
        
        success
    }
    
    /// システムを再起動する
    pub fn system_reboot(&self) -> bool {
        debug!("システムを再起動します");
        
        // 再起動前イベントを発行
        self.emit_power_event(&events::PowerEvent::SystemRebootPending);
        
        // プラットフォーム固有の実装
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let success = self.platform.system_reboot();
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let success = false;
        
        if !success {
            error!("システムを再起動できませんでした");
        }
        
        success
    }
    
    /// 電源イベントリスナーを登録
    pub fn register_event_handler<F>(&self, handler: F)
    where
        F: Fn(&events::PowerEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.lock().unwrap();
        handlers.push(Box::new(handler));
    }
    
    /// 電源イベントを発行
    fn emit_power_event(&self, event: &events::PowerEvent) {
        let handlers = self.event_handlers.lock().unwrap();
        for handler in &*handlers {
            handler(event);
        }
    }
}

impl Default for PowerInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    
    #[test]
    fn test_battery_info() {
        let mut battery = BatteryInfo::new("BAT0", "メインバッテリー");
        battery.level = 75;
        battery.charging_state = ChargingState::Discharging;
        battery.design_capacity = Some(50000);
        battery.current_capacity = Some(45000);
        
        // 健康状態の計算
        battery.update_health();
        assert_eq!(battery.health, BatteryHealth::Good);
        assert_eq!(battery.health_percentage(), Some(90));
        
        // バッテリーレベル判定
        assert!(!battery.is_low());
        assert!(!battery.is_critical());
        
        battery.level = 15;
        assert!(battery.is_low());
        assert!(!battery.is_critical());
        
        battery.level = 3;
        assert!(battery.is_low());
        assert!(battery.is_critical());
        
        // 充電中の場合は警告しない
        battery.charging_state = ChargingState::Charging;
        assert!(!battery.is_low());
        assert!(!battery.is_critical());
    }
    
    #[test]
    fn test_power_interface_events() {
        let power = PowerInterface::new();
        
        // イベントリスナーを設定
        let (tx, rx) = mpsc::channel();
        power.register_event_handler(move |event| {
            if let events::PowerEvent::PowerSourceChanged { .. } = event {
                tx.send(true).unwrap();
            }
        });
        
        // 電源変更イベントを発行
        let event = events::PowerEvent::PowerSourceChanged {
            old: PowerSource::Battery,
            new: PowerSource::AC,
        };
        power.emit_power_event(&event);
        
        // イベントが発火されたことを確認
        assert!(rx.recv().unwrap());
    }
} 