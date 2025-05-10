// LumosDesktop CPUモニターモジュール
// CPUの使用率、周波数、温度などの情報を収集・分析します

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::time::{Duration, Instant};
use std::thread;

/// CPU周波数情報
#[derive(Debug, Clone)]
pub struct CpuFrequency {
    /// 現在の周波数 (MHz)
    pub current: f64,
    /// 最小周波数 (MHz)
    pub min: f64,
    /// 最大周波数 (MHz)
    pub max: f64,
    /// ベース周波数 (MHz)
    pub base: f64,
}

/// CPUコア使用率情報
#[derive(Debug, Clone)]
pub struct CpuCoreUsage {
    /// コアID
    pub id: usize,
    /// ユーザーモードでの使用率 (%)
    pub user: f64,
    /// システムモードでの使用率 (%)
    pub system: f64,
    /// アイドル状態の割合 (%)
    pub idle: f64,
    /// IO待ち状態の割合 (%)
    pub iowait: f64,
    /// 他の使用状態の割合 (%)
    pub other: f64,
}

/// CPU使用率情報
#[derive(Debug, Clone)]
pub struct CpuUsage {
    /// 全体的なCPU使用率 (%)
    pub total: f64,
    /// ユーザーモードでの使用率 (%)
    pub user: f64,
    /// システムモードでの使用率 (%)
    pub system: f64,
    /// アイドル状態の割合 (%)
    pub idle: f64,
    /// IO待ち状態の割合 (%)
    pub iowait: f64,
    /// コアごとの使用率
    pub cores: Vec<CpuCoreUsage>,
    /// 過去1分間の平均負荷
    pub load_avg_1min: Option<f64>,
    /// 過去5分間の平均負荷
    pub load_avg_5min: Option<f64>,
    /// 過去15分間の平均負荷
    pub load_avg_15min: Option<f64>,
}

/// CPU基本情報
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// CPU名（モデル名）
    pub model_name: String,
    /// ベンダー（製造元）
    pub vendor: String,
    /// 物理コア数
    pub physical_cores: usize,
    /// 論理コア数（スレッド数）
    pub logical_cores: usize,
    /// ソケット数
    pub sockets: usize,
    /// キャッシュサイズ（階層ごと、L1/L2/L3）
    pub cache_size: HashMap<String, u64>,
    /// CPU周波数情報
    pub frequency: CpuFrequency,
    /// 使用率情報
    pub usage: CpuUsage,
    /// 温度情報（摂氏）、利用できない場合はNone
    pub temperature: Option<f64>,
    /// プロセッサフラグ（対応機能）
    pub flags: Vec<String>,
    /// 最終更新時刻
    pub last_updated: Instant,
}

// CPU状態を追跡するための内部構造体
#[derive(Debug, Clone)]
struct CpuStat {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
    guest: u64,
    guest_nice: u64,
    total: u64,
}

impl CpuStat {
    // /proc/statからの行をパース
    fn from_proc_stat(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 8 || !parts[0].starts_with("cpu") {
            return None;
        }

        // 各カウンタを抽出し、失敗したら0を割り当て
        let user = parts.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let nice = parts.get(2).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let system = parts.get(3).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let idle = parts.get(4).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let iowait = parts.get(5).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let irq = parts.get(6).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let softirq = parts.get(7).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        
        // オプションのカウンタを抽出（カーネルによって利用可能かどうかが異なる）
        let steal = parts.get(8).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let guest = parts.get(9).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let guest_nice = parts.get(10).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        // 合計値を計算
        let total = user + nice + system + idle + iowait + irq + softirq + steal + guest + guest_nice;

        Some(Self {
            user,
            nice,
            system,
            idle,
            iowait,
            irq,
            softirq,
            steal,
            guest,
            guest_nice,
            total,
        })
    }

    // 2つのCpuStat間の使用率を計算
    fn calculate_usage(&self, prev: &Self) -> (f64, f64, f64, f64, f64) {
        // 各カウンタの差分を計算
        let user_diff = self.user.saturating_sub(prev.user) as f64;
        let nice_diff = self.nice.saturating_sub(prev.nice) as f64;
        let system_diff = self.system.saturating_sub(prev.system) as f64;
        let idle_diff = self.idle.saturating_sub(prev.idle) as f64;
        let iowait_diff = self.iowait.saturating_sub(prev.iowait) as f64;
        let irq_diff = self.irq.saturating_sub(prev.irq) as f64;
        let softirq_diff = self.softirq.saturating_sub(prev.softirq) as f64;
        let steal_diff = self.steal.saturating_sub(prev.steal) as f64;
        let guest_diff = self.guest.saturating_sub(prev.guest) as f64;
        let guest_nice_diff = self.guest_nice.saturating_sub(prev.guest_nice) as f64;

        // 合計の差分を計算
        let total_diff = self.total.saturating_sub(prev.total) as f64;
        
        if total_diff == 0.0 {
            return (0.0, 0.0, 0.0, 0.0, 0.0);
        }

        // 各状態のパーセンテージを計算
        let user_percent = ((user_diff + nice_diff) / total_diff) * 100.0;
        let system_percent = ((system_diff + irq_diff + softirq_diff) / total_diff) * 100.0;
        let idle_percent = (idle_diff / total_diff) * 100.0;
        let iowait_percent = (iowait_diff / total_diff) * 100.0;
        let other_percent = ((steal_diff + guest_diff + guest_nice_diff) / total_diff) * 100.0;

        (user_percent, system_percent, idle_percent, iowait_percent, other_percent)
    }
}

/// 現在のCPU情報を収集
pub fn collect_cpu_info() -> Result<CpuInfo, io::Error> {
    // モデル名、ベンダー、コア数などの基本情報を読み取り
    let (model_name, vendor, flags) = read_cpu_info()?;
    
    // 物理/論理コア数を判定
    let (physical_cores, logical_cores, sockets) = count_cpu_cores()?;
    
    // キャッシュサイズを取得
    let cache_size = read_cache_info()?;
    
    // CPU周波数情報を取得
    let frequency = read_cpu_frequency()?;
    
    // 使用率を2回計測して計算（前回と今回の差分が必要）
    let prev_stats = read_cpu_stats()?;
    thread::sleep(Duration::from_millis(100)); // 短い遅延でサンプリング
    let current_stats = read_cpu_stats()?;
    
    // 使用率を計算
    let usage = calculate_cpu_usage(&prev_stats, &current_stats)?;
    
    // 温度を取得
    let temperature = read_cpu_temperature();
    
    Ok(CpuInfo {
        model_name,
        vendor,
        physical_cores,
        logical_cores,
        sockets,
        cache_size,
        frequency,
        usage,
        temperature,
        flags,
        last_updated: Instant::now(),
    })
}

/// CPU基本情報を/proc/cpuinfoから読み取り
fn read_cpu_info() -> Result<(String, String, Vec<String>), io::Error> {
    let file = File::open("/proc/cpuinfo")?;
    let reader = BufReader::new(file);

    let mut model_name = String::new();
    let mut vendor = String::new();
    let mut flags = Vec::new();

    for line in reader.lines() {
        let line = line?;
        
        if line.starts_with("model name") {
            if let Some(value) = line.split(':').nth(1) {
                model_name = value.trim().to_string();
            }
        } else if line.starts_with("vendor_id") {
            if let Some(value) = line.split(':').nth(1) {
                vendor = value.trim().to_string();
            }
        } else if line.starts_with("flags") {
            if let Some(value) = line.split(':').nth(1) {
                flags = value.trim().split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                // フラグは通常1行だけなので、これ以上処理する必要はない
                break;
            }
        }
    }

    if model_name.is_empty() {
        model_name = "Unknown".to_string();
    }
    
    if vendor.is_empty() {
        vendor = "Unknown".to_string();
    }

    Ok((model_name, vendor, flags))
}

/// CPU実コア数と論理コア数、ソケット数をカウント
fn count_cpu_cores() -> Result<(usize, usize, usize), io::Error> {
    let file = File::open("/proc/cpuinfo")?;
    let reader = BufReader::new(file);

    let mut physical_ids = std::collections::HashSet::new();
    let mut core_ids = std::collections::HashSet::new();
    let mut logical_cores = 0;
    let mut current_physical_id = None;

    for line in reader.lines() {
        let line = line?;
        
        if line.starts_with("processor") {
            logical_cores += 1;
        } else if line.starts_with("physical id") {
            if let Some(value) = line.split(':').nth(1) {
                let physical_id = value.trim().to_string();
                physical_ids.insert(physical_id.clone());
                current_physical_id = Some(physical_id);
            }
        } else if line.starts_with("core id") {
            if let Some(value) = line.split(':').nth(1) {
                if let Some(physical_id) = &current_physical_id {
                    // 物理ID+コアIDの組み合わせを使用して、物理コアを一意に識別
                    let combined_id = format!("{}_{}", physical_id, value.trim());
                    core_ids.insert(combined_id);
                }
            }
        } else if line.is_empty() {
            current_physical_id = None;
        }
    }

    // 物理コア数がゼロの場合、論理コア数を代わりに使用
    let physical_cores = if core_ids.is_empty() { logical_cores } else { core_ids.len() };
    
    // ソケット数は物理IDの数
    let sockets = if physical_ids.is_empty() { 1 } else { physical_ids.len() };

    Ok((physical_cores, logical_cores, sockets))
}

/// CPUキャッシュ情報を読み取り
fn read_cache_info() -> Result<HashMap<String, u64>, io::Error> {
    let mut cache_info = HashMap::new();
    
    // Linuxシステムの場合は/sys/devices/system/cpu/cpu0/cache/からキャッシュ情報を読み取る
    let cache_path = "/sys/devices/system/cpu/cpu0/cache";
    
    if let Ok(entries) = std::fs::read_dir(cache_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(index_str) = path.file_name().and_then(|n| n.to_str()) {
                        // インデックスディレクトリの場合（index0, index1, index2など）
                        let level_path = path.join("level");
                        let size_path = path.join("size");
                        let type_path = path.join("type");
                        
                        // キャッシュレベルを読み取り（L1, L2, L3）
                        let level = if let Ok(content) = std::fs::read_to_string(&level_path) {
                            content.trim().to_string()
                        } else {
                            continue;
                        };
                        
                        // キャッシュタイプを読み取り（Data, Instruction, Unified）
                        let cache_type = if let Ok(content) = std::fs::read_to_string(&type_path) {
                            content.trim().to_string()
                        } else {
                            "Unknown".to_string()
                        };
                        
                        // キャッシュサイズを読み取り
                        if let Ok(content) = std::fs::read_to_string(&size_path) {
                            let key = format!("L{} {}", level, cache_type);
                            // サイズ文字列をパース（例: "32K" -> 32768）
                            let size_str = content.trim();
                            if let Some(size_bytes) = parse_size_string(size_str) {
                                cache_info.insert(key, size_bytes);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(cache_info)
}

/// サイズ文字列をバイト数に変換（例: "32K" -> 32768）
fn parse_size_string(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim().to_uppercase();
    
    // 数値部分と単位を分離
    let mut size_value = 0.0;
    let mut unit = "";
    
    for (i, c) in size_str.char_indices() {
        if c.is_alphabetic() {
            if let Ok(value) = size_str[..i].parse::<f64>() {
                size_value = value;
                unit = &size_str[i..];
                break;
            }
        }
    }
    
    if unit.is_empty() {
        // 単位がない場合は直接数値に変換を試みる
        return size_str.parse::<u64>().ok();
    }
    
    // 単位に基づいてバイト数に変換
    let multiplier = match unit {
        "K" | "KB" => 1024,
        "M" | "MB" => 1024 * 1024,
        "G" | "GB" => 1024 * 1024 * 1024,
        _ => return None,
    };
    
    Some((size_value * multiplier as f64) as u64)
}

/// CPU周波数情報を読み取り
fn read_cpu_frequency() -> Result<CpuFrequency, io::Error> {
    let mut current = 0.0;
    let mut min = 0.0;
    let mut max = 0.0;
    let mut base = 0.0;
    
    // 現在の周波数を読み取り
    if let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") {
        if let Ok(freq) = content.trim().parse::<f64>() {
            current = freq / 1000.0; // KHzからMHzに変換
        }
    }
    
    // 最小周波数を読み取り
    if let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq") {
        if let Ok(freq) = content.trim().parse::<f64>() {
            min = freq / 1000.0; // KHzからMHzに変換
        }
    }
    
    // 最大周波数を読み取り
    if let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq") {
        if let Ok(freq) = content.trim().parse::<f64>() {
            max = freq / 1000.0; // KHzからMHzに変換
        }
    }
    
    // ベース周波数を取得（利用可能な場合）
    if let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/base_frequency") {
        if let Ok(freq) = content.trim().parse::<f64>() {
            base = freq / 1000.0; // KHzからMHzに変換
        }
    } else if let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
        // base_frequencyがない場合はcpuinfo_max_freqを代用
        if let Ok(freq) = content.trim().parse::<f64>() {
            base = freq / 1000.0; // KHzからMHzに変換
        }
    }
    
    // ファイルから取得できない場合はコマンドライン出力を試す（バックアップ手段）
    if current == 0.0 {
        // 存在しているかどうか確認してから実行する
        if std::process::Command::new("lscpu").output().is_ok() {
            if let Ok(output) = std::process::Command::new("lscpu").output() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("CPU MHz") {
                            if let Some(value) = line.split(':').nth(1) {
                                if let Ok(freq) = value.trim().parse::<f64>() {
                                    current = freq;
                                }
                            }
                        } else if base == 0.0 && line.contains("CPU max MHz") {
                            if let Some(value) = line.split(':').nth(1) {
                                if let Ok(freq) = value.trim().parse::<f64>() {
                                    base = freq;
                                    if max == 0.0 {
                                        max = freq;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // それでも不明な場合はデフォルト値を提供
    if current == 0.0 {
        current = 1000.0; // 適当なデフォルト値（1GHz）
    }
    
    if min == 0.0 {
        min = current / 2.0; // 現在の周波数の半分をデフォルトとして設定
    }
    
    if max == 0.0 {
        max = current * 1.5; // 現在の周波数の1.5倍をデフォルトとして設定
    }
    
    if base == 0.0 {
        base = max; // 最大周波数をベース周波数のデフォルトとして設定
    }
    
    Ok(CpuFrequency {
        current,
        min,
        max,
        base,
    })
}

/// CPUの温度情報を取得
fn read_cpu_temperature() -> Option<f64> {
    // 複数の可能なパスをチェック
    let possible_paths = [
        // hwmonパス（最も一般的）
        "/sys/class/hwmon/hwmon0/temp1_input",
        "/sys/class/hwmon/hwmon1/temp1_input",
        "/sys/class/hwmon/hwmon2/temp1_input",
        // 特定のCPUセンサー
        "/sys/class/thermal/thermal_zone0/temp",
        "/sys/devices/platform/coretemp.0/temp1_input",
        // k10tempモジュールを使用するAMDプロセッサー向け
        "/sys/devices/pci0000:00/0000:00:18.3/hwmon/hwmon0/temp1_input",
    ];
    
    for path in &possible_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(temp) = content.trim().parse::<f64>() {
                // 多くのシステムでは温度が1000倍で格納されているため、
                // 1000で割って摂氏に変換
                if temp > 1000.0 {
                    return Some(temp / 1000.0);
                } else {
                    return Some(temp);
                }
            }
        }
    }
    
    // バックアップとして「sensors」コマンドを試す
    if let Ok(output) = std::process::Command::new("sensors").output() {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            for line in output_str.lines() {
                // 一般的なCPU温度出力パターンを検索
                if line.contains("Core") && line.contains("°C") {
                    if let Some(temp_part) = line.split(':').nth(1) {
                        if let Some(temp_str) = temp_part.split('°').next() {
                            if let Ok(temp) = temp_str.trim().parse::<f64>() {
                                return Some(temp);
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// CPU統計情報を/proc/statから読み取り
fn read_cpu_stats() -> Result<HashMap<String, CpuStat>, io::Error> {
    let file = File::open("/proc/stat")?;
    let reader = BufReader::new(file);
    let mut stats = HashMap::new();
    
    for line in reader.lines() {
        let line = line?;
        
        if line.starts_with("cpu") {
            let cpu_id = line.split_whitespace().next().unwrap_or("cpu");
            if let Some(stat) = CpuStat::from_proc_stat(&line) {
                stats.insert(cpu_id.to_string(), stat);
            }
        }
    }
    
    Ok(stats)
}

/// CPU使用率を計算
fn calculate_cpu_usage(
    prev_stats: &HashMap<String, CpuStat>,
    current_stats: &HashMap<String, CpuStat>
) -> Result<CpuUsage, io::Error> {
    let mut cores = Vec::new();
    
    // 全体的なCPU使用率（cpu行）
    let (user, system, idle, iowait, other) = if let (Some(prev), Some(current)) = (prev_stats.get("cpu"), current_stats.get("cpu")) {
        current.calculate_usage(prev)
    } else {
        (0.0, 0.0, 0.0, 0.0, 0.0)
    };
    
    // 各コアの使用率（cpu0, cpu1, ...）
    for (key, current_stat) in current_stats {
        if key == "cpu" || !key.starts_with("cpu") {
            continue;
        }
        
        if let Some(core_id_str) = key.strip_prefix("cpu") {
            if let Ok(core_id) = core_id_str.parse::<usize>() {
                if let Some(prev_stat) = prev_stats.get(key) {
                    let (core_user, core_system, core_idle, core_iowait, core_other) = 
                        current_stat.calculate_usage(prev_stat);
                    
                    cores.push(CpuCoreUsage {
                        id: core_id,
                        user: core_user,
                        system: core_system,
                        idle: core_idle,
                        iowait: core_iowait,
                        other: core_other,
                    });
                }
            }
        }
    }
    
    // コアIDでソート
    cores.sort_by_key(|core| core.id);
    
    // ロードアベレージの読み取り
    let (load_avg_1min, load_avg_5min, load_avg_15min) = read_load_average()?;
    
    let total_usage = user + system + iowait + other;
    
    Ok(CpuUsage {
        total: total_usage,
        user,
        system,
        idle,
        iowait,
        cores,
        load_avg_1min: Some(load_avg_1min),
        load_avg_5min: Some(load_avg_5min),
        load_avg_15min: Some(load_avg_15min),
    })
}

/// ロードアベレージを/proc/loadavgから読み取り
fn read_load_average() -> Result<(f64, f64, f64), io::Error> {
    let content = std::fs::read_to_string("/proc/loadavg")?;
    let parts: Vec<&str> = content.split_whitespace().collect();
    
    let load_1min = parts.get(0).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let load_5min = parts.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let load_15min = parts.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    
    Ok((load_1min, load_5min, load_15min))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("32K"), Some(32 * 1024));
        assert_eq!(parse_size_string("2M"), Some(2 * 1024 * 1024));
        assert_eq!(parse_size_string("1G"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_size_string("16KB"), Some(16 * 1024));
        assert_eq!(parse_size_string("512MB"), Some(512 * 1024 * 1024));
        assert_eq!(parse_size_string("1024"), Some(1024));
        assert_eq!(parse_size_string("invalid"), None);
    }
    
    #[test]
    fn test_cpu_stat_calculation() {
        let prev = CpuStat {
            user: 100,
            nice: 10,
            system: 50,
            idle: 800,
            iowait: 20,
            irq: 5,
            softirq: 5,
            steal: 0,
            guest: 0,
            guest_nice: 0,
            total: 990,
        };
        
        let current = CpuStat {
            user: 150,
            nice: 15,
            system: 70,
            idle: 900,
            iowait: 30,
            irq: 10,
            softirq: 10,
            steal: 5,
            guest: 0,
            guest_nice: 0,
            total: 1190,
        };
        
        let (user, system, idle, iowait, other) = current.calculate_usage(&prev);
        
        // 計算結果を検証 (手動で計算した期待値との比較)
        let expected_user = ((150 - 100) + (15 - 10)) as f64 / (1190 - 990) as f64 * 100.0;
        let expected_system = ((70 - 50) + (10 - 5) + (10 - 5)) as f64 / (1190 - 990) as f64 * 100.0;
        let expected_idle = (900 - 800) as f64 / (1190 - 990) as f64 * 100.0;
        let expected_iowait = (30 - 20) as f64 / (1190 - 990) as f64 * 100.0;
        let expected_other = (5 - 0) as f64 / (1190 - 990) as f64 * 100.0;
        
        assert!((user - expected_user).abs() < 0.01);
        assert!((system - expected_system).abs() < 0.01);
        assert!((idle - expected_idle).abs() < 0.01);
        assert!((iowait - expected_iowait).abs() < 0.01);
        assert!((other - expected_other).abs() < 0.01);
    }
    
    #[test]
    fn test_collect_cpu_info() {
        // 実際のCPU情報を収集してみる
        // この部分はシステムに依存するため、最小限のチェックのみ実施
        match collect_cpu_info() {
            Ok(info) => {
                // 基本的な整合性チェック
                assert!(!info.model_name.is_empty());
                assert!(info.logical_cores > 0);
                assert!(info.physical_cores > 0);
                assert!(info.physical_cores <= info.logical_cores);
                assert!(info.sockets > 0);
                assert!(info.frequency.current > 0.0);
                
                // 使用率が0-100%の範囲内かチェック
                assert!(info.usage.total >= 0.0 && info.usage.total <= 100.0);
                assert!(info.usage.user >= 0.0 && info.usage.user <= 100.0);
                assert!(info.usage.system >= 0.0 && info.usage.system <= 100.0);
                assert!(info.usage.idle >= 0.0 && info.usage.idle <= 100.0);
                
                // 各コアの使用率も0-100%の範囲内かチェック
                for core in &info.usage.cores {
                    assert!(core.user >= 0.0 && core.user <= 100.0);
                    assert!(core.system >= 0.0 && core.system <= 100.0);
                    assert!(core.idle >= 0.0 && core.idle <= 100.0);
                }
            },
            Err(e) => {
                // テスト環境によってはエラーになる可能性もあるため、
                // エラーメッセージを出力してスキップ
                eprintln!("CPU情報収集エラー（環境依存のためスキップ）: {}", e);
            }
        }
    }
} 