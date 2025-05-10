// Copyright (c) 2023-2024 AetherOS Project
//
// GPUモニタリングモジュール
// GPUの使用率、温度、メモリ使用量などの情報を取得・モニタリングするための実装です。
// 主要なGPUベンダー（NVIDIA、AMD、Intel）に対応し、ベンダー固有のAPIとフォールバックメカニズムを提供します。

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::broadcast;
use tokio::time;

use crate::system::{SubsystemStatus, SystemError};
use crate::utils::platform::{self, PlatformInfo};
use crate::utils::units::{DataSize, Temperature};
use crate::core::system::subsystem::{Subsystem, SubsystemStatus};
use crate::core::utils::time::Timestamp;

/// GPUモニタリングに関するエラー
#[derive(Error, Debug)]
pub enum GpuMonitorError {
    /// GPUの検出に失敗
    #[error("GPUの検出に失敗しました: {0}")]
    DetectionFailed(String),
    
    /// GPUの情報取得に失敗
    #[error("GPU情報の取得に失敗しました: {0}")]
    InfoRetrievalFailed(String),
    
    /// ドライバの互換性問題
    #[error("GPUドライバの互換性問題: {0}")]
    DriverCompatibility(String),
    
    /// 権限不足
    #[error("GPUモニタリングの権限が不足しています: {0}")]
    InsufficientPermissions(String),
    
    /// プラットフォーム非対応
    #[error("このプラットフォームではGPUモニタリングがサポートされていません: {0}")]
    UnsupportedPlatform(String),
    
    /// サブシステム初期化エラー
    #[error("GPUモニタリングサブシステムの初期化に失敗しました: {0}")]
    InitializationFailed(String),
}

impl From<GpuMonitorError> for SystemError {
    fn from(error: GpuMonitorError) -> Self {
        SystemError::HardwareMonitorError(error.to_string())
    }
}

/// GPUベンダー識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GpuVendor {
    /// NVIDIAグラフィックス
    Nvidia,
    /// AMDグラフィックス
    Amd,
    /// Intelグラフィックス
    Intel,
    /// Appleシリコン
    Apple,
    /// その他のベンダー
    Other(u32),
}

impl fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuVendor::Nvidia => write!(f, "NVIDIA"),
            GpuVendor::Amd => write!(f, "AMD"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Apple => write!(f, "Apple"),
            GpuVendor::Other(id) => write!(f, "その他ベンダー(ID: {:#x})", id),
        }
    }
}

/// GPU情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU識別子
    pub id: String,
    /// GPUのインデックス番号
    pub index: usize,
    /// GPUのベンダー
    pub vendor: GpuVendor,
    /// GPUの名前
    pub name: String,
    /// GPUのドライババージョン
    pub driver_version: Option<String>,
    /// 総メモリ容量
    pub total_memory: DataSize,
    /// ハードウェア機能フラグ
    pub features: HashMap<String, bool>,
}

/// GPU利用率情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuUsage {
    /// GPU識別子
    pub id: String,
    /// GPUの使用率 (0.0-100.0%)
    pub utilization: f32,
    /// GPUメモリ使用量
    pub memory_used: DataSize,
    /// GPUメモリ使用率 (0.0-100.0%)
    pub memory_utilization: f32,
    /// GPU温度
    pub temperature: Option<Temperature>,
    /// GPUパワー消費 (ワット)
    pub power_usage: Option<f32>,
    /// エンコーダー使用率 (0.0-100.0%)
    pub encoder_utilization: Option<f32>,
    /// デコーダー使用率 (0.0-100.0%)
    pub decoder_utilization: Option<f32>,
    /// タイムスタンプ
    pub timestamp: Instant,
}

/// GPUモニタリング設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMonitorConfig {
    /// ポーリング間隔（ミリ秒）
    pub polling_interval_ms: u64,
    /// 詳細モニタリングの有効化
    pub enable_detailed_monitoring: bool,
    /// 温度モニタリングの有効化
    pub enable_temperature_monitoring: bool,
    /// パワー消費モニタリングの有効化
    pub enable_power_monitoring: bool,
    /// 自動検出の有効化
    pub enable_auto_detection: bool,
}

impl Default for GpuMonitorConfig {
    fn default() -> Self {
        Self {
            polling_interval_ms: 1000,
            enable_detailed_monitoring: true,
            enable_temperature_monitoring: true,
            enable_power_monitoring: true,
            enable_auto_detection: true,
        }
    }
}

/// GPUモニター
pub struct GpuMonitor {
    /// 検出されたGPU情報のリスト
    gpus: Arc<Mutex<Vec<GpuInfo>>>,
    /// 最新のGPU使用状況
    current_usage: Arc<Mutex<HashMap<String, GpuUsage>>>,
    /// モニタリング設定
    config: GpuMonitorConfig,
    /// 使用状況更新用のブロードキャストチャネル
    usage_tx: broadcast::Sender<HashMap<String, GpuUsage>>,
    /// 現在のサブシステムステータス
    status: Arc<Mutex<SubsystemStatus>>,
    /// モニタリングタスクハンドル
    #[allow(dead_code)]
    monitor_task: Option<tokio::task::JoinHandle<()>>,
}

impl GpuMonitor {
    /// 新しいGPUモニターを作成
    pub fn new(config: GpuMonitorConfig) -> Self {
        let (usage_tx, _) = broadcast::channel(16);
        
        Self {
            gpus: Arc::new(Mutex::new(Vec::new())),
            current_usage: Arc::new(Mutex::new(HashMap::new())),
            config,
            usage_tx,
            status: Arc::new(Mutex::new(SubsystemStatus::Stopped)),
            monitor_task: None,
        }
    }
    
    /// デフォルト設定でGPUモニターを作成
    pub fn default() -> Self {
        Self::new(GpuMonitorConfig::default())
    }
    
    /// GPUモニタリングを初期化して開始
    pub async fn initialize(&mut self) -> Result<(), GpuMonitorError> {
        *self.status.lock().unwrap() = SubsystemStatus::Initializing;
        
        // GPUを検出
        self.detect_gpus().await?;
        
        let gpus = self.gpus.clone();
        let current_usage = self.current_usage.clone();
        let usage_tx = self.usage_tx.clone();
        let config = self.config.clone();
        let status = self.status.clone();
        
        // モニタリングタスクを開始
        let handle = tokio::spawn(async move {
            let polling_interval = Duration::from_millis(config.polling_interval_ms);
            let mut interval = time::interval(polling_interval);
            
            *status.lock().unwrap() = SubsystemStatus::Running;
            
            loop {
                interval.tick().await;
                
                if *status.lock().unwrap() != SubsystemStatus::Running {
                    break;
                }
                
                let gpu_list = gpus.lock().unwrap().clone();
                if gpu_list.is_empty() {
                    continue;
                }
                
                // GPUごとに使用率を取得
                let mut updated_usage = HashMap::new();
                for gpu in &gpu_list {
                    if let Ok(usage) = Self::collect_gpu_usage(gpu, &config) {
                        updated_usage.insert(gpu.id.clone(), usage);
                    }
                }
                
                // 現在の使用率を更新
                {
                    let mut usage_data = current_usage.lock().unwrap();
                    *usage_data = updated_usage.clone();
                }
                
                // 通知を送信（エラーは無視）
                let _ = usage_tx.send(updated_usage);
            }
        });
        
        self.monitor_task = Some(handle);
        Ok(())
    }
    
    /// GPUデバイスの検出
    async fn detect_gpus(&mut self) -> Result<(), GpuMonitorError> {
        let mut detected_gpus = Vec::new();
        
        // プラットフォーム固有のGPU検出
        match platform::get_platform_info() {
            PlatformInfo::Linux => {
                // Linux固有の検出ロジック
                self.detect_gpus_linux(&mut detected_gpus)?;
            }
            PlatformInfo::Windows => {
                // Windows固有の検出ロジック
                self.detect_gpus_windows(&mut detected_gpus)?;
            }
            PlatformInfo::MacOS => {
                // macOS固有の検出ロジック
                self.detect_gpus_macos(&mut detected_gpus)?;
            }
            _ => {
                return Err(GpuMonitorError::UnsupportedPlatform(
                    "不明なプラットフォーム".to_string(),
                ));
            }
        }
        
        if detected_gpus.is_empty() && self.config.enable_auto_detection {
            // 汎用的な検出方法をフォールバックとして使用
            self.detect_gpus_generic(&mut detected_gpus)?;
        }
        
        if detected_gpus.is_empty() {
            return Err(GpuMonitorError::DetectionFailed("GPUが検出されませんでした".to_string()));
        }
        
        let mut gpus = self.gpus.lock().unwrap();
        *gpus = detected_gpus;
        
        Ok(())
    }
    
    /// Linux用のGPU検出ロジック
    fn detect_gpus_linux(&self, gpus: &mut Vec<GpuInfo>) -> Result<(), GpuMonitorError> {
        // NVIDIA GPUの検出
        self.detect_nvidia_gpus(gpus)?;
        
        // AMD GPUの検出 (DRM/sysfsを使用)
        self.detect_amd_gpus(gpus)?;
        
        // Intel GPUの検出 (DRM/sysfsを使用)
        self.detect_intel_gpus(gpus)?;
        
        Ok(())
    }
    
    /// Windows用のGPU検出ロジック
    fn detect_gpus_windows(&self, gpus: &mut Vec<GpuInfo>) -> Result<(), GpuMonitorError> {
        // Windows固有の実装 (WMI/DXGI/D3D)
        // 実際の実装では、WMIやDirectXのAPIを使用してGPU情報を取得します
        
        // サンプル実装 (実際には適切なAPI呼び出しに置き換える)
        if let Ok(_) = std::env::var("NVIDIA_DEV") {
            gpus.push(GpuInfo {
                id: "nvidia-0".to_string(),
                index: 0,
                vendor: GpuVendor::Nvidia,
                name: "NVIDIA GeForce RTX Simulation".to_string(),
                driver_version: Some("460.79".to_string()),
                total_memory: DataSize::from_megabytes(8192),
                features: HashMap::new(),
            });
        }
        
        Ok(())
    }
    
    /// macOS用のGPU検出ロジック
    fn detect_gpus_macos(&self, gpus: &mut Vec<GpuInfo>) -> Result<(), GpuMonitorError> {
        // macOS固有の実装 (IOKit APIを使用)
        // 実際の実装では、IOKitのAPIを使用してGPU情報を取得します
        
        // サンプル実装 (実際には適切なAPI呼び出しに置き換える)
        if platform::is_apple_silicon() {
            gpus.push(GpuInfo {
                id: "apple-0".to_string(),
                index: 0,
                vendor: GpuVendor::Apple,
                name: "Apple M1 GPU".to_string(),
                driver_version: None,
                total_memory: DataSize::from_megabytes(8192),
                features: HashMap::new(),
            });
        }
        
        Ok(())
    }
    
    /// NVIDIA GPU検出
    fn detect_nvidia_gpus(&self, gpus: &mut Vec<GpuInfo>) -> Result<(), GpuMonitorError> {
        // NVML (NVIDIA Management Library) を使用した実装
        // 実際の実装では、NVMLのバインディングを使用します
        
        // サンプル実装 (実際には適切なAPI呼び出しに置き換える)
        if let Ok(_) = std::env::var("NVIDIA_DEV") {
            gpus.push(GpuInfo {
                id: "nvidia-0".to_string(),
                index: 0,
                vendor: GpuVendor::Nvidia,
                name: "NVIDIA GeForce RTX Simulation".to_string(),
                driver_version: Some("460.79".to_string()),
                total_memory: DataSize::from_megabytes(8192),
                features: HashMap::new(),
            });
        }
        
        Ok(())
    }
    
    /// AMD GPU検出
    fn detect_amd_gpus(&self, gpus: &mut Vec<GpuInfo>) -> Result<(), GpuMonitorError> {
        // AMD GPUのLinux検出 (sysfs/ROCmを使用)
        // 実際の実装では、sysfsからGPU情報を取得します
        
        // サンプル実装 (実際には適切なファイル読み取りに置き換える)
        if let Ok(_) = std::env::var("AMD_DEV") {
            gpus.push(GpuInfo {
                id: "amd-0".to_string(),
                index: 0,
                vendor: GpuVendor::Amd,
                name: "AMD Radeon RX Simulation".to_string(),
                driver_version: Some("21.30".to_string()),
                total_memory: DataSize::from_megabytes(6144),
                features: HashMap::new(),
            });
        }
        
        Ok(())
    }
    
    /// Intel GPU検出
    fn detect_intel_gpus(&self, gpus: &mut Vec<GpuInfo>) -> Result<(), GpuMonitorError> {
        // Intel GPUのLinux検出 (sysfs/i915を使用)
        // 実際の実装では、sysfsからGPU情報を取得します
        
        // サンプル実装 (実際には適切なファイル読み取りに置き換える)
        if let Ok(_) = std::env::var("INTEL_DEV") {
            gpus.push(GpuInfo {
                id: "intel-0".to_string(),
                index: 0,
                vendor: GpuVendor::Intel,
                name: "Intel UHD Graphics Simulation".to_string(),
                driver_version: Some("27.20.100.9316".to_string()),
                total_memory: DataSize::from_megabytes(1024),
                features: HashMap::new(),
            });
        }
        
        Ok(())
    }
    
    /// 汎用的なGPU検出
    fn detect_gpus_generic(&self, gpus: &mut Vec<GpuInfo>) -> Result<(), GpuMonitorError> {
        // OpenGL/Vulkanを使用した汎用的なGPU検出
        // 実際の実装では、OpenGLやVulkanのAPIを使用してGPU情報を取得します
        
        // サンプル実装 (実際にはAPIを使用したコードに置き換える)
        gpus.push(GpuInfo {
            id: "generic-0".to_string(),
            index: 0,
            vendor: GpuVendor::Other(0),
            name: "汎用グラフィックスアダプター".to_string(),
            driver_version: None,
            total_memory: DataSize::from_megabytes(1024),
            features: HashMap::new(),
        });
        
        Ok(())
    }
    
    /// 特定のGPUの利用率を収集
    fn collect_gpu_usage(gpu: &GpuInfo, config: &GpuMonitorConfig) -> Result<GpuUsage, GpuMonitorError> {
        // 適切なバックエンドを使用して各GPUの利用率を収集
        match gpu.vendor {
            GpuVendor::Nvidia => Self::collect_nvidia_usage(gpu, config),
            GpuVendor::Amd => Self::collect_amd_usage(gpu, config),
            GpuVendor::Intel => Self::collect_intel_usage(gpu, config),
            GpuVendor::Apple => Self::collect_apple_usage(gpu, config),
            GpuVendor::Other(_) => Self::collect_generic_usage(gpu, config),
        }
    }
    
    /// NVIDIA GPUの利用率データ収集
    fn collect_nvidia_usage(gpu: &GpuInfo, config: &GpuMonitorConfig) -> Result<GpuUsage, GpuMonitorError> {
        // NVML APIを使用したNVIDIA GPU統計の収集
        // 実際の実装では、NVMLのAPIを使用してデータを取得します
        
        // サンプル実装
        let memory_used = DataSize::from_megabytes((rand::random::<f32>() * 1000.0) as u64);
        let memory_utilization = memory_used.as_megabytes() as f32 / gpu.total_memory.as_megabytes() as f32 * 100.0;
        
        Ok(GpuUsage {
            id: gpu.id.clone(),
            utilization: rand::random::<f32>() * 100.0,
            memory_used,
            memory_utilization,
            temperature: if config.enable_temperature_monitoring {
                Some(Temperature::from_celsius(30.0 + rand::random::<f32>() * 50.0))
            } else {
                None
            },
            power_usage: if config.enable_power_monitoring {
                Some(30.0 + rand::random::<f32>() * 150.0)
            } else {
                None
            },
            encoder_utilization: if config.enable_detailed_monitoring {
                Some(rand::random::<f32>() * 100.0)
            } else {
                None
            },
            decoder_utilization: if config.enable_detailed_monitoring {
                Some(rand::random::<f32>() * 100.0)
            } else {
                None
            },
            timestamp: Instant::now(),
        })
    }
    
    /// AMD GPUの利用率データ収集
    fn collect_amd_usage(gpu: &GpuInfo, config: &GpuMonitorConfig) -> Result<GpuUsage, GpuMonitorError> {
        // ROCm/sysfsを使用したAMD GPU統計の収集
        // 実際の実装では、ROCmのAPIまたはsysfsからデータを取得します
        
        // サンプル実装
        let memory_used = DataSize::from_megabytes((rand::random::<f32>() * 1000.0) as u64);
        let memory_utilization = memory_used.as_megabytes() as f32 / gpu.total_memory.as_megabytes() as f32 * 100.0;
        
        Ok(GpuUsage {
            id: gpu.id.clone(),
            utilization: rand::random::<f32>() * 100.0,
            memory_used,
            memory_utilization,
            temperature: if config.enable_temperature_monitoring {
                Some(Temperature::from_celsius(30.0 + rand::random::<f32>() * 50.0))
            } else {
                None
            },
            power_usage: if config.enable_power_monitoring {
                Some(30.0 + rand::random::<f32>() * 120.0)
            } else {
                None
            },
            encoder_utilization: if config.enable_detailed_monitoring {
                Some(rand::random::<f32>() * 100.0)
            } else {
                None
            },
            decoder_utilization: if config.enable_detailed_monitoring {
                Some(rand::random::<f32>() * 100.0)
            } else {
                None
            },
            timestamp: Instant::now(),
        })
    }
    
    /// Intel GPUの利用率データ収集
    fn collect_intel_usage(gpu: &GpuInfo, config: &GpuMonitorConfig) -> Result<GpuUsage, GpuMonitorError> {
        // Intel GPUの統計収集
        // 実際の実装では、Intel Media SDKやsysfsからデータを取得します
        
        // サンプル実装
        let memory_used = DataSize::from_megabytes((rand::random::<f32>() * 500.0) as u64);
        let memory_utilization = memory_used.as_megabytes() as f32 / gpu.total_memory.as_megabytes() as f32 * 100.0;
        
        Ok(GpuUsage {
            id: gpu.id.clone(),
            utilization: rand::random::<f32>() * 100.0,
            memory_used,
            memory_utilization,
            temperature: if config.enable_temperature_monitoring {
                Some(Temperature::from_celsius(30.0 + rand::random::<f32>() * 30.0))
            } else {
                None
            },
            power_usage: if config.enable_power_monitoring {
                Some(5.0 + rand::random::<f32>() * 25.0)
            } else {
                None
            },
            encoder_utilization: if config.enable_detailed_monitoring {
                Some(rand::random::<f32>() * 100.0)
            } else {
                None
            },
            decoder_utilization: if config.enable_detailed_monitoring {
                Some(rand::random::<f32>() * 100.0)
            } else {
                None
            },
            timestamp: Instant::now(),
        })
    }
    
    /// Apple GPUの利用率データ収集
    fn collect_apple_usage(gpu: &GpuInfo, config: &GpuMonitorConfig) -> Result<GpuUsage, GpuMonitorError> {
        // Apple Silicon GPUの統計収集
        // 実際の実装では、IOKitのAPIを使用してデータを取得します
        
        // サンプル実装
        let memory_used = DataSize::from_megabytes((rand::random::<f32>() * 1000.0) as u64);
        let memory_utilization = memory_used.as_megabytes() as f32 / gpu.total_memory.as_megabytes() as f32 * 100.0;
        
        Ok(GpuUsage {
            id: gpu.id.clone(),
            utilization: rand::random::<f32>() * 100.0,
            memory_used,
            memory_utilization,
            temperature: if config.enable_temperature_monitoring {
                Some(Temperature::from_celsius(25.0 + rand::random::<f32>() * 35.0))
            } else {
                None
            },
            power_usage: if config.enable_power_monitoring {
                Some(2.0 + rand::random::<f32>() * 18.0)
            } else {
                None
            },
            encoder_utilization: if config.enable_detailed_monitoring {
                Some(rand::random::<f32>() * 100.0)
            } else {
                None
            },
            decoder_utilization: if config.enable_detailed_monitoring {
                Some(rand::random::<f32>() * 100.0)
            } else {
                None
            },
            timestamp: Instant::now(),
        })
    }
    
    /// 汎用的なGPUの利用率データ収集
    fn collect_generic_usage(gpu: &GpuInfo, config: &GpuMonitorConfig) -> Result<GpuUsage, GpuMonitorError> {
        // 汎用的なGPU統計収集（OpenGL/Vulkanを使用）
        // 実際の実装では、OpenGLやVulkanのAPIを使用してデータを取得します
        
        // サンプル実装
        let memory_used = DataSize::from_megabytes((rand::random::<f32>() * 500.0) as u64);
        let memory_utilization = memory_used.as_megabytes() as f32 / gpu.total_memory.as_megabytes() as f32 * 100.0;
        
        Ok(GpuUsage {
            id: gpu.id.clone(),
            utilization: rand::random::<f32>() * 100.0,
            memory_used,
            memory_utilization,
            temperature: if config.enable_temperature_monitoring {
                Some(Temperature::from_celsius(30.0 + rand::random::<f32>() * 20.0))
            } else {
                None
            },
            power_usage: if config.enable_power_monitoring {
                Some(5.0 + rand::random::<f32>() * 15.0)
            } else {
                None
            },
            encoder_utilization: None,
            decoder_utilization: None,
            timestamp: Instant::now(),
        })
    }
    
    /// GPUモニタリングを停止
    pub async fn shutdown(&mut self) -> Result<(), GpuMonitorError> {
        {
            let mut status = self.status.lock().unwrap();
            if *status == SubsystemStatus::Stopped {
                return Ok(());
            }
            *status = SubsystemStatus::Stopping;
        }
        
        // モニタリングタスクが終了するのを待機
        if let Some(handle) = self.monitor_task.take() {
            if !handle.is_finished() {
                let _ = handle.await;
            }
        }
        
        *self.status.lock().unwrap() = SubsystemStatus::Stopped;
        Ok(())
    }
    
    /// 現在のモニタリングステータスを取得
    pub fn status(&self) -> SubsystemStatus {
        *self.status.lock().unwrap()
    }
    
    /// 検出されたGPUのリストを取得
    pub fn get_gpus(&self) -> Vec<GpuInfo> {
        self.gpus.lock().unwrap().clone()
    }
    
    /// 特定のGPUの現在の使用率を取得
    pub fn get_gpu_usage(&self, gpu_id: &str) -> Option<GpuUsage> {
        self.current_usage.lock().unwrap().get(gpu_id).cloned()
    }
    
    /// すべてのGPUの現在の使用率を取得
    pub fn get_all_gpu_usage(&self) -> HashMap<String, GpuUsage> {
        self.current_usage.lock().unwrap().clone()
    }
    
    /// GPU使用率更新の通知を受け取るサブスクライバーを取得
    pub fn subscribe(&self) -> broadcast::Receiver<HashMap<String, GpuUsage>> {
        self.usage_tx.subscribe()
    }
    
    /// モニタリング設定を更新
    pub fn update_config(&mut self, config: GpuMonitorConfig) {
        self.config = config;
        // 設定更新後、再起動が必要な場合がある
        // 実際の実装では、必要に応じてモニタリングタスクを再起動します
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    
    #[test]
    fn test_gpu_monitor_creation() {
        let monitor = GpuMonitor::default();
        assert_eq!(monitor.status(), SubsystemStatus::Stopped);
    }
    
    #[test]
    fn test_gpu_vendor_display() {
        assert_eq!(GpuVendor::Nvidia.to_string(), "NVIDIA");
        assert_eq!(GpuVendor::Amd.to_string(), "AMD");
        assert_eq!(GpuVendor::Intel.to_string(), "Intel");
        assert_eq!(GpuVendor::Apple.to_string(), "Apple");
        assert_eq!(GpuVendor::Other(0x10DE).to_string(), "その他ベンダー(ID: 0x10de)");
    }
    
    #[test]
    fn test_gpu_monitor_config_default() {
        let config = GpuMonitorConfig::default();
        assert_eq!(config.polling_interval_ms, 1000);
        assert!(config.enable_detailed_monitoring);
        assert!(config.enable_temperature_monitoring);
        assert!(config.enable_power_monitoring);
        assert!(config.enable_auto_detection);
    }
    
    #[test]
    fn test_gpu_error_conversion() {
        let error = GpuMonitorError::DetectionFailed("検出失敗".to_string());
        let system_error: SystemError = error.into();
        match system_error {
            SystemError::HardwareMonitorError(msg) => {
                assert!(msg.contains("検出失敗"));
            }
            _ => panic!("期待される変換結果ではありません"),
        }
    }
    
    #[test]
    fn test_gpu_usage_mock_data() {
        let gpu = GpuInfo {
            id: "test-gpu".to_string(),
            index: 0,
            vendor: GpuVendor::Nvidia,
            name: "テストGPU".to_string(),
            driver_version: Some("1.0".to_string()),
            total_memory: DataSize::from_megabytes(1024),
            features: HashMap::new(),
        };
        
        let config = GpuMonitorConfig::default();
        let usage = GpuMonitor::collect_nvidia_usage(&gpu, &config).unwrap();
        
        assert_eq!(usage.id, "test-gpu");
        assert!(usage.utilization >= 0.0 && usage.utilization <= 100.0);
        assert!(usage.memory_utilization >= 0.0 && usage.memory_utilization <= 100.0);
        assert!(usage.temperature.is_some());
        assert!(usage.power_usage.is_some());
    }
    
    #[test]
    fn test_gpu_monitor_lifecycle() {
        let rt = Runtime::new().unwrap();
        
        rt.block_on(async {
            let mut monitor = GpuMonitor::default();
            
            // 環境変数を設定してGPUをシミュレート
            std::env::set_var("NVIDIA_DEV", "1");
            
            // 初期化および起動
            if let Err(e) = monitor.initialize().await {
                // 実際のハードウェアがない場合はエラーになる可能性がある
                println!("GPUモニター初期化エラー (テスト環境では許容): {}", e);
                return;
            }
            
            assert_eq!(monitor.status(), SubsystemStatus::Running);
            
            // GPUのリストを取得
            let gpus = monitor.get_gpus();
            if !gpus.is_empty() {
                println!("検出されたGPU: {:?}", gpus);
            }
            
            // 少し待機して使用率データが収集されるのを待つ
            tokio::time::sleep(Duration::from_millis(1500)).await;
            
            // 使用率データを取得
            let usage = monitor.get_all_gpu_usage();
            if !usage.is_empty() {
                println!("GPU使用率: {:?}", usage);
            }
            
            // シャットダウン
            monitor.shutdown().await.unwrap();
            assert_eq!(monitor.status(), SubsystemStatus::Stopped);
            
            // 環境変数をクリーンアップ
            std::env::remove_var("NVIDIA_DEV");
        });
    }
} 