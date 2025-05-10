// LumosDesktop ハードウェアモニターモジュール
// システムのハードウェアリソースの使用状況とパフォーマンスを監視します

//! # ハードウェアモニターモジュール
//!
//! このモジュールはシステムのハードウェアリソースの使用状況とパフォーマンスを
//! リアルタイムで監視するための機能を提供します。
//!
//! 主な機能：
//! - CPU使用率とコア温度の監視
//! - メモリ使用状況の追跡
//! - GPU使用率と温度の監視
//! - ストレージデバイスの容量と健康状態の監視
//! - ネットワークインターフェースの帯域使用率の追跡
//! - バッテリー状態（モバイルデバイス用）の監視
//! - センサー情報（温度、ファン速度など）の取得
//!
//! これらの情報は、パフォーマンスの最適化、省電力モードの制御、
//! そしてユーザーへのシステム状態の通知に使用されます。

pub mod cpu_monitor;
pub mod memory_monitor;
pub mod disk_monitor;
pub mod gpu_monitor;
pub mod network_monitor;
pub mod thermal_monitor;
pub mod battery_monitor;
pub mod sensor_monitor;
pub mod resource_alert;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;

// これらのモジュールから主要な型をre-export
pub use cpu_monitor::{CpuInfo, CpuUsage, CpuFrequency};
pub use memory_monitor::{MemoryInfo, MemoryUsage};
pub use disk_monitor::{DiskInfo, DiskUsage};
pub use gpu_monitor::{GpuInfo, GpuUsage};
pub use network_monitor::{NetworkInfo, NetworkUsage};
pub use thermal_monitor::{ThermalZone, ThermalStatus, CoolingPolicy};
pub use battery_monitor::{BatteryInfo, BatteryStatus, PowerSource};
pub use sensor_monitor::{SensorType, SensorInfo, SensorReading};
pub use resource_alert::{ResourceAlert, AlertLevel, AlertType};

/// データポイント - 時系列データの1ポイント
#[derive(Debug, Clone)]
pub struct DataPoint {
    /// データ取得時刻
    pub timestamp: Instant,
    /// データラベル（デバイス名/識別子）
    pub label: String,
    /// データ値
    pub value: f64,
}

/// 履歴データタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HistoryDataType {
    /// CPU使用率
    CpuUsage,
    /// CPU周波数
    CpuFrequency,
    /// CPU温度
    CpuTemperature,
    /// メモリ使用率
    MemoryUsage,
    /// メモリ空き容量
    MemoryFree,
    /// スワップ使用率
    SwapUsage,
    /// ディスク使用率
    DiskUsage,
    /// ディスク空き容量
    DiskFreeSpace,
    /// ディスク読み取りIO
    DiskIORead,
    /// ディスク書き込みIO
    DiskIOWrite,
    /// ディスク健全性
    DiskHealth,
    /// GPU使用率
    GpuUsage,
    /// GPU温度
    GpuTemperature,
    /// GPU VRAM使用率
    GpuVramUsage,
    /// ネットワーク受信レート
    NetworkRxRate,
    /// ネットワーク送信レート
    NetworkTxRate,
    /// バッテリー残量
    BatteryLevel,
    /// バッテリー放電率
    BatteryDischargeRate,
    /// センサー値
    SensorValue,
}

/// モニタリングデータ
#[derive(Debug, Default)]
pub struct MonitoringData {
    /// CPU使用率データポイント
    pub cpu_usage: Vec<DataPoint>,
    /// CPU周波数データポイント
    pub cpu_frequency: Vec<DataPoint>,
    /// CPU温度データポイント
    pub cpu_temperature: Vec<DataPoint>,
    /// メモリ使用率データポイント
    pub memory_usage: Vec<DataPoint>,
    /// メモリ空き容量データポイント
    pub memory_free: Vec<DataPoint>,
    /// スワップ使用率データポイント
    pub swap_usage: Vec<DataPoint>,
    /// ディスク使用率データポイント
    pub disk_usage: Vec<DataPoint>,
    /// ディスク空き容量データポイント
    pub disk_free_space: Vec<DataPoint>,
    /// ディスク読み取りIOデータポイント
    pub disk_io_read: Vec<DataPoint>,
    /// ディスク書き込みIOデータポイント
    pub disk_io_write: Vec<DataPoint>,
    /// ディスク健全性データポイント
    pub disk_health: Vec<DataPoint>,
    /// GPU使用率データポイント
    pub gpu_usage: Vec<DataPoint>,
    /// GPU温度データポイント
    pub gpu_temperature: Vec<DataPoint>,
    /// GPU VRAM使用率データポイント
    pub gpu_vram_usage: Vec<DataPoint>,
    /// ネットワーク受信レートデータポイント
    pub network_rx_rate: Vec<DataPoint>,
    /// ネットワーク送信レートデータポイント
    pub network_tx_rate: Vec<DataPoint>,
    /// バッテリー残量データポイント
    pub battery_level: Vec<DataPoint>,
    /// バッテリー放電率データポイント
    pub battery_discharge_rate: Vec<DataPoint>,
    /// センサー値データポイント
    pub sensor_values: Vec<DataPoint>,
}

/// ハードウェアモニターの構成
#[derive(Debug, Clone)]
pub struct HardwareMonitorConfig {
    /// 全体の監視間隔（ミリ秒）
    pub interval_ms: u64,
    /// 履歴データ保持期間（分）
    pub history_retention_minutes: u64,
    /// CPU監視有効
    pub enable_cpu_monitoring: bool,
    /// メモリ監視有効
    pub enable_memory_monitoring: bool,
    /// ディスク監視有効
    pub enable_disk_monitoring: bool,
    /// GPU監視有効
    pub enable_gpu_monitoring: bool,
    /// ネットワーク監視有効
    pub enable_network_monitoring: bool,
    /// 熱監視有効
    pub enable_thermal_monitoring: bool,
    /// バッテリー監視有効
    pub enable_battery_monitoring: bool,
    /// センサー監視有効
    pub enable_sensor_monitoring: bool,
}

impl Default for HardwareMonitorConfig {
    fn default() -> Self {
        Self {
            interval_ms: 3000,             // 3秒間隔
            history_retention_minutes: 30, // 30分間の履歴保持
            enable_cpu_monitoring: true,
            enable_memory_monitoring: true,
            enable_disk_monitoring: true,
            enable_gpu_monitoring: true,
            enable_network_monitoring: true,
            enable_thermal_monitoring: true,
            enable_battery_monitoring: true,
            enable_sensor_monitoring: false, // デフォルトでは無効
        }
    }
}

/// ハードウェアモニターマネージャ
pub struct HardwareMonitor {
    /// 設定
    config: HardwareMonitorConfig,
    /// 現在のデータ
    current_data: Arc<Mutex<MonitoringData>>,
    /// 履歴データ
    history: Arc<Mutex<HashMap<HistoryDataType, Vec<DataPoint>>>>,
    /// モニタリングスレッドが実行中かどうか
    running: Arc<Mutex<bool>>,
    /// アラートコールバック
    alert_callbacks: Arc<Mutex<Vec<AlertCallback>>>,
    /// イベントチャネル送信側
    event_sender: Option<crossbeam_channel::Sender<ResourceAlert>>,
    /// イベントチャネル受信側
    event_receiver: Option<crossbeam_channel::Receiver<ResourceAlert>>,
}

impl HardwareMonitor {
    /// 新しいハードウェアモニターを作成
    pub fn new() -> Self {
        Self::with_config(HardwareMonitorConfig::default())
    }

    /// 設定を指定してハードウェアモニターを作成
    pub fn with_config(config: HardwareMonitorConfig) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        
        Self {
            config,
            current_data: Arc::new(Mutex::new(MonitoringData::default())),
            history: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
            alert_callbacks: Arc::new(Mutex::new(Vec::new())),
            event_sender: Some(sender),
            event_receiver: Some(receiver),
        }
    }

    /// モニタリングを開始
    pub fn start(&mut self) -> Result<(), String> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err("モニタリングはすでに実行中です".to_string());
        }
        
        *running = true;
        drop(running);
        
        // 各監視モジュールのスレッドを起動
        self.start_cpu_monitoring()?;
        self.start_memory_monitoring()?;
        self.start_gpu_monitoring()?;
        self.start_storage_monitoring()?;
        self.start_network_monitoring()?;
        self.start_battery_monitoring()?;
        self.start_sensor_monitoring()?;
        
        // 履歴データクリーンアップスレッドを開始
        self.start_history_cleanup();
        
        // アラート処理スレッドを開始
        self.start_alert_processor();
        
        Ok(())
    }

    /// モニタリングを停止
    pub fn stop(&self) -> Result<(), String> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err("モニタリングは実行されていません".to_string());
        }
        
        *running = false;
        Ok(())
    }

    /// 現在のモニタリングデータを取得
    pub fn get_current_data(&self) -> MonitoringData {
        self.current_data.lock().unwrap().clone()
    }

    /// 履歴データを取得
    pub fn get_history_data(&self, data_type: HistoryDataType) -> Vec<DataPoint> {
        let history = self.history.lock().unwrap();
        history.get(&data_type).cloned().unwrap_or_default()
    }

    /// リソースアラートコールバックを登録
    pub fn register_alert_callback<F>(&self, callback: F)
    where
        F: Fn(&ResourceAlert) + Send + Sync + 'static,
    {
        let mut callbacks = self.alert_callbacks.lock().unwrap();
        callbacks.push(Box::new(callback));
    }

    /// CPU監視スレッドを開始
    fn start_cpu_monitoring(&self) -> Result<(), String> {
        let interval = self.config.interval_ms;
        let current_data = Arc::clone(&self.current_data);
        let history = Arc::clone(&self.history);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        let sender = self.event_sender.clone();
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                // CPUデータを収集
                if let Ok(cpu_info) = cpu_monitor::collect_cpu_info() {
                    // 現在のデータを更新
                    let mut data = current_data.lock().unwrap();
                    data.cpu_usage.push(DataPoint {
                        timestamp: Instant::now(),
                        label: cpu_info.cpu_name.clone(),
                        value: cpu_info.usage.total,
                    });
                    data.cpu_frequency.push(DataPoint {
                        timestamp: Instant::now(),
                        label: cpu_info.cpu_name.clone(),
                        value: cpu_info.frequency,
                    });
                    data.cpu_temperature.push(DataPoint {
                        timestamp: Instant::now(),
                        label: cpu_info.cpu_name.clone(),
                        value: cpu_info.temperature.unwrap_or(0.0),
                    });
                    drop(data);
                    
                    // 履歴データを更新
                    let mut history_data = history.lock().unwrap();
                    let timestamp = Instant::now().elapsed().as_secs_f64();
                    
                    // CPU使用率履歴を更新
                    let cpu_usage_history = history_data.entry(HistoryDataType::CpuUsage)
                        .or_insert_with(Vec::new);
                    cpu_usage_history.push(DataPoint {
                        timestamp,
                        label: cpu_info.cpu_name.clone(),
                        value: cpu_info.usage.total,
                    });
                    
                    // CPU温度履歴を更新
                    if let Some(temp) = cpu_info.temperature {
                        let cpu_temp_history = history_data.entry(HistoryDataType::CpuTemperature)
                            .or_insert_with(Vec::new);
                        cpu_temp_history.push(DataPoint {
                            timestamp,
                            label: cpu_info.cpu_name.clone(),
                            value: temp,
                        });
                        
                        // 温度アラートをチェック
                        if config.enable_thermal_monitoring && temp > config.temperature_threshold {
                            if let Some(sender) = &sender {
                                let alert = ResourceAlert::new(
                                    AlertType::HighTemperature,
                                    AlertLevel::Warning,
                                    format!("CPU温度が閾値を超えています: {:.1}°C", temp),
                                );
                                let _ = sender.send(alert);
                            }
                        }
                    }
                    
                    // CPUアラートをチェック
                    if config.enable_cpu_monitoring && cpu_info.usage.total > config.cpu_usage_threshold {
                        if let Some(sender) = &sender {
                            let alert = ResourceAlert::new(
                                AlertType::HighCpuUsage,
                                AlertLevel::Warning,
                                format!("CPU使用率が閾値を超えています: {:.1}%", cpu_info.usage.total),
                            );
                            let _ = sender.send(alert);
                        }
                    }
                }
                
                // 指定された間隔でスリープ
                thread::sleep(Duration::from_millis(interval));
            }
        });
        
        Ok(())
    }

    /// メモリ監視スレッドを開始
    fn start_memory_monitoring(&self) -> Result<(), String> {
        let interval = self.config.interval_ms;
        let current_data = Arc::clone(&self.current_data);
        let history = Arc::clone(&self.history);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        let sender = self.event_sender.clone();
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                // メモリデータを収集
                if let Ok(memory_info) = memory_monitor::collect_memory_info() {
                    // 現在のデータを更新
                    let mut data = current_data.lock().unwrap();
                    data.memory_usage.push(DataPoint {
                        timestamp: Instant::now(),
                        label: memory_info.memory_device.clone(),
                        value: memory_info.usage.used_percent,
                    });
                    data.memory_free.push(DataPoint {
                        timestamp: Instant::now(),
                        label: memory_info.memory_device.clone(),
                        value: (100.0 - memory_info.usage.used_percent),
                    });
                    drop(data);
                    
                    // 履歴データを更新
                    let mut history_data = history.lock().unwrap();
                    let timestamp = Instant::now().elapsed().as_secs_f64();
                    
                    // メモリ使用率履歴を更新
                    let memory_usage_history = history_data.entry(HistoryDataType::MemoryUsage)
                        .or_insert_with(Vec::new);
                    memory_usage_history.push(DataPoint {
                        timestamp,
                        label: memory_info.memory_device.clone(),
                        value: memory_info.usage.used_percent,
                    });
                    
                    // メモリアラートをチェック
                    if config.enable_memory_monitoring && memory_info.usage.used_percent > config.memory_usage_threshold {
                        if let Some(sender) = &sender {
                            let alert = ResourceAlert::new(
                                AlertType::HighMemoryUsage,
                                AlertLevel::Warning,
                                format!("メモリ使用率が閾値を超えています: {:.1}%", memory_info.usage.used_percent),
                            );
                            let _ = sender.send(alert);
                        }
                    }
                }
                
                // 指定された間隔でスリープ
                thread::sleep(Duration::from_millis(interval));
            }
        });
        
        Ok(())
    }

    /// GPU監視スレッドを開始
    fn start_gpu_monitoring(&self) -> Result<(), String> {
        let interval = self.config.interval_ms;
        let current_data = Arc::clone(&self.current_data);
        let history = Arc::clone(&self.history);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        let sender = self.event_sender.clone();
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                // GPUデータを収集
                if let Ok(gpu_info) = gpu_monitor::collect_gpu_info() {
                    // 現在のデータを更新
                    let mut data = current_data.lock().unwrap();
                    data.gpu_usage.push(DataPoint {
                        timestamp: Instant::now(),
                        label: gpu_info.gpu_name.clone(),
                        value: gpu_info.usage.utilization,
                    });
                    data.gpu_temperature.push(DataPoint {
                        timestamp: Instant::now(),
                        label: gpu_info.gpu_name.clone(),
                        value: gpu_info.temperature.unwrap_or(0.0),
                    });
                    data.gpu_vram_usage.push(DataPoint {
                        timestamp: Instant::now(),
                        label: gpu_info.gpu_name.clone(),
                        value: gpu_info.vram_usage,
                    });
                    drop(data);
                    
                    // 履歴データを更新
                    let mut history_data = history.lock().unwrap();
                    let timestamp = Instant::now().elapsed().as_secs_f64();
                    
                    // GPU使用率履歴を更新
                    let gpu_usage_history = history_data.entry(HistoryDataType::GpuUsage)
                        .or_insert_with(Vec::new);
                    gpu_usage_history.push(DataPoint {
                        timestamp,
                        label: gpu_info.gpu_name.clone(),
                        value: gpu_info.usage.utilization,
                    });
                    
                    // GPU温度履歴を更新
                    if let Some(temp) = gpu_info.temperature {
                        let gpu_temp_history = history_data.entry(HistoryDataType::GpuTemperature)
                            .or_insert_with(Vec::new);
                        gpu_temp_history.push(DataPoint {
                            timestamp,
                            label: gpu_info.gpu_name.clone(),
                            value: temp,
                        });
                        
                        // 温度アラートをチェック
                        if config.enable_thermal_monitoring && temp > config.temperature_threshold {
                            if let Some(sender) = &sender {
                                let alert = ResourceAlert::new(
                                    AlertType::HighTemperature,
                                    AlertLevel::Warning,
                                    format!("GPU温度が閾値を超えています: {:.1}°C", temp),
                                );
                                let _ = sender.send(alert);
                            }
                        }
                    }
                }
                
                // 指定された間隔でスリープ
                thread::sleep(Duration::from_millis(interval));
            }
        });
        
        Ok(())
    }

    /// ストレージ監視スレッドを開始
    fn start_storage_monitoring(&self) -> Result<(), String> {
        let interval = self.config.interval_ms;
        let current_data = Arc::clone(&self.current_data);
        let history = Arc::clone(&self.history);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        let sender = self.event_sender.clone();
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                // ストレージデータを収集
                if let Ok(disk_info) = disk_monitor::collect_disk_info() {
                    // 現在のデータを更新
                    let mut data = current_data.lock().unwrap();
                    data.disk_usage.push(DataPoint {
                        timestamp: Instant::now(),
                        label: disk_info.device_name.clone(),
                        value: disk_info.usage.used_percent,
                    });
                    data.disk_free_space.push(DataPoint {
                        timestamp: Instant::now(),
                        label: disk_info.device_name.clone(),
                        value: (100.0 - disk_info.usage.used_percent),
                    });
                    data.disk_io_read.push(DataPoint {
                        timestamp: Instant::now(),
                        label: disk_info.device_name.clone(),
                        value: disk_info.io_read as f64,
                    });
                    data.disk_io_write.push(DataPoint {
                        timestamp: Instant::now(),
                        label: disk_info.device_name.clone(),
                        value: disk_info.io_write as f64,
                    });
                    data.disk_health.push(DataPoint {
                        timestamp: Instant::now(),
                        label: disk_info.device_name.clone(),
                        value: disk_info.health.map_or(0.0, |h| h.is_healthy as f64),
                    });
                    drop(data);
                    
                    // ストレージアラートをチェック
                    if config.enable_disk_monitoring {
                        if disk_info.usage.used_percent > config.disk_usage_threshold {
                            if let Some(sender) = &sender {
                                let alert = ResourceAlert::new(
                                    AlertType::HighDiskUsage,
                                    AlertLevel::Warning,
                                    format!("ディスク {} の使用率が閾値を超えています: {:.1}%", 
                                            disk_info.device_name, disk_info.usage.used_percent),
                                );
                                let _ = sender.send(alert);
                            }
                        }
                        if !disk_info.health.is_healthy {
                            if let Some(sender) = &sender {
                                let alert = ResourceAlert::new(
                                    AlertType::StorageHealth,
                                    AlertLevel::Critical,
                                    format!("ディスク {} の健康状態に問題があります", 
                                            disk_info.device_name),
                                );
                                let _ = sender.send(alert);
                            }
                        }
                    }
                }
                
                // 指定された間隔でスリープ
                thread::sleep(Duration::from_millis(interval));
            }
        });
        
        Ok(())
    }

    /// ネットワーク監視スレッドを開始
    fn start_network_monitoring(&self) -> Result<(), String> {
        let interval = self.config.interval_ms;
        let current_data = Arc::clone(&self.current_data);
        let history = Arc::clone(&self.history);
        let running = Arc::clone(&self.running);
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                // ネットワークデータを収集
                if let Ok(network_info) = network_monitor::collect_network_info() {
                    // 現在のデータを更新
                    let mut data = current_data.lock().unwrap();
                    data.network_rx_rate.push(DataPoint {
                        timestamp: Instant::now(),
                        label: network_info.primary_interface.clone(),
                        value: network_info.rx_rate,
                    });
                    data.network_tx_rate.push(DataPoint {
                        timestamp: Instant::now(),
                        label: network_info.primary_interface.clone(),
                        value: network_info.tx_rate,
                    });
                    drop(data);
                    
                    // 履歴データを更新（主要インターフェースのみ）
                    if let Some(primary_interface) = network_monitor::get_primary_interface() {
                        if let Some(interface_info) = network_info.get(&primary_interface) {
                            let mut history_data = history.lock().unwrap();
                            let timestamp = Instant::now().elapsed().as_secs_f64();
                            
                            // 帯域使用率履歴を更新
                            let total_bandwidth = interface_info.bandwidth_in + interface_info.bandwidth_out;
                            let network_usage_history = history_data.entry(HistoryDataType::NetworkRxRate)
                                .or_insert_with(Vec::new);
                            network_usage_history.push(DataPoint {
                                timestamp,
                                label: primary_interface.clone(),
                                value: interface_info.rx_rate as f64,
                            });
                            let network_usage_history = history_data.entry(HistoryDataType::NetworkTxRate)
                                .or_insert_with(Vec::new);
                            network_usage_history.push(DataPoint {
                                timestamp,
                                label: primary_interface.clone(),
                                value: interface_info.tx_rate as f64,
                            });
                        }
                    }
                }
                
                // 指定された間隔でスリープ
                thread::sleep(Duration::from_millis(interval));
            }
        });
        
        Ok(())
    }

    /// バッテリー監視スレッドを開始
    fn start_battery_monitoring(&self) -> Result<(), String> {
        let interval = self.config.interval_ms;
        let current_data = Arc::clone(&self.current_data);
        let history = Arc::clone(&self.history);
        let running = Arc::clone(&self.running);
        let sender = self.event_sender.clone();
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                // バッテリーデータを収集
                if let Ok(battery_info) = battery_monitor::collect_battery_info() {
                    // 現在のデータを更新
                    let mut data = current_data.lock().unwrap();
                    data.battery_level.push(DataPoint {
                        timestamp: Instant::now(),
                        label: battery_info.battery_name.clone(),
                        value: battery_info.percentage,
                    });
                    data.battery_discharge_rate.push(DataPoint {
                        timestamp: Instant::now(),
                        label: battery_info.battery_name.clone(),
                        value: battery_info.discharge_rate,
                    });
                    drop(data);
                    
                    // バッテリー状態が変化した場合にアラートを発行
                    if battery_info.power_source != PowerSource::Battery {
                        if let Some(sender) = &sender {
                            let message = match battery_info.power_source {
                                PowerSource::AC => "電源がACアダプターに切り替わりました",
                                PowerSource::Unknown => "電源状態が不明になりました",
                            };
                            
                            let alert = ResourceAlert::new(
                                AlertType::PowerSourceChanged,
                                AlertLevel::Info,
                                message.to_string(),
                            );
                            let _ = sender.send(alert);
                        }
                    }
                    
                    // バッテリー残量が低下した場合
                    if battery_info.percentage <= 20.0 {
                        if let Some(sender) = &sender {
                            let alert = ResourceAlert::new(
                                AlertType::LowBattery,
                                AlertLevel::Warning,
                                format!("バッテリー残量が低下しています: {:.1}%", battery_info.percentage),
                            );
                            let _ = sender.send(alert);
                        }
                    }
                    
                    // バッテリー残量が危機的な場合
                    if battery_info.percentage <= 5.0 {
                        if let Some(sender) = &sender {
                            let alert = ResourceAlert::new(
                                AlertType::CriticalBattery,
                                AlertLevel::Critical,
                                format!("バッテリー残量が危機的に低下しています: {:.1}%", battery_info.percentage),
                            );
                            let _ = sender.send(alert);
                        }
                    }
                }
                
                // 指定された間隔でスリープ
                thread::sleep(Duration::from_millis(interval));
            }
        });
        
        Ok(())
    }

    /// センサー監視スレッドを開始
    fn start_sensor_monitoring(&self) -> Result<(), String> {
        let interval = self.config.interval_ms;
        let current_data = Arc::clone(&self.current_data);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        let sender = self.event_sender.clone();
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                // センサーデータを収集
                if let Ok(sensor_readings) = sensor_monitor::collect_sensor_readings() {
                    // 現在のデータを更新
                    let mut data = current_data.lock().unwrap();
                    for reading in sensor_readings {
                        data.sensor_values.push(DataPoint {
                            timestamp: Instant::now(),
                            label: reading.sensor_id.clone(),
                            value: reading.value,
                        });
                    }
                    drop(data);
                    
                    // 温度センサーアラートをチェック
                    if config.enable_sensor_monitoring {
                        for reading in &sensor_readings {
                            if reading.sensor_type == SensorType::Temperature {
                                if reading.value > config.temperature_threshold {
                                    if let Some(sender) = &sender {
                                        let alert = ResourceAlert::new(
                                            AlertType::HighTemperature,
                                            AlertLevel::Warning,
                                            format!("センサー {} の温度が閾値を超えています: {:.1}°C", 
                                                    reading.sensor_id, reading.value),
                                        );
                                        let _ = sender.send(alert);
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 指定された間隔でスリープ
                thread::sleep(Duration::from_millis(interval));
            }
        });
        
        Ok(())
    }

    /// 履歴データのクリーンアップスレッドを開始
    fn start_history_cleanup(&self) {
        let history = Arc::clone(&self.history);
        let running = Arc::clone(&self.running);
        let retention_time = self.config.history_retention_minutes * 60;
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                // 30秒ごとにクリーンアップ
                thread::sleep(Duration::from_secs(30));
                
                let cutoff_time = Instant::now().elapsed().as_secs_f64() - retention_time as f64;
                let mut history_data = history.lock().unwrap();
                
                for (_, data_points) in history_data.iter_mut() {
                    // 古いデータポイントを削除
                    data_points.retain(|point| point.timestamp >= cutoff_time);
                }
            }
        });
    }

    /// アラート処理スレッドを開始
    fn start_alert_processor(&self) {
        if let Some(receiver) = self.event_receiver.clone() {
            let callbacks = Arc::clone(&self.alert_callbacks);
            let running = Arc::clone(&self.running);
            
            thread::spawn(move || {
                while *running.lock().unwrap() {
                    // アラートを受信
                    if let Ok(alert) = receiver.recv_timeout(Duration::from_millis(100)) {
                        // 登録されているすべてのコールバックを呼び出し
                        let callbacks_guard = callbacks.lock().unwrap();
                        for callback in callbacks_guard.iter() {
                            callback(&alert);
                        }
                    }
                }
            });
        }
    }
}

impl Drop for HardwareMonitor {
    fn drop(&mut self) {
        // インスタンスが破棄される際にモニタリングを停止
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    #[test]
    fn test_hardware_monitor_creation() {
        let monitor = HardwareMonitor::new();
        assert!(monitor.config.enable_cpu_monitoring);
        assert!(monitor.config.enable_memory_monitoring);
        assert!(monitor.config.enable_disk_monitoring);
        assert!(monitor.config.enable_gpu_monitoring);
        assert!(monitor.config.enable_network_monitoring);
        assert!(monitor.config.enable_thermal_monitoring);
        assert!(monitor.config.enable_battery_monitoring);
        assert!(monitor.config.enable_sensor_monitoring);
        assert_eq!(monitor.config.interval_ms, 3000);
    }
    
    #[test]
    fn test_hardware_monitor_custom_config() {
        let mut config = HardwareMonitorConfig::default();
        config.interval_ms = 5000;
        config.enable_cpu_monitoring = false;
        config.enable_memory_monitoring = false;
        config.enable_disk_monitoring = false;
        config.enable_gpu_monitoring = false;
        config.enable_network_monitoring = false;
        config.enable_thermal_monitoring = false;
        config.enable_battery_monitoring = false;
        config.enable_sensor_monitoring = true;
        
        let monitor = HardwareMonitor::with_config(config);
        assert!(!monitor.config.enable_cpu_monitoring);
        assert!(!monitor.config.enable_memory_monitoring);
        assert!(!monitor.config.enable_disk_monitoring);
        assert!(!monitor.config.enable_gpu_monitoring);
        assert!(!monitor.config.enable_network_monitoring);
        assert!(!monitor.config.enable_thermal_monitoring);
        assert!(!monitor.config.enable_battery_monitoring);
        assert!(monitor.config.enable_sensor_monitoring);
        assert_eq!(monitor.config.interval_ms, 5000);
    }
    
    #[test]
    fn test_alert_callback() {
        let monitor = HardwareMonitor::new();
        let alert_received = Arc::new(AtomicBool::new(false));
        
        let alert_received_clone = Arc::clone(&alert_received);
        monitor.register_alert_callback(move |_alert| {
            alert_received_clone.store(true, Ordering::SeqCst);
        });
        
        // アラートプロセッサーを手動で呼び出し
        if let Some(sender) = &monitor.event_sender {
            let alert = ResourceAlert::new(
                AlertType::HighCpuUsage,
                AlertLevel::Warning,
                "テストアラート".to_string(),
            );
            sender.send(alert).unwrap();
            
            // コールバックが実行される時間を確保
            thread::sleep(Duration::from_millis(100));
            
            assert!(alert_received.load(Ordering::SeqCst));
        }
    }
} 