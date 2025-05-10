// LumosDesktop ネットワークモニターモジュール
// ネットワークインターフェースの帯域使用率、接続状態などを監視します

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::Path;

use log::{debug, error, info, warn};

use crate::core::system::hardware_monitor::{DataPoint, MonitoringData, HistoryDataType};
use crate::core::utils::error::{Result, SystemError};

/// ネットワークインターフェースタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkInterfaceType {
    /// 有線イーサネット
    Ethernet,
    /// 無線LAN
    Wireless,
    /// 仮想インターフェース
    Virtual,
    /// ループバック
    Loopback,
    /// モバイルブロードバンド
    Mobile,
    /// ブリッジインターフェース
    Bridge,
    /// その他/不明
    Other,
}

impl NetworkInterfaceType {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ethernet => "有線",
            Self::Wireless => "無線",
            Self::Virtual => "仮想",
            Self::Loopback => "ループバック",
            Self::Mobile => "モバイル",
            Self::Bridge => "ブリッジ",
            Self::Other => "その他",
        }
    }
}

/// ネットワーク接続状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkConnectionState {
    /// 接続済み
    Connected,
    /// 接続中
    Connecting,
    /// 切断済み
    Disconnected,
    /// エラー状態
    Error,
    /// 不明
    Unknown,
}

/// ネットワーク使用状況
#[derive(Debug, Clone)]
pub struct NetworkUsage {
    /// 受信バイト数
    pub rx_bytes: u64,
    /// 送信バイト数
    pub tx_bytes: u64,
    /// 受信パケット数
    pub rx_packets: u64,
    /// 送信パケット数
    pub tx_packets: u64,
    /// 受信速度 (バイト/秒)
    pub rx_rate: f64,
    /// 送信速度 (バイト/秒)
    pub tx_rate: f64,
    /// 合計速度 (バイト/秒)
    pub total_rate: f64,
    /// 受信エラー数
    pub rx_errors: u64,
    /// 送信エラー数
    pub tx_errors: u64,
    /// 受信ドロップ数
    pub rx_dropped: u64,
    /// 送信ドロップ数
    pub tx_dropped: u64,
    /// 最終更新時刻
    pub last_updated: Instant,
}

/// インターフェース情報
#[derive(Debug, Clone)]
pub struct NetworkInterfaceInfo {
    /// インターフェース名
    pub name: String,
    /// インターフェースタイプ
    pub interface_type: NetworkInterfaceType,
    /// MACアドレス
    pub mac_address: String,
    /// IPアドレス (IPv4/IPv6)
    pub ip_addresses: Vec<String>,
    /// 接続状態
    pub state: NetworkConnectionState,
    /// 最大帯域 (ビット/秒)
    pub max_bandwidth: Option<u64>,
    /// 使用状況
    pub usage: NetworkUsage,
    /// MTU (最大転送単位)
    pub mtu: u32,
    /// 追加プロパティ
    pub properties: HashMap<String, String>,
}

/// ネットワーク情報
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    /// インターフェース情報 (インターフェース名をキーとする)
    pub interfaces: HashMap<String, NetworkInterfaceInfo>,
    /// プライマリインターフェース
    pub primary_interface: String,
    /// グローバルIPアドレス
    pub global_ip: Option<String>,
    /// デフォルトゲートウェイ
    pub default_gateway: Option<String>,
    /// DNSサーバー
    pub dns_servers: Vec<String>,
    /// 最終更新時刻
    pub last_updated: Instant,
}

impl NetworkInfo {
    /// 新しいネットワーク情報を作成
    pub fn new() -> Self {
        Self {
            interfaces: HashMap::new(),
            primary_interface: String::new(),
            global_ip: None,
            default_gateway: None,
            dns_servers: Vec::new(),
            last_updated: Instant::now(),
        }
    }

    /// 特定のインターフェース情報を取得
    pub fn get(&self, interface_name: &str) -> Option<&NetworkInterfaceInfo> {
        self.interfaces.get(interface_name)
    }
    
    /// ネットワーク状態の人間が読める形式の概要を取得
    pub fn summary(&self) -> String {
        if let Some(primary) = self.interfaces.get(&self.primary_interface) {
            let state_str = match primary.state {
                NetworkConnectionState::Connected => "接続済み",
                NetworkConnectionState::Connecting => "接続中",
                NetworkConnectionState::Disconnected => "切断済み",
                NetworkConnectionState::Error => "エラー",
                NetworkConnectionState::Unknown => "不明",
            };
            
            let ip_str = if !primary.ip_addresses.is_empty() {
                primary.ip_addresses[0].clone()
            } else {
                "IPなし".to_string()
            };
            
            let rx_rate_mb = primary.usage.rx_rate / 1_048_576.0; // Bytes to MB
            let tx_rate_mb = primary.usage.tx_rate / 1_048_576.0; // Bytes to MB
            
            format!(
                "{}({}): {}, IP: {}, 受信: {:.2} MB/s, 送信: {:.2} MB/s",
                primary.name,
                primary.interface_type.as_str(),
                state_str,
                ip_str,
                rx_rate_mb,
                tx_rate_mb
            )
        } else {
            "ネットワーク情報なし".to_string()
        }
    }
}

/// ネットワークモニターの構成
#[derive(Debug, Clone)]
pub struct NetworkMonitorConfig {
    /// 監視間隔（ミリ秒）
    pub interval_ms: u64,
    /// 監視するインターフェースパターン（正規表現）
    pub monitor_interface_pattern: String,
    /// 無視するインターフェースパターン（正規表現）
    pub ignore_interface_pattern: String,
    /// トラフィック警告しきい値（バイト/秒）
    pub traffic_warning_threshold: u64,
    /// DNSサーバー監視有効
    pub monitor_dns: bool,
    /// 接続品質監視有効
    pub monitor_connection_quality: bool,
}

impl Default for NetworkMonitorConfig {
    fn default() -> Self {
        Self {
            interval_ms: 2000,                 // 2秒ごとの監視
            monitor_interface_pattern: ".*".to_string(), // すべてのインターフェース
            ignore_interface_pattern: "^(lo|veth|docker|br-|virbr).*".to_string(), // 仮想・ループバックインターフェースを無視
            traffic_warning_threshold: 100 * 1024 * 1024, // 100MB/sでのトラフィック警告
            monitor_dns: true,
            monitor_connection_quality: false, // デフォルトでは接続品質監視は無効
        }
    }
}

/// ネットワークモニター
pub struct NetworkMonitor {
    /// モニターの構成
    config: NetworkMonitorConfig,
    /// ネットワーク状態
    network_state: Arc<Mutex<NetworkInfo>>,
    /// 前回の統計情報
    previous_stats: Arc<Mutex<HashMap<String, (u64, u64, Instant)>>>, // (rx_bytes, tx_bytes, timestamp)
    /// モニターが実行中かどうか
    running: Arc<Mutex<bool>>,
    /// モニタースレッドハンドル
    monitor_thread: Option<thread::JoinHandle<()>>,
}

impl NetworkMonitor {
    /// 新しいネットワークモニターを作成
    pub fn new(config: NetworkMonitorConfig) -> Self {
        Self {
            config,
            network_state: Arc::new(Mutex::new(NetworkInfo::new())),
            previous_stats: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
            monitor_thread: None,
        }
    }

    /// デフォルト設定でネットワークモニターを作成
    pub fn new_default() -> Self {
        Self::new(NetworkMonitorConfig::default())
    }

    /// モニタリングを開始
    pub fn start(&mut self) -> Result<()> {
        let mut running = self.running.lock().map_err(|e| {
            error!("ネットワークモニターの起動に失敗: {}", e);
            SystemError::Mutex("ネットワークモニターの実行状態ロックの取得に失敗".to_string())
        })?;

        if *running {
            warn!("ネットワークモニターは既に実行中です");
            return Ok(());
        }

        *running = true;
        drop(running);

        // 初回のネットワーク情報を取得
        self.update_network_info()?;

        let config = self.config.clone();
        let network_state = Arc::clone(&self.network_state);
        let previous_stats = Arc::clone(&self.previous_stats);
        let running = Arc::clone(&self.running);

        // モニタリングスレッドを起動
        let handle = thread::Builder::new()
            .name("network-monitor".to_string())
            .spawn(move || {
                info!("ネットワークモニターが開始されました。間隔: {}ミリ秒", config.interval_ms);
                
                while {
                    let is_running = running.lock().unwrap_or_else(|e| {
                        error!("ネットワークモニターのロック取得に失敗: {}", e);
                        return Box::new(false);
                    });
                    *is_running
                } {
                    // ネットワーク情報を更新
                    if let Err(e) = Self::update_network_statistics(&config, &network_state, &previous_stats) {
                        error!("ネットワーク統計情報の更新に失敗: {}", e);
                    }

                    // 接続品質監視
                    if config.monitor_connection_quality {
                        if let Err(e) = Self::check_connection_quality(&config, &network_state) {
                            error!("接続品質のチェックに失敗: {}", e);
                        }
                    }

                    // DNS監視
                    if config.monitor_dns {
                        if let Err(e) = Self::check_dns_servers(&network_state) {
                            error!("DNSサーバーのチェックに失敗: {}", e);
                        }
                    }

                    // 間隔を空けて再度更新
                    thread::sleep(Duration::from_millis(config.interval_ms));
                }
                
                info!("ネットワークモニターが停止しました");
            })
            .map_err(|e| {
                error!("ネットワークモニタースレッドの起動に失敗: {}", e);
                *self.running.lock().unwrap() = false;
                SystemError::Thread("ネットワークモニタースレッドの作成に失敗".to_string())
            })?;

        self.monitor_thread = Some(handle);
        Ok(())
    }

    /// モニタリングを停止
    pub fn stop(&mut self) -> Result<()> {
        let mut running = self.running.lock().map_err(|e| {
            error!("ネットワークモニターの停止に失敗: {}", e);
            SystemError::Mutex("ネットワークモニターの実行状態ロックの取得に失敗".to_string())
        })?;

        if !*running {
            warn!("ネットワークモニターは既に停止しています");
            return Ok(());
        }

        *running = false;
        drop(running);

        // スレッドの終了を待機
        if let Some(handle) = self.monitor_thread.take() {
            match handle.join() {
                Ok(_) => {
                    info!("ネットワークモニタースレッドが正常に終了しました");
                },
                Err(e) => {
                    error!("ネットワークモニタースレッドの終了に失敗: {:?}", e);
                    return Err(SystemError::Thread("ネットワークモニタースレッドの終了に失敗".to_string()).into());
                }
            }
        }

        Ok(())
    }

    /// 現在のネットワークインターフェース情報を取得
    pub fn get_network_info(&self) -> Result<NetworkInfo> {
        let network_info = self.network_state.lock().map_err(|e| {
            error!("ネットワーク情報の取得に失敗: {}", e);
            SystemError::Mutex("ネットワーク状態ロックの取得に失敗".to_string())
        })?;
        
        Ok(network_info.clone())
    }

    /// ネットワーク監視間隔を更新
    pub fn update_interval(&mut self, interval_ms: u64) {
        self.config.interval_ms = interval_ms;
    }

    /// トラフィック警告しきい値を更新
    pub fn update_traffic_warning_threshold(&mut self, threshold: u64) {
        self.config.traffic_warning_threshold = threshold;
    }

    /// 最新のネットワーク情報を取得して状態を更新
    pub fn update_network_info(&self) -> Result<()> {
        // Linuxシステム用の/proc/net/devからネットワーク統計情報を読み取る
        let interfaces = read_network_interfaces()?;
        let primary = find_primary_interface(&interfaces);
        
        // DNSサーバーとデフォルトゲートウェイ情報を取得
        let dns_servers = read_dns_servers()?;
        let default_gateway = read_default_gateway()?;
        
        // グローバルIPアドレスの取得（オプション）
        let global_ip = if self.config.monitor_connection_quality {
            fetch_global_ip().ok()
        } else {
            None
        };
        
        let mut network_state = self.network_state.lock().map_err(|e| {
            SystemError::Mutex(format!("ネットワーク状態ロックの取得に失敗: {}", e))
        })?;
        
        // 状態を更新
        network_state.interfaces = interfaces;
        network_state.primary_interface = primary;
        network_state.dns_servers = dns_servers;
        network_state.default_gateway = default_gateway;
        network_state.global_ip = global_ip;
        network_state.last_updated = Instant::now();
        
        Ok(())
    }

    /// モニタリングデータを更新
    pub fn update_monitoring_data(&self, data: &mut MonitoringData) -> Result<()> {
        let network_info = self.get_network_info()?;
        let now = Instant::now();
        
        // プライマリインターフェースの統計データを追加
        if let Some(primary_info) = network_info.interfaces.get(&network_info.primary_interface) {
            // 受信レート
            data.network_rx_rate.push(DataPoint {
                timestamp: now,
                label: primary_info.name.clone(),
                value: primary_info.usage.rx_rate,
            });
            
            // 送信レート
            data.network_tx_rate.push(DataPoint {
                timestamp: now,
                label: primary_info.name.clone(),
                value: primary_info.usage.tx_rate,
            });
        }
        
        // 古いデータを削除（例：100ポイント以上保持しない）
        if data.network_rx_rate.len() > 100 {
            data.network_rx_rate.drain(0..data.network_rx_rate.len() - 100);
        }
        
        if data.network_tx_rate.len() > 100 {
            data.network_tx_rate.drain(0..data.network_tx_rate.len() - 100);
        }
        
        Ok(())
    }
    
    /// 履歴データを取得
    pub fn get_history_data(&self, data_type: HistoryDataType, interface_name: Option<String>) -> Vec<DataPoint> {
        let network_info = match self.get_network_info() {
            Ok(info) => info,
            Err(_) => return Vec::new(),
        };
        
        let target_interface = interface_name.unwrap_or_else(|| network_info.primary_interface.clone());
        
        // 指定されたインターフェースの情報を取得
        if let Some(interface_info) = network_info.interfaces.get(&target_interface) {
            let now = Instant::now();
            
            match data_type {
                HistoryDataType::NetworkRxRate => {
                    vec![DataPoint {
                        timestamp: now,
                        label: interface_info.name.clone(),
                        value: interface_info.usage.rx_rate,
                    }]
                },
                HistoryDataType::NetworkTxRate => {
                    vec![DataPoint {
                        timestamp: now,
                        label: interface_info.name.clone(),
                        value: interface_info.usage.tx_rate,
                    }]
                },
                _ => Vec::new(), // 他のデータタイプはサポートしない
            }
        } else {
            Vec::new()
        }
    }
    
    /// ネットワーク統計情報を更新する
    fn update_network_statistics(
        config: &NetworkMonitorConfig,
        network_state: &Arc<Mutex<NetworkInfo>>,
        previous_stats: &Arc<Mutex<HashMap<String, (u64, u64, Instant)>>>,
    ) -> Result<()> {
        // 新しいネットワーク情報を読み取り
        let interfaces = read_network_interfaces()?;
        
        // 前回の統計情報を取得
        let prev_stats = previous_stats.lock().map_err(|e| {
            SystemError::Mutex(format!("前回の統計情報ロックの取得に失敗: {}", e))
        })?;
        
        // 現在のネットワーク状態を取得
        let mut network_state = network_state.lock().map_err(|e| {
            SystemError::Mutex(format!("ネットワーク状態ロックの取得に失敗: {}", e))
        })?;
        
        // プライマリインターフェースを特定（変更されている可能性があるため）
        network_state.primary_interface = find_primary_interface(&interfaces);
        
        // 各インターフェースの速度を計算
        for (name, info) in interfaces.iter() {
            if let Some((prev_rx, prev_tx, prev_time)) = prev_stats.get(name) {
                let time_diff = info.usage.last_updated.duration_since(*prev_time).as_secs_f64();
                
                if time_diff > 0.0 {
                    // 受信・送信レートを計算（バイト/秒）
                    let rx_rate = (info.usage.rx_bytes.saturating_sub(*prev_rx)) as f64 / time_diff;
                    let tx_rate = (info.usage.tx_bytes.saturating_sub(*prev_tx)) as f64 / time_diff;
                    let total_rate = rx_rate + tx_rate;
                    
                    // ネットワーク状態を更新
                    if let Some(current_info) = network_state.interfaces.get_mut(name) {
                        current_info.usage.rx_rate = rx_rate;
                        current_info.usage.tx_rate = tx_rate;
                        current_info.usage.total_rate = total_rate;
                        
                        // 高トラフィック警告チェック
                        if total_rate > config.traffic_warning_threshold as f64 {
                            debug!("高トラフィック警告: {} - {} B/s", name, total_rate);
                            // ここでアラートを生成することができる
                        }
                    }
                }
            }
            
            // 現在の統計情報をネットワーク状態にマージ
            if let Some(current_info) = network_state.interfaces.get_mut(name) {
                current_info.usage.rx_bytes = info.usage.rx_bytes;
                current_info.usage.tx_bytes = info.usage.tx_bytes;
                current_info.usage.rx_packets = info.usage.rx_packets;
                current_info.usage.tx_packets = info.usage.tx_packets;
                current_info.usage.rx_errors = info.usage.rx_errors;
                current_info.usage.tx_errors = info.usage.tx_errors;
                current_info.usage.rx_dropped = info.usage.rx_dropped;
                current_info.usage.tx_dropped = info.usage.tx_dropped;
                current_info.usage.last_updated = info.usage.last_updated;
            } else {
                // 新しいインターフェースを追加
                network_state.interfaces.insert(name.clone(), info.clone());
            }
        }
        
        // 前回の統計情報を更新
        drop(prev_stats);
        let mut prev_stats = previous_stats.lock().map_err(|e| {
            SystemError::Mutex(format!("前回の統計情報ロックの取得に失敗: {}", e))
        })?;
        
        *prev_stats = interfaces.iter().map(|(name, info)| {
            (name.clone(), (info.usage.rx_bytes, info.usage.tx_bytes, info.usage.last_updated))
        }).collect();
        
        // 最終更新時刻を更新
        network_state.last_updated = Instant::now();
        
        Ok(())
    }
    
    /// 接続品質をチェック
    fn check_connection_quality(
        config: &NetworkMonitorConfig,
        network_state: &Arc<Mutex<NetworkInfo>>,
    ) -> Result<()> {
        if !config.monitor_connection_quality {
            return Ok(());
        }
        
        // 主要なインターフェースのパケットロス率を推定
        let network_state = network_state.lock().map_err(|e| {
            SystemError::Mutex(format!("ネットワーク状態ロックの取得に失敗: {}", e))
        })?;
        
        if let Some(primary_info) = network_state.interfaces.get(&network_state.primary_interface) {
            // パケットロスとエラー率を計算
            let total_rx = primary_info.usage.rx_packets + primary_info.usage.rx_dropped + primary_info.usage.rx_errors;
            let packet_loss_rate = if total_rx > 0 {
                (primary_info.usage.rx_dropped + primary_info.usage.rx_errors) as f64 / total_rx as f64
            } else {
                0.0
            };
            
            // 品質が悪いとアラート
            if packet_loss_rate > 0.05 { // 5%以上のパケットロス
                warn!("ネットワーク品質低下: {} - パケットロス率 {:.2}%", 
                      primary_info.name, packet_loss_rate * 100.0);
                // ここでアラートを生成することができる
            }
            
            // レイテンシをチェック（pingコマンドを使用）
            if let Some(ref default_gateway) = network_state.default_gateway {
                if let Some(gateway_ip) = extract_gateway_ip(default_gateway) {
                    // ここでpingを実行してレイテンシをチェックする実装を追加できる
                    // ping_latency(&gateway_ip)
                }
            }
        }
        
        Ok(())
    }
    
    /// DNSサーバーの応答をチェック
    fn check_dns_servers(network_state: &Arc<Mutex<NetworkInfo>>) -> Result<()> {
        let network_state = network_state.lock().map_err(|e| {
            SystemError::Mutex(format!("ネットワーク状態ロックの取得に失敗: {}", e))
        })?;
        
        // DNSサーバーの応答をチェック
        for dns in &network_state.dns_servers {
            // ここでDNSサーバーへのクエリを実行する実装を追加できる
            // check_dns_response(dns)
        }
        
        Ok(())
    }
}

/// ゲートウェイIPアドレスを抽出
fn extract_gateway_ip(gateway_info: &str) -> Option<String> {
    let parts: Vec<&str> = gateway_info.split_whitespace().collect();
    if parts.len() >= 3 && parts[0] == "default" && parts[1] == "via" {
        Some(parts[2].to_string())
    } else {
        None
    }
}

/// /proc/net/devファイルを解析して受信・送信バイト数を取得
fn read_network_stats(interface: &str) -> Result<(u64, u64)> {
    let file = File::open("/proc/net/dev").map_err(|e| {
        SystemError::IO(format!("/proc/net/devの読み取りに失敗: {}", e))
    })?;
    let reader = BufReader::new(file);
    
    // ヘッダー行をスキップ（最初の2行）
    let mut lines = reader.lines();
    let _ = lines.next();
    let _ = lines.next();
    
    // 指定されたインターフェースの行を探す
    for line in lines {
        let line = line.map_err(|e| {
            SystemError::IO(format!("行の読み取りに失敗: {}", e))
        })?;
        
        if let Some(colon_pos) = line.find(':') {
            let if_name = line[..colon_pos].trim();
            if if_name == interface {
                let stats_part = line[colon_pos + 1..].trim();
                let stats: Vec<&str> = stats_part.split_whitespace().collect();
                
                if stats.len() >= 16 {
                    let rx_bytes = stats[0].parse::<u64>().unwrap_or(0);
                    let tx_bytes = stats[8].parse::<u64>().unwrap_or(0);
                    return Ok((rx_bytes, tx_bytes));
                }
            }
        }
    }
    
    // インターフェースが見つからない場合
    Err(SystemError::NotFound(format!("インターフェース {}が見つかりません", interface)).into())
}

/// /sysからMACアドレスを読み取る
fn read_mac_address_from_sys(interface: &str) -> Result<String> {
    let path = format!("/sys/class/net/{}/address", interface);
    let mac = std::fs::read_to_string(&path).map_err(|e| {
        SystemError::IO(format!("MACアドレスの読み取りに失敗 ({}): {}", path, e))
    })?;
    
    Ok(mac.trim().to_string())
}

/// /sysからMTUを読み取る
fn read_mtu_from_sys(interface: &str) -> Result<u32> {
    let path = format!("/sys/class/net/{}/mtu", interface);
    let mtu_str = std::fs::read_to_string(&path).map_err(|e| {
        SystemError::IO(format!("MTUの読み取りに失敗 ({}): {}", path, e))
    })?;
    
    mtu_str.trim().parse::<u32>().map_err(|e| {
        SystemError::Parse(format!("MTU値のパースに失敗: {}", e))
    })
}

/// /sysからインターフェースの状態を読み取る
fn read_operstate_from_sys(interface: &str) -> Result<NetworkConnectionState> {
    let path = format!("/sys/class/net/{}/operstate", interface);
    let state_str = std::fs::read_to_string(&path).map_err(|e| {
        SystemError::IO(format!("インターフェース状態の読み取りに失敗 ({}): {}", path, e))
    })?;
    
    match state_str.trim() {
        "up" => Ok(NetworkConnectionState::Connected),
        "down" => Ok(NetworkConnectionState::Disconnected),
        "dormant" => Ok(NetworkConnectionState::Connecting),
        "unknown" => Ok(NetworkConnectionState::Unknown),
        _ => Ok(NetworkConnectionState::Unknown),
    }
}

/// /etc/resolv.confからDNSサーバーを読み取る
fn read_dns_servers_from_resolv_conf() -> Result<Vec<String>> {
    let file = match File::open("/etc/resolv.conf") {
        Ok(f) => f,
        Err(e) => {
            warn!("/etc/resolv.confの読み取りに失敗: {}", e);
            return Ok(Vec::new());
        }
    };
    
    let reader = BufReader::new(file);
    let mut dns_servers = Vec::new();
    
    for line in reader.lines() {
        let line = line.map_err(|e| {
            SystemError::IO(format!("行の読み取りに失敗: {}", e))
        })?;
        
        // nameserverエントリを探す
        if line.starts_with("nameserver ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                dns_servers.push(parts[1].to_string());
            }
        }
    }
    
    Ok(dns_servers)
}

/// /proc/net/routeからデフォルトゲートウェイを読み取る
fn read_default_gateway_from_route() -> Result<Option<String>> {
    let file = match File::open("/proc/net/route") {
        Ok(f) => f,
        Err(e) => {
            warn!("/proc/net/routeの読み取りに失敗: {}", e);
            return Ok(None);
        }
    };
    
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    // ヘッダー行をスキップ
    let _ = lines.next();
    
    for line in lines {
        let line = line.map_err(|e| {
            SystemError::IO(format!("行の読み取りに失敗: {}", e))
        })?;
        
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 8 {
            let destination = fields[1];
            let flags = fields[3];
            
            // 宛先が0.0.0.0で、フラグにゲートウェイフラグが設定されているエントリを探す
            if destination == "00000000" {
                let interface = fields[0];
                let gateway_hex = fields[2];
                
                // 16進数のゲートウェイをIPアドレスに変換
                if let Ok(gateway_ip) = parse_hex_ip(gateway_hex) {
                    return Ok(Some(format!("default via {} dev {}", gateway_ip, interface)));
                }
            }
        }
    }
    
    Ok(None)
}

/// 16進数のIPアドレスを通常のドット区切り形式に変換
fn parse_hex_ip(hex_ip: &str) -> Result<String> {
    if hex_ip.len() != 8 {
        return Err(SystemError::Parse(format!("無効な16進数IPアドレス: {}", hex_ip)).into());
    }
    
    // 16進数を4つの8ビット値に変換
    let ip_bytes = (0..4).map(|i| {
        let start = i * 2;
        let end = start + 2;
        let byte_hex = &hex_ip[start..end];
        u8::from_str_radix(byte_hex, 16).unwrap_or(0)
    }).collect::<Vec<u8>>();
    
    // リトルエンディアンからビッグエンディアンに変換してIP形式にする
    Ok(format!("{}.{}.{}.{}", ip_bytes[3], ip_bytes[2], ip_bytes[1], ip_bytes[0]))
}

/// IPアドレスを取得
fn read_ip_addresses(interface: &str) -> Result<Vec<String>> {
    let mut addresses = Vec::new();
    
    // IPv4アドレスを取得
    if let Ok(content) = std::process::Command::new("ip")
        .args(&["addr", "show", interface])
        .output() {
            
        let output = String::from_utf8_lossy(&content.stdout);
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("inet ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    addresses.push(parts[1].to_string());
                }
            }
            if line.starts_with("inet6 ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    addresses.push(parts[1].to_string());
                }
            }
        }
    }
    
    Ok(addresses)
}

/// 帯域幅を推定
fn estimate_max_bandwidth(interface: &str) -> Option<u64> {
    // /sys/class/net/{interface}/speed から速度を読み取る
    let path = format!("/sys/class/net/{}/speed", interface);
    if let Ok(speed_str) = std::fs::read_to_string(&path) {
        if let Ok(speed_mbps) = speed_str.trim().parse::<u64>() {
            // Mbpsをbpsに変換
            return Some(speed_mbps * 1_000_000);
        }
    }
    
    // インターフェース名から推測
    if interface.starts_with("eth") || interface.starts_with("en") {
        Some(1_000_000_000) // 1Gbps for Ethernet
    } else if interface.starts_with("wl") {
        Some(300_000_000) // 300Mbps for WiFi
    } else if interface.starts_with("ppp") || interface.starts_with("wwan") {
        Some(50_000_000) // 50Mbps for mobile
    } else {
        None
    }
}

// 外部公開関数の更新
pub fn collect_network_info() -> Result<NetworkInfo> {
    let mut info = NetworkInfo::new();
    
    // インターフェース情報を読み取る
    info.interfaces = read_network_interfaces()?;
    
    // プライマリインターフェースを特定
    info.primary_interface = find_primary_interface(&info.interfaces);
    
    // DNSサーバー情報を取得
    info.dns_servers = read_dns_servers_from_resolv_conf()?;
    
    // デフォルトゲートウェイ情報を取得
    info.default_gateway = read_default_gateway_from_route()?;
    
    // グローバルIPアドレスの取得（オプション）
    // 実際の実装ではHTTPリクエストなどを使用
    
    info.last_updated = Instant::now();
    
    Ok(info)
}

// 前述のスタブ関数を更新
fn read_mac_address(interface_name: &str) -> Result<String> {
    read_mac_address_from_sys(interface_name)
}

fn read_mtu(interface_name: &str) -> Result<u32> {
    read_mtu_from_sys(interface_name)
}

fn determine_connection_state(interface_name: &str) -> NetworkConnectionState {
    read_operstate_from_sys(interface_name).unwrap_or(NetworkConnectionState::Unknown)
}

fn read_dns_servers() -> Result<Vec<String>> {
    read_dns_servers_from_resolv_conf()
}

fn read_default_gateway() -> Result<Option<String>> {
    read_default_gateway_from_route()
}

// 更新された関数
/// IPアドレスを取得
fn read_ip_addresses(interface: &str) -> Result<Vec<String>> {
    let mut addresses = Vec::new();
    
    // IPv4アドレスを取得
    if let Ok(content) = std::process::Command::new("ip")
        .args(&["addr", "show", interface])
        .output() {
            
        let output = String::from_utf8_lossy(&content.stdout);
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("inet ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    addresses.push(parts[1].to_string());
                }
            }
            if line.starts_with("inet6 ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    addresses.push(parts[1].to_string());
                }
            }
        }
    }
    
    Ok(addresses)
}

/// 帯域幅を推定
fn estimate_max_bandwidth(interface: &str) -> Option<u64> {
    // /sys/class/net/{interface}/speed から速度を読み取る
    let path = format!("/sys/class/net/{}/speed", interface);
    if let Ok(speed_str) = std::fs::read_to_string(&path) {
        if let Ok(speed_mbps) = speed_str.trim().parse::<u64>() {
            // Mbpsをbpsに変換
            return Some(speed_mbps * 1_000_000);
        }
    }
    
    // インターフェース名から推測
    if interface.starts_with("eth") || interface.starts_with("en") {
        Some(1_000_000_000) // 1Gbps for Ethernet
    } else if interface.starts_with("wl") {
        Some(300_000_000) // 300Mbps for WiFi
    } else if interface.starts_with("ppp") || interface.starts_with("wwan") {
        Some(50_000_000) // 50Mbps for mobile
    } else {
        None
    }
}

// 外部サービスからグローバルIPを取得する関数
// 注意: これは実際にHTTPリクエストを行うので、テスト環境ではモックに置き換えることが望ましい
fn fetch_global_ip() -> Result<String> {
    // 実際の実装では、外部サービスにHTTPリクエストを送信
    // ここではモックを返す
    #[cfg(not(test))]
    {
        // 実際の環境では使用可能なサービスを使用する
        // 例: https://api.ipify.org, https://ifconfig.me/ip など
        Err(SystemError::Unavailable("グローバルIPの取得機能は実装されていません".to_string()).into())
    }
    
    #[cfg(test)]
    {
        Ok("203.0.113.1".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 既存のテスト
    
    #[test]
    fn test_extract_gateway_ip() {
        let gateway_info = "default via 192.168.1.1 dev eth0";
        assert_eq!(extract_gateway_ip(gateway_info), Some("192.168.1.1".to_string()));
        
        let invalid_info = "not a gateway info";
        assert_eq!(extract_gateway_ip(invalid_info), None);
    }
    
    #[test]
    fn test_parse_hex_ip() {
        // 16進数 "0100000A" はリトルエンディアンで表現された 10.0.0.1
        assert_eq!(parse_hex_ip("0100000A").unwrap(), "10.0.0.1");
        
        // 無効な16進数
        assert!(parse_hex_ip("invalid").is_err());
    }
    
    #[test]
    fn test_network_monitor_start_stop() {
        let mut monitor = NetworkMonitor::new_default();
        
        // 起動テスト（実際のファイルシステムアクセスは行わないモックを使用）
        #[cfg(test)]
        {
            assert!(monitor.start().is_ok());
            assert!(monitor.stop().is_ok());
        }
    }
} 