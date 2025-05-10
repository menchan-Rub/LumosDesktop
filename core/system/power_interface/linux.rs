//! Linux用の電源インターフェース実装
//!
//! このモジュールは、Linuxシステム向けの電源関連機能を実装します。
//! UPowerを使用してバッテリー情報を取得し、systemdを使用して電源管理機能を提供します。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use log::{debug, error, info, warn};

use super::{BatteryHealth, BatteryInfo, ChargingState, PowerPlan, PowerSource, PowerSystemInfo};

/// UPowerが提供するデバイスパスの接頭辞
const UPOWER_DEVICE_PREFIX: &str = "/org/freedesktop/UPower/devices/";

/// Linux固有の電源情報を取得して更新
pub fn update_power_info(info: &mut PowerSystemInfo) {
    info.last_updated = Instant::now();
    
    // バッテリー情報を更新
    let batteries = get_battery_devices();
    if batteries.is_empty() {
        debug!("バッテリーが見つかりません");
        info.power_source = PowerSource::AC;
    } else {
        info.batteries.clear();
        
        let mut on_battery = false;
        for device_path in batteries {
            if let Some(battery_info) = get_battery_info(&device_path) {
                if battery_info.charging_state == ChargingState::Discharging {
                    on_battery = true;
                }
                info.batteries.push(battery_info);
            }
        }
        
        info.power_source = if on_battery {
            PowerSource::Battery
        } else {
            PowerSource::AC
        };
    }
    
    // 現在の電源プランを取得
    info.current_power_plan = get_current_power_plan();
    
    // スリープ/休止状態のサポートを確認
    info.supports_sleep = check_sleep_support();
    info.supports_hibernate = check_hibernate_support();
}

/// システムにインストールされているバッテリーデバイスのパスを取得
fn get_battery_devices() -> Vec<String> {
    let mut batteries = Vec::new();
    
    // UPowerコマンドでデバイス一覧を取得
    let output = match Command::new("upower")
        .args(&["--enumerate"])
        .output() {
            Ok(output) => output,
            Err(e) => {
                error!("UPowerコマンドの実行に失敗しました: {}", e);
                return Vec::new();
            }
        };
    
    let devices = String::from_utf8_lossy(&output.stdout);
    for line in devices.lines() {
        if line.contains("/battery_") {
            batteries.push(line.trim().to_string());
        }
    }
    
    debug!("検出されたバッテリー: {:?}", batteries);
    batteries
}

/// 指定されたデバイスのバッテリー情報を取得
fn get_battery_info(device_path: &str) -> Option<BatteryInfo> {
    // UPowerコマンドでバッテリー情報を取得
    let output = match Command::new("upower")
        .args(&["--show-info", device_path])
        .output() {
            Ok(output) => output,
            Err(e) => {
                error!("バッテリー情報の取得に失敗しました: {}", e);
                return None;
            }
        };
    
    let info_text = String::from_utf8_lossy(&output.stdout);
    parse_battery_info(&info_text, device_path)
}

/// UPowerの出力からバッテリー情報を解析
fn parse_battery_info(info_text: &str, device_path: &str) -> Option<BatteryInfo> {
    let mut battery = BatteryInfo::default();
    
    // デバイスパスからIDを抽出
    if let Some(id) = device_path.strip_prefix(UPOWER_DEVICE_PREFIX) {
        battery.id = id.replace("battery_", "").to_string();
    } else {
        battery.id = device_path.to_string();
    }
    
    let mut percentage: f32 = 0.0;
    let mut energy_full_design: f64 = 0.0;
    let mut energy_full: f64 = 0.0;
    
    for line in info_text.lines() {
        let line = line.trim();
        
        if line.starts_with("native-path:") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() > 1 {
                battery.name = parts[1].trim().to_string();
            }
        } else if line.starts_with("vendor:") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() > 1 {
                battery.manufacturer = parts[1].trim().to_string();
            }
        } else if line.starts_with("serial:") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() > 1 {
                battery.serial_number = parts[1].trim().to_string();
            }
        } else if line.starts_with("energy-full-design:") {
            if let Some(value) = extract_value_with_unit(line, "Wh") {
                energy_full_design = value * 1000.0; // WhからmWhに変換
                battery.design_capacity = value as u32 * 1000;
            }
        } else if line.starts_with("energy-full:") {
            if let Some(value) = extract_value_with_unit(line, "Wh") {
                energy_full = value * 1000.0; // WhからmWhに変換
                battery.current_capacity = value as u32 * 1000;
            }
        } else if line.starts_with("percentage:") {
            if let Some(value) = extract_value_with_unit(line, "%") {
                percentage = value as f32;
                battery.charge_level = value as u8;
            }
        } else if line.starts_with("state:") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() > 1 {
                battery.charging_state = match parts[1].trim() {
                    "charging" => ChargingState::Charging,
                    "discharging" => ChargingState::Discharging,
                    "fully-charged" => ChargingState::Full,
                    "pending-charge" => ChargingState::Charging,
                    "pending-discharge" => ChargingState::Discharging,
                    "empty" => ChargingState::Discharging,
                    _ => ChargingState::Unknown,
                };
            }
        } else if line.starts_with("voltage:") {
            if let Some(value) = extract_value_with_unit(line, "V") {
                battery.voltage = (value * 1000.0) as u32; // VからmVに変換
            }
        } else if line.starts_with("energy-rate:") {
            if let Some(value) = extract_value_with_unit(line, "W") {
                // 放電時は正、充電時は負
                let power = (value * 1000.0) as i32; // WからmWに変換
                battery.power_consumption = if battery.charging_state == ChargingState::Discharging {
                    power
                } else {
                    -power
                };
            }
        } else if line.starts_with("time to empty:") {
            if battery.charging_state == ChargingState::Discharging {
                if let Some(value) = extract_value_with_unit(line, "hours") {
                    battery.time_remaining = Some((value * 60.0) as u32);
                } else if let Some(value) = extract_value_with_unit(line, "minutes") {
                    battery.time_remaining = Some(value as u32);
                }
            }
        } else if line.starts_with("time to full:") {
            if battery.charging_state == ChargingState::Charging {
                if let Some(value) = extract_value_with_unit(line, "hours") {
                    battery.time_to_full = Some((value * 60.0) as u32);
                } else if let Some(value) = extract_value_with_unit(line, "minutes") {
                    battery.time_to_full = Some(value as u32);
                }
            }
        }
    }
    
    // バッテリーの健康状態を計算
    if energy_full_design > 0.0 {
        let health_ratio = energy_full / energy_full_design;
        battery.health = if health_ratio >= 0.8 {
            BatteryHealth::Good
        } else if health_ratio >= 0.5 {
            BatteryHealth::Degrading
        } else {
            BatteryHealth::Poor
        };
    } else {
        battery.health = BatteryHealth::Unknown;
    }
    
    Some(battery)
}

/// 行から数値と単位を抽出
fn extract_value_with_unit(line: &str, unit: &str) -> Option<f64> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() < 2 {
        return None;
    }
    
    let value_part = parts[1].trim();
    if !value_part.ends_with(unit) {
        return None;
    }
    
    let value_str = value_part.trim_end_matches(unit).trim();
    value_str.parse::<f64>().ok()
}

/// 現在の電源プランを取得
fn get_current_power_plan() -> PowerPlan {
    // TLPの設定を確認
    if Path::new("/usr/bin/tlp-stat").exists() {
        let output = Command::new("tlp-stat")
            .args(&["-s"])
            .output();
        
        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("Mode           = battery") {
                return PowerPlan::PowerSaver;
            } else if output_str.contains("Mode           = performance") {
                return PowerPlan::HighPerformance;
            } else if output_str.contains("Mode           = balanced") {
                return PowerPlan::Balanced;
            }
        }
    }
    
    // power-profilesの設定を確認
    if Path::new("/usr/bin/powerprofilesctl").exists() {
        let output = Command::new("powerprofilesctl")
            .args(&["get"])
            .output();
        
        if let Ok(output) = output {
            let profile = String::from_utf8_lossy(&output.stdout).trim().to_string();
            match profile.as_str() {
                "power-saver" => return PowerPlan::PowerSaver,
                "balanced" => return PowerPlan::Balanced,
                "performance" => return PowerPlan::HighPerformance,
                _ => {}
            }
        }
    }
    
    // cpufreq-infoの設定を確認
    if Path::new("/usr/bin/cpufreq-info").exists() {
        let output = Command::new("cpufreq-info")
            .output();
        
        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("governor \"performance\"") {
                return PowerPlan::HighPerformance;
            } else if output_str.contains("governor \"powersave\"") {
                return PowerPlan::PowerSaver;
            } else if output_str.contains("governor \"ondemand\"") || 
                     output_str.contains("governor \"conservative\"") {
                return PowerPlan::Balanced;
            }
        }
    }
    
    // デフォルト値
    PowerPlan::Balanced
}

/// 電源プランを設定
pub fn set_power_plan(plan: PowerPlan) -> Result<(), String> {
    info!("Linuxの電源プランを設定: {:?}", plan);
    
    // power-profilesが利用可能かチェック
    if Path::new("/usr/bin/powerprofilesctl").exists() {
        let profile = match plan {
            PowerPlan::HighPerformance => "performance",
            PowerPlan::Balanced => "balanced",
            PowerPlan::PowerSaver => "power-saver",
            PowerPlan::Custom(_) => return Err("カスタム電源プランはpower-profilesではサポートされていません".to_string()),
        };
        
        let output = Command::new("powerprofilesctl")
            .args(&["set", profile])
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                warn!("power-profilesでの電源プラン設定に失敗しました: {}", e);
            }
        }
    }
    
    // TLPが利用可能かチェック
    if Path::new("/usr/bin/tlp").exists() {
        let tlp_mode = match plan {
            PowerPlan::HighPerformance => "performance",
            PowerPlan::Balanced => "balanced",
            PowerPlan::PowerSaver => "battery",
            PowerPlan::Custom(_) => return Err("カスタム電源プランはTLPではサポートされていません".to_string()),
        };
        
        let output = Command::new("tlp")
            .args(&[tlp_mode])
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                warn!("TLPでの電源プラン設定に失敗しました: {}", e);
            }
        }
    }
    
    // cpufreq-setが利用可能かチェック
    if Path::new("/usr/bin/cpufreq-set").exists() {
        let governor = match plan {
            PowerPlan::HighPerformance => "performance",
            PowerPlan::Balanced => "ondemand",
            PowerPlan::PowerSaver => "powersave",
            PowerPlan::Custom(_) => return Err("カスタム電源プランはcpufreqではサポートされていません".to_string()),
        };
        
        // すべてのCPUコアに適用
        let num_cores = num_cpus::get();
        for core in 0..num_cores {
            let output = Command::new("cpufreq-set")
                .args(&["-c", &core.to_string(), "-g", governor])
                .output();
                
            if let Err(e) = output {
                warn!("CPU{}のガバナー設定に失敗しました: {}", core, e);
            }
        }
        
        return Ok(());
    }
    
    Err("サポートされている電源管理ツールが見つかりません".to_string())
}

/// システムをスリープ状態にする
pub fn sleep_system() -> Result<(), String> {
    // systemdが利用可能かチェック
    if Path::new("/usr/bin/systemctl").exists() {
        let output = Command::new("systemctl")
            .args(&["suspend"])
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("systemctlによるスリープに失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    // pmutilsが利用可能かチェック
    if Path::new("/usr/sbin/pm-suspend").exists() {
        let output = Command::new("pm-suspend")
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("pm-suspendによるスリープに失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    // sysfsインターフェースを試す
    if Path::new("/sys/power/state").exists() {
        match fs::write("/sys/power/state", "mem") {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("sysfsインターフェースによるスリープに失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    Err("サポートされているスリープメカニズムが見つかりません".to_string())
}

/// システムを休止状態にする
pub fn hibernate_system() -> Result<(), String> {
    // systemdが利用可能かチェック
    if Path::new("/usr/bin/systemctl").exists() {
        let output = Command::new("systemctl")
            .args(&["hibernate"])
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("systemctlによる休止状態に失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    // pmutilsが利用可能かチェック
    if Path::new("/usr/sbin/pm-hibernate").exists() {
        let output = Command::new("pm-hibernate")
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("pm-hibernateによる休止状態に失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    // sysfsインターフェースを試す
    if Path::new("/sys/power/state").exists() {
        match fs::write("/sys/power/state", "disk") {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("sysfsインターフェースによる休止状態に失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    Err("サポートされている休止状態メカニズムが見つかりません".to_string())
}

/// システムをシャットダウンする
pub fn shutdown_system() -> Result<(), String> {
    // systemdが利用可能かチェック
    if Path::new("/usr/bin/systemctl").exists() {
        let output = Command::new("systemctl")
            .args(&["poweroff"])
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("systemctlによるシャットダウンに失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    // shutdownコマンドを試す
    if Path::new("/sbin/shutdown").exists() {
        let output = Command::new("shutdown")
            .args(&["-h", "now"])
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("shutdownコマンドによるシャットダウンに失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    // poweroffコマンドを試す
    if Path::new("/sbin/poweroff").exists() {
        let output = Command::new("poweroff")
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("poweroffコマンドによるシャットダウンに失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    Err("サポートされているシャットダウンメカニズムが見つかりません".to_string())
}

/// システムを再起動する
pub fn reboot_system() -> Result<(), String> {
    // systemdが利用可能かチェック
    if Path::new("/usr/bin/systemctl").exists() {
        let output = Command::new("systemctl")
            .args(&["reboot"])
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("systemctlによる再起動に失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    // shutdownコマンドを試す
    if Path::new("/sbin/shutdown").exists() {
        let output = Command::new("shutdown")
            .args(&["-r", "now"])
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("shutdownコマンドによる再起動に失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    // rebootコマンドを試す
    if Path::new("/sbin/reboot").exists() {
        let output = Command::new("reboot")
            .output();
            
        match output {
            Ok(_) => return Ok(()),
            Err(e) => {
                let err_msg = format!("rebootコマンドによる再起動に失敗しました: {}", e);
                error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    Err("サポートされている再起動メカニズムが見つかりません".to_string())
}

/// スリープ機能がサポートされているか確認
fn check_sleep_support() -> bool {
    // systemdのサスペンド機能をチェック
    if Path::new("/usr/bin/systemctl").exists() {
        if let Ok(output) = Command::new("systemctl")
            .args(&["can-suspend"])
            .output() {
            if output.status.success() {
                return true;
            }
        }
    }
    
    // pmutilsをチェック
    if Path::new("/usr/sbin/pm-suspend").exists() {
        return true;
    }
    
    // sysfsインターフェースをチェック
    if let Ok(states) = fs::read_to_string("/sys/power/state") {
        return states.contains("mem");
    }
    
    false
}

/// 休止状態機能がサポートされているか確認
fn check_hibernate_support() -> bool {
    // systemdの休止機能をチェック
    if Path::new("/usr/bin/systemctl").exists() {
        if let Ok(output) = Command::new("systemctl")
            .args(&["can-hibernate"])
            .output() {
            if output.status.success() {
                return true;
            }
        }
    }
    
    // pmutilsをチェック
    if Path::new("/usr/sbin/pm-hibernate").exists() {
        return true;
    }
    
    // sysfsインターフェースをチェック
    if let Ok(states) = fs::read_to_string("/sys/power/state") {
        return states.contains("disk");
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_value_with_unit() {
        assert_eq!(extract_value_with_unit("percentage:      75%", "%"), Some(75.0));
        assert_eq!(extract_value_with_unit("energy-rate:     15.25 W", "W"), Some(15.25));
        assert_eq!(extract_value_with_unit("voltage:         12.1 V", "V"), Some(12.1));
        assert_eq!(extract_value_with_unit("time to empty:   2.5 hours", "hours"), Some(2.5));
        assert_eq!(extract_value_with_unit("time to full:    45 minutes", "minutes"), Some(45.0));
        
        assert_eq!(extract_value_with_unit("invalid line", "W"), None);
        assert_eq!(extract_value_with_unit("energy-rate:     N/A", "W"), None);
    }
    
    #[test]
    fn test_parse_battery_info() {
        let sample_output = r#"
  native-path:          BAT0
  vendor:               ACME Corp
  model:                Laptop Battery
  serial:               1234567890
  power supply:         yes
  updated:              Mon 14 Sep 2020 02:37:24 PM JST (35 seconds ago)
  has history:          yes
  has statistics:       yes
  battery
    present:             yes
    rechargeable:        yes
    state:               discharging
    warning-level:       none
    energy:              35.2 Wh
    energy-empty:        0 Wh
    energy-full:         45.7 Wh
    energy-full-design:  56.3 Wh
    energy-rate:         8.24 W
    voltage:             11.8 V
    time to empty:       4.3 hours
    percentage:          77%
    capacity:            81.1731%
    technology:          lithium-ion
    icon-name:          'battery-good-symbolic'
        "#;
        
        let battery = parse_battery_info(sample_output, "/org/freedesktop/UPower/devices/battery_BAT0").unwrap();
        
        assert_eq!(battery.id, "BAT0");
        assert_eq!(battery.name, "BAT0");
        assert_eq!(battery.manufacturer, "ACME Corp");
        assert_eq!(battery.serial_number, "1234567890");
        assert_eq!(battery.design_capacity, 56300);
        assert_eq!(battery.current_capacity, 45700);
        assert_eq!(battery.charge_level, 77);
        assert_eq!(battery.charging_state, ChargingState::Discharging);
        assert_eq!(battery.voltage, 11800);
        assert_eq!(battery.power_consumption, 8240);
        assert_eq!(battery.time_remaining, Some(258)); // 4.3時間 = 258分
        assert_eq!(battery.health, BatteryHealth::Degrading);
    }
} 