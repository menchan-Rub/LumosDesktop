use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use log::{debug, error, info, warn};

use crate::core::system::hardware_monitor::{DataPoint, MonitoringData, HistoryDataType};
use crate::core::utils::error::{Result, SystemError};
use crate::core::utils::system_info::{self, DiskInfo, DiskHealth, DiskType, DiskPerformance};

/// ディスクの監視状態を表す構造体
#[derive(Debug, Clone)]
pub struct DiskMonitorState {
    /// ディスクデバイスパス
    pub device_path: String,
    /// ディスク名
    pub name: String,
    /// ディスクタイプ (SSD, HDD, NVMe など)
    pub disk_type: DiskType,
    /// ディスク総容量 (バイト単位)
    pub total_space: u64,
    /// 使用済み容量 (バイト単位)
    pub used_space: u64,
    /// 空き容量 (バイト単位)
    pub free_space: u64,
    /// 使用率 (0.0 ~ 1.0)
    pub usage_percent: f64,
    /// ディスク健全性
    pub health: DiskHealth,
    /// パフォーマンスメトリクス
    pub performance: DiskPerformance,
    /// マウントポイント
    pub mount_points: Vec<String>,
    /// 最終更新時刻
    pub last_updated: Instant,
}

impl DiskMonitorState {
    /// 新しいディスク監視状態を作成
    pub fn new(disk_info: DiskInfo) -> Self {
        let used = disk_info.total_space.saturating_sub(disk_info.free_space);
        let usage_percent = if disk_info.total_space > 0 {
            used as f64 / disk_info.total_space as f64
        } else {
            0.0
        };

        Self {
            device_path: disk_info.device_path,
            name: disk_info.name,
            disk_type: disk_info.disk_type,
            total_space: disk_info.total_space,
            used_space: used,
            free_space: disk_info.free_space,
            usage_percent,
            health: disk_info.health,
            performance: disk_info.performance,
            mount_points: disk_info.mount_points,
            last_updated: Instant::now(),
        }
    }

    /// 使用率が指定されたしきい値を超えているかどうかをチェック
    pub fn is_usage_critical(&self, threshold: f64) -> bool {
        self.usage_percent >= threshold
    }

    /// 空き容量が指定されたしきい値未満かどうかをチェック（バイト単位）
    pub fn is_free_space_low(&self, min_free_bytes: u64) -> bool {
        self.free_space < min_free_bytes
    }

    /// ディスク健全性に問題があるかどうかをチェック
    pub fn has_health_issues(&self) -> bool {
        match self.health {
            DiskHealth::Good => false,
            DiskHealth::Warning | DiskHealth::Critical | DiskHealth::Unknown => true,
        }
    }

    /// ディスク状態の人間が読める形式の概要を取得
    pub fn summary(&self) -> String {
        let health_str = match self.health {
            DiskHealth::Good => "良好",
            DiskHealth::Warning => "警告",
            DiskHealth::Critical => "危険",
            DiskHealth::Unknown => "不明",
        };

        let disk_type_str = match self.disk_type {
            DiskType::Hdd => "HDD",
            DiskType::Ssd => "SSD",
            DiskType::Nvme => "NVMe",
            DiskType::Unknown => "不明",
        };

        format!(
            "{}({}): {}%, 空き容量: {:.2} GB, 健全性: {}, 読取: {:.1} MB/s, 書込: {:.1} MB/s",
            self.name,
            disk_type_str,
            (self.usage_percent * 100.0) as u32,
            self.free_space as f64 / 1_073_741_824.0, // バイトからGBに変換
            health_str,
            self.performance.read_rate / 1_048_576.0,  // バイト/秒からMB/秒に変換
            self.performance.write_rate / 1_048_576.0, // バイト/秒からMB/秒に変換
        )
    }
}

/// ディスクモニターの構成
#[derive(Debug, Clone)]
pub struct DiskMonitorConfig {
    /// 監視間隔（ミリ秒）
    pub interval_ms: u64,
    /// S.M.A.R.T.データの読み取り間隔（ミリ秒）
    pub smart_read_interval_ms: u64,
    /// ディスク使用率のクリティカルしきい値（0.0～1.0）
    pub usage_critical_threshold: f64,
    /// 空き容量の最小しきい値（バイト単位）
    pub min_free_space_bytes: u64,
    /// 監視するディスクのパターン（正規表現）
    pub monitor_disk_pattern: String,
    /// 無視するディスクパターン（正規表現）
    pub ignore_disk_pattern: String,
}

impl Default for DiskMonitorConfig {
    fn default() -> Self {
        Self {
            interval_ms: 5000,               // 5秒ごとの監視
            smart_read_interval_ms: 3600000, // 1時間ごとのS.M.A.R.T.データ読み取り
            usage_critical_threshold: 0.95,  // 95%使用率でアラート
            min_free_space_bytes: 1_073_741_824, // 1GB以下で警告
            monitor_disk_pattern: ".*".to_string(), // すべてのディスク
            ignore_disk_pattern: "^(loop|ram|zram).*".to_string(), // loop, ram, zramデバイスを無視
        }
    }
}

/// ディスクモニター
pub struct DiskMonitor {
    /// モニターの構成
    config: DiskMonitorConfig,
    /// ディスク状態マップ（デバイスパスをキーとする）
    disks: Arc<Mutex<HashMap<String, DiskMonitorState>>>,
    /// 前回のS.M.A.R.T.読み取り時刻
    last_smart_read: Arc<Mutex<Instant>>,
    /// モニターが実行中かどうか
    running: Arc<Mutex<bool>>,
    /// モニタースレッドハンドル
    monitor_thread: Option<thread::JoinHandle<()>>,
}

impl DiskMonitor {
    /// 新しいディスクモニターを作成
    pub fn new(config: DiskMonitorConfig) -> Self {
        Self {
            config,
            disks: Arc::new(Mutex::new(HashMap::new())),
            last_smart_read: Arc::new(Mutex::new(Instant::now().checked_sub(Duration::from_secs(3600)).unwrap_or_else(Instant::now))),
            running: Arc::new(Mutex::new(false)),
            monitor_thread: None,
        }
    }

    /// デフォルト設定でディスクモニターを作成
    pub fn new_default() -> Self {
        Self::new(DiskMonitorConfig::default())
    }

    /// モニタリングを開始
    pub fn start(&mut self) -> Result<()> {
        let mut running = self.running.lock().map_err(|e| {
            error!("ディスクモニターの起動に失敗: {}", e);
            SystemError::Mutex("ディスクモニターの実行状態ロックの取得に失敗".to_string())
        })?;

        if *running {
            warn!("ディスクモニターは既に実行中です");
            return Ok(());
        }

        *running = true;
        drop(running);

        let config = self.config.clone();
        let disks = Arc::clone(&self.disks);
        let running = Arc::clone(&self.running);
        let last_smart_read = Arc::clone(&self.last_smart_read);

        // 初回のスキャンと基準値の設定
        self.scan_disks()?;

        // モニタリングスレッドを起動
        let handle = thread::Builder::new()
            .name("disk-monitor".to_string())
            .spawn(move || {
                info!("ディスクモニターが開始されました。間隔: {}ミリ秒", config.interval_ms);
                
                while {
                    let is_running = running.lock().unwrap_or_else(|e| {
                        error!("ディスクモニターのロック取得に失敗: {}", e);
                        return Box::new(false);
                    });
                    *is_running
                } {
                    // ディスク情報をスキャン
                    if let Err(e) = Self::update_disk_info(&config, &disks, &last_smart_read) {
                        error!("ディスク情報の更新に失敗: {}", e);
                    }

                    // ディスク使用率とヘルスチェック
                    Self::check_disk_conditions(&config, &disks);

                    // 間隔を空けて再度スキャン
                    thread::sleep(Duration::from_millis(config.interval_ms));
                }
                
                info!("ディスクモニターが停止しました");
            })
            .map_err(|e| {
                error!("ディスクモニタースレッドの起動に失敗: {}", e);
                *self.running.lock().unwrap() = false;
                SystemError::Thread("ディスクモニタースレッドの作成に失敗".to_string())
            })?;

        self.monitor_thread = Some(handle);
        Ok(())
    }

    /// モニタリングを停止
    pub fn stop(&mut self) -> Result<()> {
        let mut running = self.running.lock().map_err(|e| {
            error!("ディスクモニターの停止に失敗: {}", e);
            SystemError::Mutex("ディスクモニターの実行状態ロックの取得に失敗".to_string())
        })?;

        if !*running {
            warn!("ディスクモニターは既に停止しています");
            return Ok(());
        }

        *running = false;
        drop(running);

        // スレッドの終了を待機
        if let Some(handle) = self.monitor_thread.take() {
            match handle.join() {
                Ok(_) => {
                    info!("ディスクモニタースレッドが正常に終了しました");
                    Ok(())
                }
                Err(e) => {
                    error!("ディスクモニタースレッドの終了に失敗: {:?}", e);
                    Err(SystemError::Thread("ディスクモニタースレッドの終了に失敗".to_string()).into())
                }
            }
        } else {
            warn!("ディスクモニタースレッドが見つかりません");
            Ok(())
        }
    }

    /// 現在のディスク状態を取得
    pub fn get_disk_states(&self) -> Result<HashMap<String, DiskMonitorState>> {
        let disks = self.disks.lock().map_err(|e| {
            error!("ディスク状態の取得に失敗: {}", e);
            SystemError::Mutex("ディスク状態ロックの取得に失敗".to_string())
        })?;
        Ok(disks.clone())
    }

    /// 特定のディスクの状態を取得
    pub fn get_disk_state(&self, device_path: &str) -> Result<Option<DiskMonitorState>> {
        let disks = self.disks.lock().map_err(|e| {
            error!("ディスク状態の取得に失敗: {}", e);
            SystemError::Mutex("ディスク状態ロックの取得に失敗".to_string())
        })?;
        Ok(disks.get(device_path).cloned())
    }

    /// 監視間隔を更新
    pub fn update_interval(&mut self, interval_ms: u64) {
        self.config.interval_ms = interval_ms;
        info!("ディスクモニター間隔が{}ミリ秒に更新されました", interval_ms);
    }

    /// S.M.A.R.T.データの読み取り間隔を更新
    pub fn update_smart_read_interval(&mut self, interval_ms: u64) {
        self.config.smart_read_interval_ms = interval_ms;
        info!("S.M.A.R.T.読み取り間隔が{}ミリ秒に更新されました", interval_ms);
    }

    /// 使用率のクリティカルしきい値を更新
    pub fn update_usage_threshold(&mut self, threshold: f64) {
        if threshold < 0.0 || threshold > 1.0 {
            warn!("無効な使用率しきい値: {}。0.0～1.0の値を使用してください", threshold);
            return;
        }
        self.config.usage_critical_threshold = threshold;
        info!("ディスク使用率しきい値が{}に更新されました", threshold);
    }

    /// 最小空き容量しきい値を更新（バイト単位）
    pub fn update_min_free_space(&mut self, min_free_bytes: u64) {
        self.config.min_free_space_bytes = min_free_bytes;
        info!("最小空き容量しきい値が{}バイトに更新されました", min_free_bytes);
    }

    /// ディスク情報をモニタリングデータに変換
    pub fn update_monitoring_data(&self, data: &mut MonitoringData) -> Result<()> {
        let disks = self.disks.lock().map_err(|e| {
            error!("ディスク状態の取得に失敗: {}", e);
            SystemError::Mutex("ディスク状態ロックの取得に失敗".to_string())
        })?;

        // ディスク使用率のデータポイントを作成
        let mut disk_usage_points = Vec::new();
        let mut disk_free_space_points = Vec::new();
        let mut disk_io_read_points = Vec::new();
        let mut disk_io_write_points = Vec::new();
        let mut disk_health_points = Vec::new();

        for (_, disk) in disks.iter() {
            // ディスク使用率
            disk_usage_points.push(DataPoint {
                timestamp: disk.last_updated,
                label: disk.name.clone(),
                value: disk.usage_percent,
            });

            // 空き容量（GB単位）
            disk_free_space_points.push(DataPoint {
                timestamp: disk.last_updated,
                label: disk.name.clone(),
                value: disk.free_space as f64 / 1_073_741_824.0, // バイトからGBに変換
            });

            // 読み取りレート（MB/秒）
            disk_io_read_points.push(DataPoint {
                timestamp: disk.last_updated,
                label: disk.name.clone(),
                value: disk.performance.read_rate as f64 / 1_048_576.0, // バイト/秒からMB/秒に変換
            });

            // 書き込みレート（MB/秒）
            disk_io_write_points.push(DataPoint {
                timestamp: disk.last_updated,
                label: disk.name.clone(),
                value: disk.performance.write_rate as f64 / 1_048_576.0, // バイト/秒からMB/秒に変換
            });

            // 健全性（数値化）
            let health_value = match disk.health {
                DiskHealth::Good => 1.0,
                DiskHealth::Warning => 0.5,
                DiskHealth::Critical => 0.0,
                DiskHealth::Unknown => -1.0,
            };

            disk_health_points.push(DataPoint {
                timestamp: disk.last_updated,
                label: disk.name.clone(),
                value: health_value,
            });
        }

        // モニタリングデータの更新
        data.disk_usage = disk_usage_points;
        data.disk_free_space = disk_free_space_points;
        data.disk_io_read = disk_io_read_points;
        data.disk_io_write = disk_io_write_points;
        data.disk_health = disk_health_points;

        Ok(())
    }

    /// ディスク履歴データの取得
    pub fn get_history_data(&self, data_type: HistoryDataType, device_name: Option<String>) -> Vec<DataPoint> {
        match data_type {
            HistoryDataType::DiskUsage => {
                let disks = self.disks.lock().unwrap_or_else(|_| {
                    error!("ディスク状態ロックの取得に失敗");
                    return HashMap::new().into();
                });

                disks.iter()
                    .filter(|(_, disk)| {
                        if let Some(ref name) = device_name {
                            &disk.name == name
                        } else {
                            true
                        }
                    })
                    .map(|(_, disk)| {
                        DataPoint {
                            timestamp: disk.last_updated,
                            label: disk.name.clone(),
                            value: disk.usage_percent,
                        }
                    })
                    .collect()
            }
            HistoryDataType::DiskFreeSpace => {
                let disks = self.disks.lock().unwrap_or_else(|_| {
                    error!("ディスク状態ロックの取得に失敗");
                    return HashMap::new().into();
                });

                disks.iter()
                    .filter(|(_, disk)| {
                        if let Some(ref name) = device_name {
                            &disk.name == name
                        } else {
                            true
                        }
                    })
                    .map(|(_, disk)| {
                        DataPoint {
                            timestamp: disk.last_updated,
                            label: disk.name.clone(),
                            value: disk.free_space as f64 / 1_073_741_824.0, // バイトからGBに変換
                        }
                    })
                    .collect()
            }
            HistoryDataType::DiskIORead => {
                let disks = self.disks.lock().unwrap_or_else(|_| {
                    error!("ディスク状態ロックの取得に失敗");
                    return HashMap::new().into();
                });

                disks.iter()
                    .filter(|(_, disk)| {
                        if let Some(ref name) = device_name {
                            &disk.name == name
                        } else {
                            true
                        }
                    })
                    .map(|(_, disk)| {
                        DataPoint {
                            timestamp: disk.last_updated,
                            label: disk.name.clone(),
                            value: disk.performance.read_rate as f64 / 1_048_576.0, // バイト/秒からMB/秒に変換
                        }
                    })
                    .collect()
            }
            HistoryDataType::DiskIOWrite => {
                let disks = self.disks.lock().unwrap_or_else(|_| {
                    error!("ディスク状態ロックの取得に失敗");
                    return HashMap::new().into();
                });

                disks.iter()
                    .filter(|(_, disk)| {
                        if let Some(ref name) = device_name {
                            &disk.name == name
                        } else {
                            true
                        }
                    })
                    .map(|(_, disk)| {
                        DataPoint {
                            timestamp: disk.last_updated,
                            label: disk.name.clone(),
                            value: disk.performance.write_rate as f64 / 1_048_576.0, // バイト/秒からMB/秒に変換
                        }
                    })
                    .collect()
            }
            HistoryDataType::DiskHealth => {
                let disks = self.disks.lock().unwrap_or_else(|_| {
                    error!("ディスク状態ロックの取得に失敗");
                    return HashMap::new().into();
                });

                disks.iter()
                    .filter(|(_, disk)| {
                        if let Some(ref name) = device_name {
                            &disk.name == name
                        } else {
                            true
                        }
                    })
                    .map(|(_, disk)| {
                        let health_value = match disk.health {
                            DiskHealth::Good => 1.0,
                            DiskHealth::Warning => 0.5,
                            DiskHealth::Critical => 0.0,
                            DiskHealth::Unknown => -1.0,
                        };

                        DataPoint {
                            timestamp: disk.last_updated,
                            label: disk.name.clone(),
                            value: health_value,
                        }
                    })
                    .collect()
            }
            _ => Vec::new(), // その他のデータタイプは空のベクトルを返す
        }
    }

    /// ディスク情報を更新
    fn update_disk_info(
        config: &DiskMonitorConfig,
        disks: &Arc<Mutex<HashMap<String, DiskMonitorState>>>,
        last_smart_read: &Arc<Mutex<Instant>>,
    ) -> Result<()> {
        // システムからディスク情報を取得
        let disk_info_list = system_info::get_disk_info_list()?;
        
        let should_read_smart = {
            let last_read = last_smart_read.lock().map_err(|e| {
                error!("S.M.A.R.T.読み取り時刻ロックの取得に失敗: {}", e);
                SystemError::Mutex("S.M.A.R.T.読み取り時刻ロックの取得に失敗".to_string())
            })?;
            
            let now = Instant::now();
            let elapsed = now.duration_since(*last_read);
            elapsed.as_millis() >= config.smart_read_interval_ms as u128
        };

        if should_read_smart {
            debug!("S.M.A.R.T.データの読み取りを実行中...");
            // S.M.A.R.T.データの読み取り
            // 注: この部分は実際のシステム情報ライブラリに依存します
            // system_info::update_disk_smart_data()?;
            
            // 最終S.M.A.R.T.読み取り時刻を更新
            let mut last_read = last_smart_read.lock().map_err(|e| {
                error!("S.M.A.R.T.読み取り時刻ロックの取得に失敗: {}", e);
                SystemError::Mutex("S.M.A.R.T.読み取り時刻ロックの取得に失敗".to_string())
            })?;
            
            *last_read = Instant::now();
        }

        // ディスク情報を処理
        let mut disks_map = disks.lock().map_err(|e| {
            error!("ディスク状態ロックの取得に失敗: {}", e);
            SystemError::Mutex("ディスク状態ロックの取得に失敗".to_string())
        })?;

        // パターンに基づいて監視対象のディスクをフィルタリング
        for disk_info in disk_info_list {
            // 無視パターンに一致するディスクをスキップ
            if system_info::matches_pattern(&disk_info.device_path, &config.ignore_disk_pattern) {
                continue;
            }

            // 監視パターンに一致するディスクを処理
            if system_info::matches_pattern(&disk_info.device_path, &config.monitor_disk_pattern) {
                let disk_state = DiskMonitorState::new(disk_info);
                disks_map.insert(disk_state.device_path.clone(), disk_state);
            }
        }

        Ok(())
    }

    /// ディスク条件をチェックしてアラートを発生
    fn check_disk_conditions(
        config: &DiskMonitorConfig,
        disks: &Arc<Mutex<HashMap<String, DiskMonitorState>>>,
    ) {
        let disks_map = match disks.lock() {
            Ok(map) => map,
            Err(e) => {
                error!("ディスク状態ロックの取得に失敗: {}", e);
                return;
            }
        };

        for (_, disk) in disks_map.iter() {
            // 使用率チェック
            if disk.is_usage_critical(config.usage_critical_threshold) {
                warn!(
                    "ディスク使用率が高い: {} - {:.1}%",
                    disk.name,
                    disk.usage_percent * 100.0
                );
                // ここでアラートコールバックを呼び出す
            }

            // 空き容量チェック
            if disk.is_free_space_low(config.min_free_space_bytes) {
                warn!(
                    "ディスク空き容量が低い: {} - {:.2} GB",
                    disk.name,
                    disk.free_space as f64 / 1_073_741_824.0
                );
                // ここでアラートコールバックを呼び出す
            }

            // 健全性チェック
            if disk.has_health_issues() {
                warn!(
                    "ディスク健全性に問題があります: {} - {:?}",
                    disk.name, disk.health
                );
                // ここでアラートコールバックを呼び出す
            }
        }
    }

    /// すべてのディスクをスキャン
    pub fn scan_disks(&self) -> Result<()> {
        // システムからディスク情報を取得
        let disk_info_list = system_info::get_disk_info_list()?;
        
        let mut disks_map = self.disks.lock().map_err(|e| {
            error!("ディスク状態ロックの取得に失敗: {}", e);
            SystemError::Mutex("ディスク状態ロックの取得に失敗".to_string())
        })?;

        // 既存のディスクマップをクリア
        disks_map.clear();

        // パターンに基づいて監視対象のディスクをフィルタリング
        for disk_info in disk_info_list {
            // 無視パターンに一致するディスクをスキップ
            if system_info::matches_pattern(&disk_info.device_path, &self.config.ignore_disk_pattern) {
                continue;
            }

            // 監視パターンに一致するディスクを処理
            if system_info::matches_pattern(&disk_info.device_path, &self.config.monitor_disk_pattern) {
                let disk_state = DiskMonitorState::new(disk_info);
                disks_map.insert(disk_state.device_path.clone(), disk_state);
                
                info!("ディスクを検出: {}", disk_state.summary());
            }
        }

        info!("{}台のディスクがスキャンされました", disks_map.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_disk_monitor_state() {
        let disk_info = DiskInfo {
            device_path: "/dev/sda".to_string(),
            name: "Main SSD".to_string(),
            disk_type: DiskType::Ssd,
            total_space: 512_000_000_000, // 512 GB
            free_space: 256_000_000_000,  // 256 GB
            health: DiskHealth::Good,
            performance: DiskPerformance {
                read_rate: 500_000_000,  // 500 MB/s
                write_rate: 400_000_000, // 400 MB/s
                iops: 50_000,
                latency_ms: 0.5,
            },
            mount_points: vec!["/".to_string()],
        };

        let state = DiskMonitorState::new(disk_info);

        assert_eq!(state.device_path, "/dev/sda");
        assert_eq!(state.name, "Main SSD");
        assert_eq!(state.total_space, 512_000_000_000);
        assert_eq!(state.free_space, 256_000_000_000);
        assert_eq!(state.used_space, 256_000_000_000);
        assert!((state.usage_percent - 0.5).abs() < f64::EPSILON);

        // 使用率チェック
        assert!(state.is_usage_critical(0.4));
        assert!(!state.is_usage_critical(0.6));

        // 空き容量チェック
        assert!(!state.is_free_space_low(200_000_000_000));
        assert!(state.is_free_space_low(300_000_000_000));

        // 健全性チェック
        assert!(!state.has_health_issues());

        // 概要文字列
        let summary = state.summary();
        assert!(summary.contains("Main SSD"));
        assert!(summary.contains("SSD"));
        assert!(summary.contains("50%"));
        assert!(summary.contains("256.00 GB"));
        assert!(summary.contains("良好"));
    }

    #[test]
    fn test_disk_monitor_config_default() {
        let config = DiskMonitorConfig::default();
        
        assert_eq!(config.interval_ms, 5000);
        assert_eq!(config.smart_read_interval_ms, 3600000);
        assert!((config.usage_critical_threshold - 0.95).abs() < f64::EPSILON);
        assert_eq!(config.min_free_space_bytes, 1_073_741_824);
        assert_eq!(config.monitor_disk_pattern, ".*");
        assert_eq!(config.ignore_disk_pattern, "^(loop|ram|zram).*");
    }

    #[test]
    fn test_disk_monitor_basic_operations() {
        let config = DiskMonitorConfig {
            interval_ms: 100, // 短い間隔でテスト
            ..DiskMonitorConfig::default()
        };

        let mut monitor = DiskMonitor::new(config);
        
        // モニター開始
        let result = monitor.start();
        assert!(result.is_ok());
        
        // 少し待機して更新を許可
        sleep(Duration::from_millis(200));
        
        // データを取得
        let disk_states = monitor.get_disk_states();
        assert!(disk_states.is_ok());
        
        // モニター停止
        let result = monitor.stop();
        assert!(result.is_ok());
    }

    #[test]
    fn test_disk_monitor_config_updates() {
        let mut monitor = DiskMonitor::new_default();
        
        // 間隔の更新
        monitor.update_interval(10000);
        assert_eq!(monitor.config.interval_ms, 10000);
        
        // S.M.A.R.T.読み取り間隔の更新
        monitor.update_smart_read_interval(7200000);
        assert_eq!(monitor.config.smart_read_interval_ms, 7200000);
        
        // 使用率しきい値の更新
        monitor.update_usage_threshold(0.85);
        assert!((monitor.config.usage_critical_threshold - 0.85).abs() < f64::EPSILON);
        
        // 無効な使用率しきい値
        monitor.update_usage_threshold(1.5);
        assert!((monitor.config.usage_critical_threshold - 0.85).abs() < f64::EPSILON); // 変更なし
        
        // 最小空き容量の更新
        monitor.update_min_free_space(2 * 1_073_741_824); // 2GB
        assert_eq!(monitor.config.min_free_space_bytes, 2 * 1_073_741_824);
    }

    // 注: 以下のテストは実際のシステム情報取得に依存するため、モックを使用するか条件付きでテストすべきです
    
    #[test]
    #[ignore] // 実際のシステムディスクに依存するため、通常のテスト実行では無視
    fn test_disk_scanning() {
        let monitor = DiskMonitor::new_default();
        
        // ディスクスキャン
        let result = monitor.scan_disks();
        assert!(result.is_ok());
        
        // 少なくとも1つのディスクが検出されるはず
        let disk_states = monitor.get_disk_states().unwrap();
        assert!(!disk_states.is_empty());
    }
} 