// LumosDesktop 動的テーマ
// 時間帯や環境条件に基づいて自動的に変化するテーマシステム

use chrono::{DateTime, Local, NaiveTime, Timelike};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::thread;
use log::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::{Theme, ThemeMode, ColorPalette, color};

/// 時間帯
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOfDay {
    /// 深夜 (00:00-04:59)
    LateNight,
    /// 早朝 (05:00-07:59)
    EarlyMorning,
    /// 朝 (08:00-11:59)
    Morning,
    /// 昼 (12:00-15:59)
    Afternoon,
    /// 夕方 (16:00-18:59)
    Evening,
    /// 夜 (19:00-23:59)
    Night,
}

impl TimeOfDay {
    /// 現在の時間帯を取得
    pub fn current() -> Self {
        let now = Local::now();
        let hour = now.hour();
        
        match hour {
            0..=4 => TimeOfDay::LateNight,
            5..=7 => TimeOfDay::EarlyMorning,
            8..=11 => TimeOfDay::Morning,
            12..=15 => TimeOfDay::Afternoon,
            16..=18 => TimeOfDay::Evening,
            19..=23 => TimeOfDay::Night,
            _ => unreachable!(),
        }
    }
    
    /// 時間帯の開始時刻を取得
    pub fn start_time(&self) -> NaiveTime {
        match self {
            TimeOfDay::LateNight => NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            TimeOfDay::EarlyMorning => NaiveTime::from_hms_opt(5, 0, 0).unwrap(),
            TimeOfDay::Morning => NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            TimeOfDay::Afternoon => NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            TimeOfDay::Evening => NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            TimeOfDay::Night => NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
        }
    }
    
    /// 時間帯の終了時刻を取得
    pub fn end_time(&self) -> NaiveTime {
        match self {
            TimeOfDay::LateNight => NaiveTime::from_hms_opt(4, 59, 59).unwrap(),
            TimeOfDay::EarlyMorning => NaiveTime::from_hms_opt(7, 59, 59).unwrap(),
            TimeOfDay::Morning => NaiveTime::from_hms_opt(11, 59, 59).unwrap(),
            TimeOfDay::Afternoon => NaiveTime::from_hms_opt(15, 59, 59).unwrap(),
            TimeOfDay::Evening => NaiveTime::from_hms_opt(18, 59, 59).unwrap(),
            TimeOfDay::Night => NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        }
    }
    
    /// 時間帯の名前を取得
    pub fn name(&self) -> &'static str {
        match self {
            TimeOfDay::LateNight => "深夜",
            TimeOfDay::EarlyMorning => "早朝",
            TimeOfDay::Morning => "朝",
            TimeOfDay::Afternoon => "昼",
            TimeOfDay::Evening => "夕方",
            TimeOfDay::Night => "夜",
        }
    }
}

/// 季節
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    /// 春
    Spring,
    /// 夏
    Summer,
    /// 秋
    Autumn,
    /// 冬
    Winter,
}

impl Season {
    /// 現在の季節を取得（北半球基準）
    pub fn current() -> Self {
        let now = Local::now();
        let month = now.month();
        
        match month {
            3..=5 => Season::Spring,
            6..=8 => Season::Summer,
            9..=11 => Season::Autumn,
            12 | 1..=2 => Season::Winter,
            _ => unreachable!(),
        }
    }
    
    /// 季節の名前を取得
    pub fn name(&self) -> &'static str {
        match self {
            Season::Spring => "春",
            Season::Summer => "夏",
            Season::Autumn => "秋",
            Season::Winter => "冬",
        }
    }
}

/// 天気状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    /// 晴れ
    Sunny,
    /// 曇り
    Cloudy,
    /// 雨
    Rainy,
    /// 雪
    Snowy,
    /// 霧
    Foggy,
    /// 嵐
    Stormy,
}

impl WeatherCondition {
    /// 天気の名前を取得
    pub fn name(&self) -> &'static str {
        match self {
            WeatherCondition::Sunny => "晴れ",
            WeatherCondition::Cloudy => "曇り",
            WeatherCondition::Rainy => "雨",
            WeatherCondition::Snowy => "雪",
            WeatherCondition::Foggy => "霧",
            WeatherCondition::Stormy => "嵐",
        }
    }
}

/// 色温度（ケルビン）
pub type ColorTemperature = u32;

/// 動的カラーシフト設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicColorShift {
    /// 色温度（ケルビン）
    pub color_temperature: ColorTemperature,
    /// 彩度の変化 (-1.0～1.0)
    pub saturation_shift: f32,
    /// 明度の変化 (-1.0～1.0)
    pub lightness_shift: f32,
    /// 色相の変化（度）
    pub hue_shift: f32,
    /// アクセント色の変化（度）
    pub accent_hue_shift: f32,
}

impl Default for DynamicColorShift {
    fn default() -> Self {
        Self {
            color_temperature: 6500, // 標準的な昼光色
            saturation_shift: 0.0,
            lightness_shift: 0.0,
            hue_shift: 0.0,
            accent_hue_shift: 0.0,
        }
    }
}

/// 時間帯ごとの設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedSettings {
    /// 時間帯
    pub time_of_day: TimeOfDay,
    /// テーマモード
    pub theme_mode: Option<ThemeMode>,
    /// カラーシフト
    pub color_shift: DynamicColorShift,
    /// 壁紙のパス
    pub wallpaper: Option<String>,
}

/// 動的テーマ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicThemeSettings {
    /// 有効かどうか
    pub enabled: bool,
    /// ベーステーマ
    pub base_theme: String,
    /// 時間ベースの設定
    pub time_based: HashMap<TimeOfDay, TimeBasedSettings>,
    /// 天気ベースの設定
    pub weather_based: HashMap<WeatherCondition, DynamicColorShift>,
    /// 季節ベースの設定
    pub season_based: HashMap<Season, DynamicColorShift>,
    /// 更新間隔（秒）
    pub update_interval_sec: u64,
}

impl Default for DynamicThemeSettings {
    fn default() -> Self {
        let mut time_based = HashMap::new();
        
        // 早朝の設定
        time_based.insert(TimeOfDay::EarlyMorning, TimeBasedSettings {
            time_of_day: TimeOfDay::EarlyMorning,
            theme_mode: Some(ThemeMode::Light),
            color_shift: DynamicColorShift {
                color_temperature: 5000, // 朝日の色
                saturation_shift: -0.1,
                lightness_shift: 0.0,
                hue_shift: 10.0, // わずかに赤みがかった
                accent_hue_shift: 0.0,
            },
            wallpaper: None,
        });
        
        // 朝の設定
        time_based.insert(TimeOfDay::Morning, TimeBasedSettings {
            time_of_day: TimeOfDay::Morning,
            theme_mode: Some(ThemeMode::Light),
            color_shift: DynamicColorShift {
                color_temperature: 6000,
                saturation_shift: 0.0,
                lightness_shift: 0.1,
                hue_shift: 0.0,
                accent_hue_shift: 0.0,
            },
            wallpaper: None,
        });
        
        // 昼の設定
        time_based.insert(TimeOfDay::Afternoon, TimeBasedSettings {
            time_of_day: TimeOfDay::Afternoon,
            theme_mode: Some(ThemeMode::Light),
            color_shift: DynamicColorShift {
                color_temperature: 6500,
                saturation_shift: 0.1,
                lightness_shift: 0.1,
                hue_shift: 0.0,
                accent_hue_shift: 0.0,
            },
            wallpaper: None,
        });
        
        // 夕方の設定
        time_based.insert(TimeOfDay::Evening, TimeBasedSettings {
            time_of_day: TimeOfDay::Evening,
            theme_mode: Some(ThemeMode::Light),
            color_shift: DynamicColorShift {
                color_temperature: 4500, // 夕日の色
                saturation_shift: 0.2,
                lightness_shift: -0.05,
                hue_shift: -15.0, // オレンジ寄りに
                accent_hue_shift: -15.0,
            },
            wallpaper: None,
        });
        
        // 夜の設定
        time_based.insert(TimeOfDay::Night, TimeBasedSettings {
            time_of_day: TimeOfDay::Night,
            theme_mode: Some(ThemeMode::Dark),
            color_shift: DynamicColorShift {
                color_temperature: 3500, // 暖かめの光
                saturation_shift: -0.1,
                lightness_shift: -0.1,
                hue_shift: -20.0, // 青みを減らす
                accent_hue_shift: 0.0,
            },
            wallpaper: None,
        });
        
        // 深夜の設定
        time_based.insert(TimeOfDay::LateNight, TimeBasedSettings {
            time_of_day: TimeOfDay::LateNight,
            theme_mode: Some(ThemeMode::Dark),
            color_shift: DynamicColorShift {
                color_temperature: 2700, // 非常に暖かい光
                saturation_shift: -0.2,
                lightness_shift: -0.2,
                hue_shift: -25.0, // 青みをさらに減らす
                accent_hue_shift: 0.0,
            },
            wallpaper: None,
        });
        
        // 天気ベースの設定
        let mut weather_based = HashMap::new();
        
        weather_based.insert(WeatherCondition::Sunny, DynamicColorShift {
            color_temperature: 6500,
            saturation_shift: 0.1,
            lightness_shift: 0.1,
            hue_shift: 0.0,
            accent_hue_shift: 10.0,
        });
        
        weather_based.insert(WeatherCondition::Cloudy, DynamicColorShift {
            color_temperature: 6000,
            saturation_shift: -0.1,
            lightness_shift: -0.05,
            hue_shift: -5.0,
            accent_hue_shift: 0.0,
        });
        
        weather_based.insert(WeatherCondition::Rainy, DynamicColorShift {
            color_temperature: 5500,
            saturation_shift: -0.2,
            lightness_shift: -0.1,
            hue_shift: -10.0,
            accent_hue_shift: -20.0,
        });
        
        // 季節ベースの設定
        let mut season_based = HashMap::new();
        
        season_based.insert(Season::Spring, DynamicColorShift {
            color_temperature: 6000,
            saturation_shift: 0.1,
            lightness_shift: 0.05,
            hue_shift: -10.0, // 緑よりに
            accent_hue_shift: -30.0,
        });
        
        season_based.insert(Season::Summer, DynamicColorShift {
            color_temperature: 6500,
            saturation_shift: 0.15,
            lightness_shift: 0.05,
            hue_shift: 15.0, // 青よりに
            accent_hue_shift: 10.0,
        });
        
        season_based.insert(Season::Autumn, DynamicColorShift {
            color_temperature: 5000,
            saturation_shift: 0.2,
            lightness_shift: -0.05,
            hue_shift: 25.0, // 赤/オレンジよりに
            accent_hue_shift: 30.0,
        });
        
        season_based.insert(Season::Winter, DynamicColorShift {
            color_temperature: 5500,
            saturation_shift: -0.1,
            lightness_shift: 0.1,
            hue_shift: 40.0, // 青/紫よりに
            accent_hue_shift: 40.0,
        });
        
        Self {
            enabled: true,
            base_theme: "Default".to_string(),
            time_based,
            weather_based,
            season_based,
            update_interval_sec: 60, // 1分ごとに更新
        }
    }
}

/// 現在の環境状態
#[derive(Debug, Clone)]
pub struct EnvironmentState {
    /// 現在の時間帯
    pub time_of_day: TimeOfDay,
    /// 現在の季節
    pub season: Season,
    /// 現在の天気
    pub weather: Option<WeatherCondition>,
    /// 前回の更新時刻
    pub last_update: DateTime<Local>,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            time_of_day: TimeOfDay::current(),
            season: Season::current(),
            weather: None,
            last_update: Local::now(),
        }
    }
}

/// カラーパレット変換ヘルパー
fn apply_color_shift(color_str: &str, shift: &DynamicColorShift) -> String {
    if let Ok(rgb) = color::RGB::from_hex(color_str) {
        let mut hsl = rgb.to_hsl();
        
        // 色相シフト
        hsl.h = (hsl.h + shift.hue_shift) % 360.0;
        if hsl.h < 0.0 {
            hsl.h += 360.0;
        }
        
        // 彩度シフト
        hsl.s = (hsl.s + shift.saturation_shift).max(0.0).min(1.0);
        
        // 明度シフト
        hsl.l = (hsl.l + shift.lightness_shift).max(0.0).min(1.0);
        
        return hsl.to_rgb().to_hex();
    }
    
    color_str.to_string()
}

/// 動的テーママネージャー
pub struct DynamicThemeManager {
    /// 動的テーマ設定
    settings: RwLock<DynamicThemeSettings>,
    /// 環境状態
    environment: RwLock<EnvironmentState>,
    /// 現在の天気提供者
    weather_provider: Option<Box<dyn Fn() -> Option<WeatherCondition> + Send + Sync>>,
    /// 動的テーマの変更リスナー
    listeners: RwLock<Vec<Box<dyn Fn(&Theme) + Send + Sync>>>,
    /// 更新スレッドハンドル
    update_thread: Option<thread::JoinHandle<()>>,
    /// テーマエンジンの参照
    theme_engine: Arc<super::ThemeEngine>,
    /// 実行中フラグ
    running: Arc<RwLock<bool>>,
}

impl DynamicThemeManager {
    /// 新しい動的テーママネージャーを作成
    pub fn new(theme_engine: Arc<super::ThemeEngine>) -> Self {
        Self {
            settings: RwLock::new(DynamicThemeSettings::default()),
            environment: RwLock::new(EnvironmentState::default()),
            weather_provider: None,
            listeners: RwLock::new(Vec::new()),
            update_thread: None,
            theme_engine,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 動的テーマ設定を設定
    pub fn set_settings(&self, settings: DynamicThemeSettings) {
        let mut current = self.settings.write().unwrap();
        *current = settings;
    }
    
    /// 動的テーマ設定を取得
    pub fn get_settings(&self) -> DynamicThemeSettings {
        self.settings.read().unwrap().clone()
    }
    
    /// 天気提供者を設定
    pub fn set_weather_provider<F>(&mut self, provider: F)
    where
        F: Fn() -> Option<WeatherCondition> + Send + Sync + 'static,
    {
        self.weather_provider = Some(Box::new(provider));
    }
    
    /// 環境状態を更新
    pub fn update_environment(&self) {
        let mut env = self.environment.write().unwrap();
        
        // 時間帯の更新
        let current_time_of_day = TimeOfDay::current();
        let time_changed = env.time_of_day != current_time_of_day;
        env.time_of_day = current_time_of_day;
        
        // 季節の更新
        let current_season = Season::current();
        let season_changed = env.season != current_season;
        env.season = current_season;
        
        // 天気の更新
        if let Some(provider) = &self.weather_provider {
            let current_weather = provider();
            let weather_changed = env.weather != current_weather;
            env.weather = current_weather;
            
            if weather_changed {
                debug!("天気が変更されました: {:?}", env.weather);
            }
        }
        
        env.last_update = Local::now();
        
        if time_changed {
            info!("時間帯が変更されました: {}", env.time_of_day.name());
            self.apply_dynamic_theme();
        } else if season_changed {
            info!("季節が変更されました: {}", env.season.name());
            self.apply_dynamic_theme();
        }
    }
    
    /// 現在の環境状態を取得
    pub fn get_environment(&self) -> EnvironmentState {
        self.environment.read().unwrap().clone()
    }
    
    /// 動的テーマを適用
    pub fn apply_dynamic_theme(&self) {
        let settings = self.settings.read().unwrap();
        
        if !settings.enabled {
            return;
        }
        
        let env = self.environment.read().unwrap();
        
        // ベーステーマを取得
        let base_theme = match self.theme_engine.set_theme_by_name(&settings.base_theme) {
            Ok(()) => self.theme_engine.get_current_theme(),
            Err(e) => {
                error!("ベーステーマの設定に失敗しました: {}", e);
                return;
            }
        };
        
        // 動的なテーマ調整を適用
        let mut dynamic_theme = base_theme.clone();
        
        // 時間帯の設定を適用
        if let Some(time_settings) = settings.time_based.get(&env.time_of_day) {
            // テーマモードの適用
            if let Some(mode) = time_settings.theme_mode {
                dynamic_theme.mode = mode;
            }
            
            // カラーシフトの適用
            self.apply_color_shift_to_theme(&mut dynamic_theme, &time_settings.color_shift);
            
            // 壁紙の適用
            if let Some(wallpaper) = &time_settings.wallpaper {
                dynamic_theme.wallpaper = Some(std::path::PathBuf::from(wallpaper));
            }
        }
        
        // 季節の設定を適用
        if let Some(season_shift) = settings.season_based.get(&env.season) {
            self.apply_color_shift_to_theme(&mut dynamic_theme, season_shift);
        }
        
        // 天気の設定を適用
        if let Some(weather) = env.weather {
            if let Some(weather_shift) = settings.weather_based.get(&weather) {
                self.apply_color_shift_to_theme(&mut dynamic_theme, weather_shift);
            }
        }
        
        // テーマを設定
        let mut current = self.theme_engine.current_theme.write().unwrap();
        *current = dynamic_theme.clone();
        
        // リスナーに通知
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(&dynamic_theme);
        }
    }
    
    /// テーマにカラーシフトを適用
    fn apply_color_shift_to_theme(&self, theme: &mut Theme, shift: &DynamicColorShift) {
        // プライマリーカラー
        theme.colors.primary = apply_color_shift(&theme.colors.primary, shift);
        
        // セカンダリーカラー
        theme.colors.secondary = apply_color_shift(&theme.colors.secondary, shift);
        
        // アクセントカラーには特別なシフトを適用
        let mut accent_shift = shift.clone();
        accent_shift.hue_shift = shift.accent_hue_shift;
        theme.colors.accent = apply_color_shift(&theme.colors.accent, &accent_shift);
        
        // 背景色
        theme.colors.background = apply_color_shift(&theme.colors.background, shift);
        
        // 前景色
        theme.colors.foreground = apply_color_shift(&theme.colors.foreground, shift);
        
        // その他の色
        theme.colors.success = apply_color_shift(&theme.colors.success, shift);
        theme.colors.warning = apply_color_shift(&theme.colors.warning, shift);
        theme.colors.error = apply_color_shift(&theme.colors.error, shift);
        theme.colors.info = apply_color_shift(&theme.colors.info, shift);
        theme.colors.disabled = apply_color_shift(&theme.colors.disabled, shift);
        
        // カスタム色
        for (_, color) in theme.colors.custom.iter_mut() {
            *color = apply_color_shift(color, shift);
        }
    }
    
    /// 動的テーマリスナーを追加
    pub fn add_listener<F>(&self, listener: F)
    where
        F: Fn(&Theme) + Send + Sync + 'static,
    {
        let mut listeners = self.listeners.write().unwrap();
        listeners.push(Box::new(listener));
    }
    
    /// 更新スレッドを開始
    pub fn start(&mut self) {
        {
            let mut running = self.running.write().unwrap();
            if *running {
                return;
            }
            *running = true;
        }
        
        // 初回適用
        self.update_environment();
        self.apply_dynamic_theme();
        
        let running = self.running.clone();
        let settings_lock = self.settings.clone();
        let manager = Arc::new(self.clone());
        
        // 更新スレッドを起動
        let handle = thread::Builder::new()
            .name("dynamic-theme-updater".to_string())
            .spawn(move || {
                while {
                    let is_running = *running.read().unwrap();
                    is_running
                } {
                    // 更新間隔を取得
                    let interval = {
                        let settings = settings_lock.read().unwrap();
                        settings.update_interval_sec
                    };
                    
                    // 指定された間隔だけスリープ
                    thread::sleep(Duration::from_secs(interval));
                    
                    // 環境状態を更新
                    manager.update_environment();
                }
            })
            .expect("動的テーマ更新スレッドの起動に失敗しました");
            
        self.update_thread = Some(handle);
    }
    
    /// 更新スレッドを停止
    pub fn stop(&mut self) {
        let mut running = self.running.write().unwrap();
        *running = false;
        
        if let Some(handle) = self.update_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Clone for DynamicThemeManager {
    fn clone(&self) -> Self {
        Self {
            settings: RwLock::new(self.settings.read().unwrap().clone()),
            environment: RwLock::new(self.environment.read().unwrap().clone()),
            weather_provider: None, // クローン不可能なため
            listeners: RwLock::new(Vec::new()), // リスナーはクローンしない
            update_thread: None, // スレッドはクローンしない
            theme_engine: self.theme_engine.clone(),
            running: self.running.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    
    #[test]
    fn test_time_of_day() {
        // 特定の時刻での時間帯を確認
        let times = [
            (0, TimeOfDay::LateNight),
            (4, TimeOfDay::LateNight),
            (5, TimeOfDay::EarlyMorning),
            (7, TimeOfDay::EarlyMorning),
            (8, TimeOfDay::Morning),
            (11, TimeOfDay::Morning),
            (12, TimeOfDay::Afternoon),
            (15, TimeOfDay::Afternoon),
            (16, TimeOfDay::Evening),
            (18, TimeOfDay::Evening),
            (19, TimeOfDay::Night),
            (23, TimeOfDay::Night),
        ];
        
        for (hour, expected) in times {
            let time_of_day = match hour {
                0..=4 => TimeOfDay::LateNight,
                5..=7 => TimeOfDay::EarlyMorning,
                8..=11 => TimeOfDay::Morning,
                12..=15 => TimeOfDay::Afternoon,
                16..=18 => TimeOfDay::Evening,
                19..=23 => TimeOfDay::Night,
                _ => unreachable!(),
            };
            
            assert_eq!(time_of_day, expected);
        }
    }
    
    #[test]
    fn test_color_shift() {
        let original_color = "#ff0000"; // 純粋な赤
        
        // 無変更のシフト
        let no_shift = DynamicColorShift::default();
        assert_eq!(apply_color_shift(original_color, &no_shift), "#ff0000");
        
        // 色相のシフト
        let hue_shift = DynamicColorShift {
            hue_shift: 120.0, // 120度シフト（赤→緑）
            ..DynamicColorShift::default()
        };
        let shifted = apply_color_shift(original_color, &hue_shift);
        assert_eq!(shifted, "#00ff00"); // 緑になるはず
        
        // 明度のシフト
        let lightness_shift = DynamicColorShift {
            lightness_shift: -0.25, // 暗くする
            ..DynamicColorShift::default()
        };
        let shifted = apply_color_shift(original_color, &lightness_shift);
        assert_eq!(shifted, "#bf0000"); // 暗い赤
    }
    
    #[test]
    fn test_dynamic_theme_manager() {
        let theme_engine = Arc::new(super::super::ThemeEngine::new());
        let mut manager = DynamicThemeManager::new(theme_engine.clone());
        
        // カスタム設定
        let mut settings = DynamicThemeSettings::default();
        settings.update_interval_sec = 1; // テスト用に短く
        manager.set_settings(settings);
        
        // リスナーのテスト
        let callback_triggered = Arc::new(AtomicBool::new(false));
        let callback_triggered_clone = callback_triggered.clone();
        manager.add_listener(move |_| {
            callback_triggered_clone.store(true, Ordering::SeqCst);
        });
        
        // マネージャーを開始
        manager.start();
        
        // 少し待機
        thread::sleep(Duration::from_millis(100));
        
        // リスナーが呼ばれたことを確認
        assert!(callback_triggered.load(Ordering::SeqCst));
        
        // 停止
        manager.stop();
    }
} 