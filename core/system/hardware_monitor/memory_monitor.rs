// LumosDesktop メモリモニターモジュール
// システムのメモリとスワップ使用状況を監視します

use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::collections::HashMap;

/// メモリサイズ単位を表す列挙体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryUnit {
    /// バイト
    Byte,
    /// キロバイト (1024バイト)
    KB,
    /// メガバイト (1024キロバイト)
    MB,
    /// ギガバイト (1024メガバイト)
    GB,
    /// テラバイト (1024ギガバイト)
    TB,
}

impl MemoryUnit {
    /// バイト数をこの単位に変換
    pub fn convert_from_bytes(&self, bytes: u64) -> f64 {
        match self {
            MemoryUnit::Byte => bytes as f64,
            MemoryUnit::KB => bytes as f64 / 1024.0,
            MemoryUnit::MB => bytes as f64 / (1024.0 * 1024.0),
            MemoryUnit::GB => bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            MemoryUnit::TB => bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0),
        }
    }
    
    /// この単位からバイトに変換
    pub fn convert_to_bytes(&self, value: f64) -> u64 {
        match self {
            MemoryUnit::Byte => value as u64,
            MemoryUnit::KB => (value * 1024.0) as u64,
            MemoryUnit::MB => (value * 1024.0 * 1024.0) as u64,
            MemoryUnit::GB => (value * 1024.0 * 1024.0 * 1024.0) as u64,
            MemoryUnit::TB => (value * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
        }
    }
    
    /// 適切な単位を自動的に選択
    pub fn auto_select(bytes: u64) -> Self {
        if bytes < 1024 {
            MemoryUnit::Byte
        } else if bytes < 1024 * 1024 {
            MemoryUnit::KB
        } else if bytes < 1024 * 1024 * 1024 {
            MemoryUnit::MB
        } else if bytes < 1024 * 1024 * 1024 * 1024 {
            MemoryUnit::GB
        } else {
            MemoryUnit::TB
        }
    }
    
    /// 単位の文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryUnit::Byte => "B",
            MemoryUnit::KB => "KB",
            MemoryUnit::MB => "MB",
            MemoryUnit::GB => "GB",
            MemoryUnit::TB => "TB",
        }
    }
}

/// メモリ使用状況
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// 合計物理メモリ (バイト)
    pub total_bytes: u64,
    /// 使用中の物理メモリ (バイト)
    pub used_bytes: u64,
    /// 空きメモリ (バイト)
    pub free_bytes: u64,
    /// 共有メモリ (バイト)
    pub shared_bytes: u64,
    /// バッファに使用されているメモリ (バイト)
    pub buffers_bytes: u64,
    /// キャッシュに使用されているメモリ (バイト)
    pub cached_bytes: u64,
    /// 利用可能なメモリ (バイト) - free + buffers + cached
    pub available_bytes: u64,
    /// 使用率 (%)
    pub used_percent: f64,
}

/// スワップ使用状況
#[derive(Debug, Clone)]
pub struct SwapUsage {
    /// 合計スワップサイズ (バイト)
    pub total_bytes: u64,
    /// 使用中のスワップ (バイト)
    pub used_bytes: u64,
    /// 空きスワップ (バイト)
    pub free_bytes: u64,
    /// スワップ使用率 (%)
    pub used_percent: f64,
}

/// HugePages情報
#[derive(Debug, Clone)]
pub struct HugePagesInfo {
    /// HugePageのサイズ (バイト)
    pub size_bytes: u64,
    /// 合計HugePage数
    pub total: u64,
    /// 空きHugePage数
    pub free: u64,
    /// 予約されたHugePage数
    pub reserved: u64,
    /// 使用中のHugePage数
    pub used: u64,
}

/// メモリ情報
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// メモリ使用状況
    pub usage: MemoryUsage,
    /// スワップ使用状況
    pub swap: SwapUsage,
    /// HugePages情報（利用可能な場合）
    pub huge_pages: Option<HugePagesInfo>,
    /// メモリ帯域幅 (バイト/秒) (利用可能な場合)
    pub bandwidth_bytes_per_sec: Option<u64>,
    /// 追加のプロパティ
    pub properties: HashMap<String, String>,
}

/// メモリ情報を収集
pub fn collect_memory_info() -> Result<MemoryInfo, io::Error> {
    let meminfo = read_proc_meminfo()?;
    
    // 基本的なメモリ情報を抽出
    let total_mem = get_value_from_meminfo(&meminfo, "MemTotal").unwrap_or(0) * 1024;
    let free_mem = get_value_from_meminfo(&meminfo, "MemFree").unwrap_or(0) * 1024;
    let available_mem = get_value_from_meminfo(&meminfo, "MemAvailable").unwrap_or_else(|| {
        // MemAvailableがない場合は代替計算
        free_mem / 1024 + 
        get_value_from_meminfo(&meminfo, "Buffers").unwrap_or(0) +
        get_value_from_meminfo(&meminfo, "Cached").unwrap_or(0)
    }) * 1024;
    
    let buffers = get_value_from_meminfo(&meminfo, "Buffers").unwrap_or(0) * 1024;
    let cached = get_value_from_meminfo(&meminfo, "Cached").unwrap_or(0) * 1024;
    let shared = get_value_from_meminfo(&meminfo, "Shmem").unwrap_or(0) * 1024;
    
    // 使用中のメモリを計算
    let used_mem = total_mem.saturating_sub(free_mem);
    let used_percent = if total_mem > 0 {
        (used_mem as f64 / total_mem as f64) * 100.0
    } else {
        0.0
    };
    
    // スワップ情報
    let total_swap = get_value_from_meminfo(&meminfo, "SwapTotal").unwrap_or(0) * 1024;
    let free_swap = get_value_from_meminfo(&meminfo, "SwapFree").unwrap_or(0) * 1024;
    let used_swap = total_swap.saturating_sub(free_swap);
    let swap_used_percent = if total_swap > 0 {
        (used_swap as f64 / total_swap as f64) * 100.0
    } else {
        0.0
    };
    
    // HugePages情報
    let huge_pages = get_hugepages_info(&meminfo);
    
    // その他のプロパティを収集
    let properties = collect_additional_properties(&meminfo);
    
    // メモリ帯域幅（Linux特有の/proc/buddyinfoから推定するか、利用可能ならSMBios情報から取得）
    let bandwidth = estimate_memory_bandwidth();
    
    let memory_usage = MemoryUsage {
        total_bytes: total_mem,
        used_bytes: used_mem,
        free_bytes: free_mem,
        shared_bytes: shared,
        buffers_bytes: buffers,
        cached_bytes: cached,
        available_bytes: available_mem,
        used_percent,
    };
    
    let swap_usage = SwapUsage {
        total_bytes: total_swap,
        used_bytes: used_swap,
        free_bytes: free_swap,
        used_percent: swap_used_percent,
    };
    
    Ok(MemoryInfo {
        usage: memory_usage,
        swap: swap_usage,
        huge_pages,
        bandwidth_bytes_per_sec: bandwidth,
        properties,
    })
}

/// /proc/meminfoからメモリ情報を読み取り
fn read_proc_meminfo() -> Result<HashMap<String, u64>, io::Error> {
    let file = File::open("/proc/meminfo")?;
    let reader = BufReader::new(file);
    let mut result = HashMap::new();
    
    for line in reader.lines() {
        let line = line?;
        // 行をキーと値に分割（例: "MemTotal:        8167964 kB"）
        if let Some((key, value_part)) = line.split_once(':') {
            // 数値部分を抽出
            let values: Vec<&str> = value_part.trim().split_whitespace().collect();
            if let Some(value_str) = values.first() {
                if let Ok(value) = value_str.parse::<u64>() {
                    result.insert(key.trim().to_string(), value);
                }
            }
        }
    }
    
    Ok(result)
}

/// メモリ情報から特定のキーの値を取得
fn get_value_from_meminfo(meminfo: &HashMap<String, u64>, key: &str) -> Option<u64> {
    meminfo.get(key).copied()
}

/// HugePages情報を取得
fn get_hugepages_info(meminfo: &HashMap<String, u64>) -> Option<HugePagesInfo> {
    let hugepage_size = get_value_from_meminfo(meminfo, "Hugepagesize")?;
    let hugepage_total = get_value_from_meminfo(meminfo, "HugePages_Total")?;
    let hugepage_free = get_value_from_meminfo(meminfo, "HugePages_Free")?;
    let hugepage_reserved = get_value_from_meminfo(meminfo, "HugePages_Rsvd").unwrap_or(0);
    
    // 使用中のページ数 = 合計 - 空き + 予約
    let hugepage_used = hugepage_total.saturating_sub(hugepage_free).saturating_add(hugepage_reserved);
    
    Some(HugePagesInfo {
        // サイズはKBで格納されているのでバイトに変換
        size_bytes: hugepage_size * 1024,
        total: hugepage_total,
        free: hugepage_free,
        reserved: hugepage_reserved,
        used: hugepage_used,
    })
}

/// 追加のメモリプロパティを収集
fn collect_additional_properties(meminfo: &HashMap<String, u64>) -> HashMap<String, String> {
    let mut properties = HashMap::new();
    
    // 興味深いプロパティのリスト
    let interesting_keys = [
        "Dirty", "Writeback", "AnonPages", "Mapped", "KernelStack",
        "PageTables", "CommitLimit", "Committed_AS", "VmallocTotal",
        "VmallocUsed", "VmallocChunk", "DirectMap4k", "DirectMap2M",
        "DirectMap1G",
    ];
    
    for key in &interesting_keys {
        if let Some(value) = meminfo.get(*key) {
            // 値はKBで格納されているのでそれを示す
            properties.insert(key.to_string(), format!("{} kB", value));
        }
    }
    
    // DMIデータベースから追加のメモリ情報を取得（利用可能な場合）
    if let Ok(properties_from_dmi) = read_memory_info_from_dmi() {
        properties.extend(properties_from_dmi);
    }
    
    properties
}

/// DMI (Desktop Management Interface) データベースからメモリ情報を読み取り
fn read_memory_info_from_dmi() -> Result<HashMap<String, String>, io::Error> {
    let mut properties = HashMap::new();
    
    // dmidecodeコマンドが利用可能かチェック
    if let Ok(output) = std::process::Command::new("dmidecode")
        .arg("-t")
        .arg("memory")
        .output() {
        
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                // 出力から有用な情報を抽出
                for line in output_str.lines() {
                    let line = line.trim();
                    
                    // キーと値のペアを検索
                    if line.contains(':') {
                        if let Some((key, value)) = line.split_once(':') {
                            let key = key.trim();
                            let value = value.trim();
                            
                            // 重要なメモリ特性を追加
                            if key.contains("Speed") || 
                               key.contains("Type") || 
                               key.contains("Form Factor") || 
                               key.contains("Manufacturer") || 
                               key.contains("Serial Number") || 
                               key.contains("Part Number") {
                                if !value.is_empty() && value != "Unknown" {
                                    properties.insert(format!("DMI_{}", key.replace(" ", "_")), value.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(properties)
}

/// メモリ帯域幅を推定（この関数は実際の値を取得する方法がないため、推定値を返す）
fn estimate_memory_bandwidth() -> Option<u64> {
    // 実際の帯域幅を測定するには専用のベンチマークが必要
    // メモリ情報から推測する単純な方法を提供
    
    // DMIデータからメモリ速度を読み取り（例: DDR4-3200）
    if let Ok(output) = std::process::Command::new("dmidecode")
        .arg("-t")
        .arg("memory")
        .output() {
        
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("Speed") {
                        // メモリ速度を抽出（例: "Speed: 3200 MT/s"）
                        if let Some(speed_str) = line.split(':').nth(1) {
                            let speed_str = speed_str.trim();
                            // MT/sまたはMHzの部分を抽出
                            if let Some(speed_value) = speed_str.split_whitespace().next() {
                                if let Ok(speed) = speed_value.parse::<u64>() {
                                    // 推定帯域幅の計算:
                                    // DDRの転送レート（例: 3200 MT/s）* チャネル数 * バス幅（例: 64ビット = 8バイト）
                                    // チャネル数は通常2または4（ここでは仮に2と仮定）
                                    let channels = 2;
                                    let bus_width = 8; // 64ビット = 8バイト
                                    return Some(speed * 1_000_000 * channels * bus_width); // バイト/秒に変換
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // CPUモデルを読み取り
    if let Ok(file) = File::open("/proc/cpuinfo") {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("model name") {
                    // CPUの世代からメモリ帯域幅を推測
                    // ここでは単純化のためにざっくりした値を返す
                    
                    // IntelのCPUの場合
                    if line.contains("Intel") {
                        if line.contains("i9") || line.contains("i7") {
                            // 新しいIntelプロセッサはDDR4-3200または4800MT/s程度をサポート
                            let est_bandwidth = 3200 * 1_000_000 * 2 * 8; // 約51.2 GB/s
                            return Some(est_bandwidth);
                        } else if line.contains("i5") {
                            let est_bandwidth = 2933 * 1_000_000 * 2 * 8; // 約46.9 GB/s
                            return Some(est_bandwidth);
                        } else {
                            let est_bandwidth = 2400 * 1_000_000 * 2 * 8; // 約38.4 GB/s
                            return Some(est_bandwidth);
                        }
                    }
                    // AMDのCPUの場合
                    else if line.contains("AMD") {
                        if line.contains("Ryzen") {
                            // 新しいRyzen CPUはDDR4-3200または3600MT/s程度をサポート
                            let est_bandwidth = 3600 * 1_000_000 * 2 * 8; // 約57.6 GB/s
                            return Some(est_bandwidth);
                        } else {
                            let est_bandwidth = 2400 * 1_000_000 * 2 * 8; // 約38.4 GB/s
                            return Some(est_bandwidth);
                        }
                    }
                    
                    // デフォルト値（低めの推定）
                    return Some(2133 * 1_000_000 * 2 * 8); // 約34.1 GB/s
                }
            }
        }
    }
    
    // 情報が取得できない場合はNoneを返す
    None
}

/// バイト数を適切な単位に変換して文字列表現を返す
pub fn format_bytes(bytes: u64) -> String {
    let unit = MemoryUnit::auto_select(bytes);
    let value = unit.convert_from_bytes(bytes);
    
    // 小数点以下の桁数を調整
    match unit {
        MemoryUnit::Byte => format!("{:.0} {}", value, unit.as_str()),
        MemoryUnit::KB => format!("{:.1} {}", value, unit.as_str()),
        _ => format!("{:.2} {}", value, unit.as_str()),
    }
}

/// メモリスワップ率を計算
/// 
/// スワップの使用率と発生頻度からシステムのメモリプレッシャーを評価する
pub fn calculate_swap_rate() -> Result<f64, io::Error> {
    // /proc/vmstat からスワップ統計を読み取る
    let file = File::open("/proc/vmstat")?;
    let reader = BufReader::new(file);
    
    let mut swap_in = 0;
    let mut swap_out = 0;
    
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("pswpin") {
            if let Some(value_str) = line.split_whitespace().nth(1) {
                swap_in = value_str.parse::<u64>().unwrap_or(0);
            }
        } else if line.starts_with("pswpout") {
            if let Some(value_str) = line.split_whitespace().nth(1) {
                swap_out = value_str.parse::<u64>().unwrap_or(0);
            }
        }
    }
    
    // スワップ率の計算 (スワップイン + スワップアウト)
    // これは絶対値ではなく、システム起動以降の累積値
    let swap_rate = swap_in + swap_out;
    
    // システム起動時間を取得して、1秒あたりのスワップ率を計算
    let uptime = read_uptime()?;
    
    if uptime > 0.0 {
        Ok(swap_rate as f64 / uptime)
    } else {
        Ok(0.0)
    }
}

/// システムの稼働時間を秒単位で取得
fn read_uptime() -> Result<f64, io::Error> {
    let content = std::fs::read_to_string("/proc/uptime")?;
    let parts: Vec<&str> = content.split_whitespace().collect();
    
    if let Some(uptime_str) = parts.first() {
        if let Ok(uptime) = uptime_str.parse::<f64>() {
            return Ok(uptime);
        }
    }
    
    Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to parse uptime"))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_unit_conversion() {
        // バイトから他の単位への変換をテスト
        let bytes = 1024 * 1024 * 5; // 5 MB
        assert_eq!(MemoryUnit::Byte.convert_from_bytes(bytes), bytes as f64);
        assert_eq!(MemoryUnit::KB.convert_from_bytes(bytes), 5.0 * 1024.0);
        assert_eq!(MemoryUnit::MB.convert_from_bytes(bytes), 5.0);
        assert_eq!(MemoryUnit::GB.convert_from_bytes(bytes), 5.0 / 1024.0);
        
        // 他の単位からバイトへの変換をテスト
        assert_eq!(MemoryUnit::MB.convert_to_bytes(5.0), bytes);
        assert_eq!(MemoryUnit::GB.convert_to_bytes(2.0), 2 * 1024 * 1024 * 1024);
    }
    
    #[test]
    fn test_auto_select_unit() {
        assert_eq!(MemoryUnit::auto_select(500), MemoryUnit::Byte);
        assert_eq!(MemoryUnit::auto_select(1500), MemoryUnit::KB);
        assert_eq!(MemoryUnit::auto_select(1024 * 1024 * 2), MemoryUnit::MB);
        assert_eq!(MemoryUnit::auto_select(1024 * 1024 * 1024 * 3), MemoryUnit::GB);
        assert_eq!(MemoryUnit::auto_select(1024 * 1024 * 1024 * 1024 * 5), MemoryUnit::TB);
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024 * 2), "2.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 3), "3.00 GB");
    }
    
    #[test]
    fn test_collect_memory_info() {
        // 実際のメモリ情報を収集してみる
        // この部分はシステムに依存するため、最小限のチェックのみ実施
        match collect_memory_info() {
            Ok(info) => {
                // 基本的な整合性チェック
                assert!(info.usage.total_bytes > 0);
                assert!(info.usage.used_bytes <= info.usage.total_bytes);
                assert!(info.usage.free_bytes <= info.usage.total_bytes);
                assert!(info.usage.used_percent >= 0.0 && info.usage.used_percent <= 100.0);
                
                // スワップが存在しない場合もありえるので、ゼロの場合もOK
                if info.swap.total_bytes > 0 {
                    assert!(info.swap.used_bytes <= info.swap.total_bytes);
                    assert!(info.swap.free_bytes <= info.swap.total_bytes);
                    assert!(info.swap.used_percent >= 0.0 && info.swap.used_percent <= 100.0);
                }
            },
            Err(e) => {
                // テスト環境によってはエラーになる可能性もあるため、
                // エラーメッセージを出力してスキップ
                eprintln!("メモリ情報収集エラー（環境依存のためスキップ）: {}", e);
            }
        }
    }
} 