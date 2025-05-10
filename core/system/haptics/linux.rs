// Linux向けのハプティクス実装
// Linuxプラットフォーム固有のハードウェアアクセス

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::fs;
use std::collections::HashMap;

use log::{debug, error, info, warn};

use super::{HapticDevice, HapticPattern, HapticIntensity, HapticDeviceType, HapticEvent};
use crate::core::utils::error::{Result, SystemError};

// Linuxでよく使用されるハプティクスパスの定数
const VIBRATOR_PATH: &str = "/sys/class/leds/vibrator";
const VIBRATOR_ALTERNATIVE_PATH: &str = "/sys/class/timed_output/vibrator";
const VIBRATOR_PATHS: [&str; 2] = [VIBRATOR_PATH, VIBRATOR_ALTERNATIVE_PATH];

// 振動子のコントロールファイル
const VIBRATOR_ACTIVATE_FILE: &str = "activate";
const VIBRATOR_DURATION_FILE: &str = "duration";
const VIBRATOR_STATE_FILE: &str = "state";
const VIBRATOR_BRIGHTNESS_FILE: &str = "brightness";
const VIBRATOR_INTENSITY_FILE: &str = "intensity";

// フォースフィードバックのパス
const FORCE_FEEDBACK_PATH: &str = "/dev/input";

/// Linuxデバイスの種類
pub enum LinuxDeviceType {
    SysfsVibrator,       // sysfsを使用する振動子
    ForceFeedback,       // フォースフィードバックデバイス
    Custom,              // カスタムデバイス
}

/// Linuxハプティクスデバイス情報
struct LinuxHapticDevice {
    // 基本デバイス情報
    base_device: HapticDevice,
    // デバイスタイプ
    device_type: LinuxDeviceType,
    // 制御ファイルのパス (主制御ファイル)
    control_path: PathBuf,
    // 追加の制御ファイル
    control_files: HashMap<String, PathBuf>,
}

/// Linuxハプティクスデバイスを検出する
pub fn detect_linux_haptic_devices() -> Vec<HapticDevice> {
    let mut devices = Vec::new();
    
    // 内蔵振動子を検出
    if let Some(device) = detect_internal_vibrator() {
        devices.push(device);
    }
    
    // フォースフィードバックデバイスを検出
    let ff_devices = detect_force_feedback_devices();
    devices.extend(ff_devices);
    
    // 入力デバイス（タッチパッド等）を検出
    let input_devices = detect_input_haptic_devices();
    devices.extend(input_devices);
    
    info!("検出されたハプティクスデバイス: {} 個", devices.len());
    
    devices
}

/// 内蔵振動子を検出する
fn detect_internal_vibrator() -> Option<HapticDevice> {
    // 振動子のパスをチェック
    for path in VIBRATOR_PATHS.iter() {
        let path = Path::new(path);
        if path.exists() {
            debug!("振動子パスを検出: {:?}", path);
            
            // デバイス名を取得
            let name = read_device_name(path).unwrap_or_else(|| "内蔵振動子".to_string());
            
            // サポートされている強度を判断
            let mut supported_intensities = vec![
                HapticIntensity::None,
                HapticIntensity::Light,
                HapticIntensity::Medium,
                HapticIntensity::Strong,
            ];
            
            // サポートされているパターンを判断
            let supported_patterns = vec![
                HapticPattern::Click,
                HapticPattern::DoubleClick,
                HapticPattern::LongPress,
                HapticPattern::Error,
                HapticPattern::Success,
                HapticPattern::Warning,
                HapticPattern::Notification,
            ];
            
            // コントロールファイルの存在をチェック
            let mut properties = HashMap::new();
            let control_files = [
                VIBRATOR_ACTIVATE_FILE,
                VIBRATOR_DURATION_FILE,
                VIBRATOR_STATE_FILE,
                VIBRATOR_BRIGHTNESS_FILE,
                VIBRATOR_INTENSITY_FILE,
            ];
            
            for file in control_files.iter() {
                let file_path = path.join(file);
                if file_path.exists() {
                    properties.insert(file.to_string(), "supported".to_string());
                }
            }
            
            // 強度ファイルが存在する場合は、VeryLight/VeryStrongもサポート
            if path.join(VIBRATOR_INTENSITY_FILE).exists() {
                supported_intensities.push(HapticIntensity::VeryLight);
                supported_intensities.push(HapticIntensity::VeryStrong);
                properties.insert("intensity_control".to_string(), "true".to_string());
            }
            
            return Some(HapticDevice {
                id: format!("vibrator:{}", path.to_string_lossy()),
                name,
                device_type: HapticDeviceType::InternalMotor,
                supported_intensities,
                supported_patterns,
                enabled: true,
                properties,
            });
        }
    }
    
    debug!("内蔵振動子は検出されませんでした");
    None
}

/// フォースフィードバックデバイスを検出する
fn detect_force_feedback_devices() -> Vec<HapticDevice> {
    let mut devices = Vec::new();
    
    // /dev/input 内のデバイスをスキャン
    if let Ok(entries) = fs::read_dir(FORCE_FEEDBACK_PATH) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            // event* デバイスのみをチェック
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with("event") {
                    // フォースフィードバックのサポートをチェック
                    if check_ff_support(&path) {
                        let device_name = read_device_name(&path).unwrap_or_else(|| {
                            format!("フォースフィードバックデバイス ({})", file_name_str)
                        });
                        
                        // デバイスの種類に基づいてデバイスタイプを判断
                        let device_type = if device_name.to_lowercase().contains("gamepad") || 
                                            device_name.to_lowercase().contains("controller") {
                            HapticDeviceType::ExternalController
                        } else if device_name.to_lowercase().contains("touchpad") {
                            HapticDeviceType::Touchpad
                        } else {
                            HapticDeviceType::ForceFeedback
                        };
                        
                        let mut properties = HashMap::new();
                        properties.insert("path".to_string(), path.to_string_lossy().to_string());
                        
                        devices.push(HapticDevice {
                            id: format!("ff:{}", file_name_str),
                            name: device_name,
                            device_type,
                            supported_intensities: vec![
                                HapticIntensity::None,
                                HapticIntensity::Light,
                                HapticIntensity::Medium,
                                HapticIntensity::Strong,
                            ],
                            supported_patterns: vec![
                                HapticPattern::Click,
                                HapticPattern::DoubleClick,
                                HapticPattern::LongPress,
                                HapticPattern::Error,
                                HapticPattern::Success,
                            ],
                            enabled: true,
                            properties,
                        });
                    }
                }
            }
        }
    }
    
    debug!("検出されたフォースフィードバックデバイス: {} 個", devices.len());
    devices
}

/// デバイスがフォースフィードバックをサポートしているかチェック
fn check_ff_support(path: &Path) -> bool {
    // Linuxでは通常、/sys/class/input/eventX/device/capabilities/ffに
    // フォースフィードバックサポートが示されている
    
    let event_num = path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("event"))
        .and_then(|num| num.parse::<u32>().ok());
    
    if let Some(num) = event_num {
        let ff_path = format!("/sys/class/input/event{}/device/capabilities/ff", num);
        
        if let Ok(mut file) = File::open(ff_path) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                // 0以外の値はフォースフィードバックをサポートしていることを示す
                return content.trim() != "0";
            }
        }
    }
    
    false
}

/// ハプティクス対応のタッチパッドやその他の入力デバイスを検出
fn detect_input_haptic_devices() -> Vec<HapticDevice> {
    let mut devices = Vec::new();
    
    // タッチパッドを検出
    if let Some(device) = detect_haptic_touchpad() {
        devices.push(device);
    }
    
    devices
}

/// ハプティクス対応のタッチパッドを検出
fn detect_haptic_touchpad() -> Option<HapticDevice> {
    // デバイス情報をチェックするディレクトリ
    let input_dirs = [
        "/sys/class/input/input*",
        "/sys/bus/hid/devices/*",
    ];
    
    for glob_pattern in input_dirs.iter() {
        if let Ok(paths) = glob::glob(glob_pattern) {
            for path in paths.flatten() {
                // デバイス名をチェック
                let name_path = path.join("name");
                if let Ok(mut file) = File::open(&name_path) {
                    let mut name = String::new();
                    if file.read_to_string(&mut name).is_ok() {
                        name = name.trim().to_string();
                        
                        // タッチパッドを特定するキーワード
                        if name.to_lowercase().contains("touchpad") {
                            // ハプティクスのサポートをチェック
                            // これは実装固有であり、完全にデバイスに依存することに注意
                            // 例として、特定のベンダーのファイルをチェック
                            
                            let mut properties = HashMap::new();
                            properties.insert("path".to_string(), path.to_string_lossy().to_string());
                            properties.insert("name".to_string(), name.clone());
                            
                            // 関連するイベントデバイスを見つける
                            if let Some(event_path) = find_event_path_for_input(&path) {
                                properties.insert("event_path".to_string(), event_path.to_string_lossy().to_string());
                                
                                debug!("触覚対応タッチパッドを検出: {}", name);
                                
                                return Some(HapticDevice {
                                    id: format!("touchpad:{}", path.file_name().unwrap_or_default().to_string_lossy()),
                                    name: format!("触覚対応タッチパッド ({})", name),
                                    device_type: HapticDeviceType::Touchpad,
                                    supported_intensities: vec![
                                        HapticIntensity::None,
                                        HapticIntensity::VeryLight,
                                        HapticIntensity::Light,
                                        HapticIntensity::Medium,
                                    ],
                                    supported_patterns: vec![
                                        HapticPattern::Click,
                                        HapticPattern::DoubleClick,
                                    ],
                                    enabled: true,
                                    properties,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// 指定された入力デバイスに関連するイベントパスを検索
fn find_event_path_for_input(input_path: &Path) -> Option<PathBuf> {
    // input/inputX/eventY をチェック
    if let Ok(entries) = fs::read_dir(input_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with("event") {
                    return Some(PathBuf::from(format!("/dev/input/{}", name_str)));
                }
            }
        }
    }
    
    None
}

/// デバイス名を読み取る
fn read_device_name(path: &Path) -> Option<String> {
    // イベントデバイスの場合
    if path.to_string_lossy().contains("/dev/input/event") {
        let event_num = path.file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("event"))
            .and_then(|num| num.parse::<u32>().ok());
        
        if let Some(num) = event_num {
            let name_path = format!("/sys/class/input/event{}/device/name", num);
            
            if let Ok(mut file) = File::open(&name_path) {
                let mut name = String::new();
                if file.read_to_string(&mut name).is_ok() {
                    return Some(name.trim().to_string());
                }
            }
        }
    }
    
    // その他のデバイス（振動子など）
    let name_path = path.join("device").join("name");
    if name_path.exists() {
        if let Ok(mut file) = File::open(&name_path) {
            let mut name = String::new();
            if file.read_to_string(&mut name).is_ok() {
                return Some(name.trim().to_string());
            }
        }
    }
    
    None
}

/// Linuxハプティクスフィードバックを再生する
pub fn play_linux_haptic_feedback(device_id: &str, event: &HapticEvent) -> Result<()> {
    debug!("Linux触覚フィードバック再生: デバイス={}, パターン={:?}, 強度={:?}, 持続時間={:?}ms",
           device_id, event.pattern, event.intensity, event.duration.as_millis());
    
    if device_id.starts_with("vibrator:") {
        // 振動子デバイスの場合
        let path = device_id.strip_prefix("vibrator:").unwrap_or("");
        play_vibrator_feedback(Path::new(path), event)
    } else if device_id.starts_with("ff:") {
        // フォースフィードバックデバイスの場合
        let device_name = device_id.strip_prefix("ff:").unwrap_or("");
        let path = PathBuf::from(format!("/dev/input/{}", device_name));
        play_ff_feedback(&path, event)
    } else if device_id.starts_with("touchpad:") {
        // タッチパッドデバイスの場合
        play_touchpad_feedback(device_id, event)
    } else {
        // 不明なデバイスの場合
        warn!("不明なデバイスID: {}", device_id);
        Err(SystemError::InvalidParameter(format!("不明なデバイスID: {}", device_id)))
    }
}

/// 振動子フィードバックを再生
fn play_vibrator_feedback(path: &Path, event: &HapticEvent) -> Result<()> {
    // 振動時間を決定（パターンに基づく）
    let mut duration_ms = event.duration.as_millis() as u64;
    if duration_ms == 0 {
        // パターンに基づくデフォルト時間
        duration_ms = match event.pattern {
            HapticPattern::Click => 20,
            HapticPattern::DoubleClick => 50,
            HapticPattern::LongPress => 100,
            HapticPattern::Error => 150,
            HapticPattern::Success => 80,
            HapticPattern::Warning => 120,
            HapticPattern::Notification => 80,
            HapticPattern::Custom(_) => 50,
        };
    }
    
    // 強度に基づいて時間を調整（強度が強いほど長く）
    let intensity_factor = match event.intensity {
        HapticIntensity::None => 0.0,
        HapticIntensity::VeryLight => 0.6,
        HapticIntensity::Light => 0.8,
        HapticIntensity::Medium => 1.0,
        HapticIntensity::Strong => 1.2,
        HapticIntensity::VeryStrong => 1.5,
    };
    
    // 強度がNoneの場合は何もしない
    if intensity_factor <= 0.0 {
        return Ok(());
    }
    
    duration_ms = (duration_ms as f64 * intensity_factor) as u64;
    
    // パターンに基づいて特別な処理
    if event.pattern == HapticPattern::DoubleClick {
        // ダブルクリックの場合、2回の短い振動を実行
        let single_duration = duration_ms / 3;
        let pause_duration = single_duration;
        
        // 最初の振動
        write_vibrator_duration(path, single_duration)?;
        write_vibrator_activate(path)?;
        
        // 一時停止
        std::thread::sleep(Duration::from_millis(single_duration + pause_duration));
        
        // 2回目の振動
        write_vibrator_duration(path, single_duration)?;
        write_vibrator_activate(path)?;
        
        return Ok(());
    }
    
    // 通常の振動
    write_vibrator_duration(path, duration_ms)?;
    
    // 強度ファイルがある場合は強度も設定
    let intensity_path = path.join(VIBRATOR_INTENSITY_FILE);
    if intensity_path.exists() {
        let intensity_value = match event.intensity {
            HapticIntensity::VeryLight => 64,  // 約25%
            HapticIntensity::Light => 102,     // 約40%
            HapticIntensity::Medium => 153,    // 約60%
            HapticIntensity::Strong => 204,    // 約80%
            HapticIntensity::VeryStrong => 255, // 100%
            _ => 153, // デフォルト中程度
        };
        
        if let Err(e) = write_to_file(&intensity_path, &intensity_value.to_string()) {
            warn!("振動子の強度設定に失敗: {:?}", e);
        }
    }
    
    // 有効化ファイルがあれば使用、なければbrightnessまたはstateを使用
    if path.join(VIBRATOR_ACTIVATE_FILE).exists() {
        write_vibrator_activate(path)?;
    } else if path.join(VIBRATOR_BRIGHTNESS_FILE).exists() {
        write_to_file(&path.join(VIBRATOR_BRIGHTNESS_FILE), "1")?;
    } else if path.join(VIBRATOR_STATE_FILE).exists() {
        write_to_file(&path.join(VIBRATOR_STATE_FILE), "1")?;
    } else {
        return Err(SystemError::OperationFailed("振動子の有効化ファイルが見つかりません".to_string()));
    }
    
    Ok(())
}

/// 振動子の持続時間を書き込む
fn write_vibrator_duration(path: &Path, duration_ms: u64) -> Result<()> {
    let duration_path = path.join(VIBRATOR_DURATION_FILE);
    if duration_path.exists() {
        write_to_file(&duration_path, &duration_ms.to_string())?;
        Ok(())
    } else {
        Err(SystemError::OperationFailed("振動子の持続時間ファイルが見つかりません".to_string()))
    }
}

/// 振動子を有効化する
fn write_vibrator_activate(path: &Path) -> Result<()> {
    let activate_path = path.join(VIBRATOR_ACTIVATE_FILE);
    if activate_path.exists() {
        write_to_file(&activate_path, "1")?;
        Ok(())
    } else {
        Err(SystemError::OperationFailed("振動子の有効化ファイルが見つかりません".to_string()))
    }
}

/// フォースフィードバックを再生
fn play_ff_feedback(path: &Path, event: &HapticEvent) -> Result<()> {
    // Linuxでのフォースフィードバックの実装
    // 実際の実装にはlinux/input.hとioctl呼び出しを使用する必要があり、
    // ここでは基本的な実装のみ示しています
    debug!("フォースフィードバック: {:?} に {:?} パターンを再生", path, event.pattern);
    
    // 注意: 実際の実装ではrusixのような低レベルライブラリが必要
    // 以下は模擬的な実装です
    // 本格的な実装には、evdev-rsやlinuxkitパッケージを使用した方が良い
    
    Ok(())
}

/// タッチパッドフィードバックを再生
fn play_touchpad_feedback(device_id: &str, event: &HapticEvent) -> Result<()> {
    // 特定のタッチパッドドライバー固有の実装が必要
    debug!("タッチパッド触覚フィードバック: {} に {:?} パターンを再生", device_id, event.pattern);
    
    // 注意: 実際の実装はタッチパッドの種類に依存
    // 以下は模擬的な実装です
    
    Ok(())
}

/// ファイルにデータを書き込む汎用関数
fn write_to_file(path: &Path, data: &str) -> Result<()> {
    match File::create(path) {
        Ok(mut file) => {
            match file.write_all(data.as_bytes()) {
                Ok(_) => Ok(()),
                Err(e) => {
                    warn!("ファイル書き込みエラー {:?}: {}", path, e);
                    Err(SystemError::IOError(e.to_string()))
                }
            }
        },
        Err(e) => {
            warn!("ファイル作成エラー {:?}: {}", path, e);
            Err(SystemError::IOError(e.to_string()))
        }
    }
}

/// Linuxハプティクスデバイスを停止する
pub fn stop_linux_haptic_device(device_id: &str) -> Result<()> {
    debug!("Linux触覚デバイス停止: デバイス={}", device_id);
    
    if device_id.starts_with("vibrator:") {
        // 振動子デバイスの場合
        let path = device_id.strip_prefix("vibrator:").unwrap_or("");
        let path = Path::new(path);
        
        // 振動を停止
        if path.join(VIBRATOR_ACTIVATE_FILE).exists() {
            write_to_file(&path.join(VIBRATOR_ACTIVATE_FILE), "0")?;
        } else if path.join(VIBRATOR_BRIGHTNESS_FILE).exists() {
            write_to_file(&path.join(VIBRATOR_BRIGHTNESS_FILE), "0")?;
        } else if path.join(VIBRATOR_STATE_FILE).exists() {
            write_to_file(&path.join(VIBRATOR_STATE_FILE), "0")?;
        }
    } else if device_id.starts_with("ff:") {
        // フォースフィードバックデバイスの場合
        // フォースフィードバックを停止する実装
    }
    
    Ok(())
}

// テスト
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_detect_devices() {
        let devices = detect_linux_haptic_devices();
        println!("検出されたデバイス: {:?}", devices);
    }
    
    #[test]
    fn test_vibrator_feedback() {
        // 実機でしか意味がないので、モック用のパスを使用
        let mock_path = Path::new("/tmp/mock_vibrator");
        let event = HapticEvent {
            id: "test".to_string(),
            intensity: HapticIntensity::Medium,
            pattern: HapticPattern::Click,
            duration: Duration::from_millis(50),
            target_device_id: None,
            timestamp: std::time::SystemTime::now(),
            custom_parameters: HashMap::new(),
        };
        
        // この関数は通常、モックパスが存在しないため失敗する
        // これは期待される動作
        let result = play_vibrator_feedback(mock_path, &event);
        assert!(result.is_err());
    }
} 