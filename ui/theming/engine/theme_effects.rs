// LumosDesktop テーマエフェクト
// 高度な視覚効果とアニメーションの管理システム

use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// エフェクトの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EffectType {
    /// フェードイン
    FadeIn,
    /// フェードアウト
    FadeOut,
    /// スライドイン
    SlideIn,
    /// スライドアウト
    SlideOut,
    /// スケールイン
    ScaleIn,
    /// スケールアウト
    ScaleOut,
    /// ブラー
    Blur,
    /// 色相シフト
    HueShift,
    /// 波紋
    Ripple,
    /// 光沢
    Glossy,
    /// 反射
    Reflection,
    /// パーティクル
    Particles,
    /// カスタム
    Custom(String),
}

/// アニメーション方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationDirection {
    /// 上方向
    Up,
    /// 下方向
    Down,
    /// 左方向
    Left,
    /// 右方向
    Right,
    /// 内側
    In,
    /// 外側
    Out,
}

/// エフェクト設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSettings {
    /// エフェクトの強さ (0.0-1.0)
    pub strength: f32,
    /// エフェクトの持続時間（ミリ秒）
    pub duration_ms: u32,
    /// エフェクトの遅延（ミリ秒）
    pub delay_ms: u32,
    /// イージング関数
    pub easing: super::EasingFunction,
    /// アニメーション方向（該当する場合）
    pub direction: Option<AnimationDirection>,
    /// カスタムパラメータ
    pub params: HashMap<String, f32>,
}

impl Default for EffectSettings {
    fn default() -> Self {
        Self {
            strength: 1.0,
            duration_ms: 300,
            delay_ms: 0,
            easing: super::EasingFunction::EaseOutCubic,
            direction: None,
            params: HashMap::new(),
        }
    }
}

/// エフェクトの進行状態
#[derive(Debug, Clone)]
pub struct EffectProgress {
    /// エフェクトの種類
    pub effect_type: EffectType,
    /// 開始時間
    pub start_time: Instant,
    /// 終了時間
    pub end_time: Instant,
    /// 現在の進行度 (0.0-1.0)
    pub progress: f32,
    /// 設定
    pub settings: EffectSettings,
    /// 完了したかどうか
    pub completed: bool,
}

impl EffectProgress {
    /// 新しいエフェクト進行状態を作成
    pub fn new(effect_type: EffectType, settings: EffectSettings) -> Self {
        let now = Instant::now();
        let delay = Duration::from_millis(settings.delay_ms as u64);
        let duration = Duration::from_millis(settings.duration_ms as u64);
        let start_time = now + delay;
        let end_time = start_time + duration;

        Self {
            effect_type,
            start_time,
            end_time,
            progress: 0.0,
            settings,
            completed: false,
        }
    }

    /// 進行状態を更新
    pub fn update(&mut self) -> bool {
        let now = Instant::now();
        
        if now < self.start_time {
            // まだ開始していない
            self.progress = 0.0;
            return false;
        }
        
        if now >= self.end_time {
            // 完了
            self.progress = 1.0;
            self.completed = true;
            return true;
        }
        
        // 進行中
        let total_duration = self.end_time.duration_since(self.start_time);
        let elapsed = now.duration_since(self.start_time);
        let raw_progress = elapsed.as_secs_f32() / total_duration.as_secs_f32();
        
        // イージング関数を適用
        self.progress = self.apply_easing(raw_progress);
        false
    }
    
    /// イージング関数を適用
    fn apply_easing(&self, t: f32) -> f32 {
        use super::EasingFunction::*;
        
        match self.settings.easing {
            Linear => t,
            EaseIn => t * t,
            EaseOut => t * (2.0 - t),
            EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            },
            EaseOutCubic => {
                let t1 = t - 1.0;
                1.0 + t1 * t1 * t1
            },
            EaseInOutSine => -(f32::cos(std::f32::consts::PI * t) - 1.0) / 2.0,
            Bounce => {
                let mut t = t;
                if t < (1.0 / 2.75) {
                    7.5625 * t * t
                } else if t < (2.0 / 2.75) {
                    t -= 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < (2.5 / 2.75) {
                    t -= 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    t -= 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            },
            Spring => {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0_f32.powf(-10.0 * t) * f32::sin((t * 10.0 - 0.75) * c4) + 1.0
                }
            },
        }
    }
    
    /// 現在の値を取得（エフェクトの進行度に基づく）
    pub fn get_value(&self) -> f32 {
        self.progress * self.settings.strength
    }
    
    /// 指定パラメータの現在の値を取得
    pub fn get_param_value(&self, param: &str) -> Option<f32> {
        self.settings.params.get(param).map(|&base| base * self.get_value())
    }
}

/// エフェクトマネージャー
#[derive(Debug)]
pub struct EffectManager {
    /// アクティブなエフェクト
    active_effects: HashMap<String, Vec<EffectProgress>>,
    /// グローバル設定
    global_settings: HashMap<EffectType, EffectSettings>,
}

impl EffectManager {
    /// 新しいエフェクトマネージャーを作成
    pub fn new() -> Self {
        Self {
            active_effects: HashMap::new(),
            global_settings: HashMap::new(),
        }
    }
    
    /// グローバルエフェクト設定を設定
    pub fn set_global_effect_settings(&mut self, effect_type: EffectType, settings: EffectSettings) {
        self.global_settings.insert(effect_type, settings);
    }
    
    /// 対象にエフェクトを適用
    pub fn apply_effect(&mut self, target_id: &str, effect_type: EffectType, custom_settings: Option<EffectSettings>) {
        let settings = match custom_settings {
            Some(s) => s,
            None => self.global_settings
                .get(&effect_type)
                .cloned()
                .unwrap_or_default(),
        };
        
        let effect = EffectProgress::new(effect_type, settings);
        
        let effects = self.active_effects
            .entry(target_id.to_string())
            .or_insert_with(Vec::new);
            
        effects.push(effect);
    }
    
    /// すべてのエフェクトを更新
    pub fn update_all(&mut self) {
        // 完了したエフェクトをリストから削除
        for effects in self.active_effects.values_mut() {
            effects.retain(|effect| {
                effect.update();
                !effect.completed
            });
        }
        
        // 空のターゲットを削除
        self.active_effects.retain(|_, effects| !effects.is_empty());
    }
    
    /// 対象の特定のエフェクトの進行状態を取得
    pub fn get_effect_progress(&self, target_id: &str, effect_type: EffectType) -> Option<&EffectProgress> {
        self.active_effects.get(target_id)
            .and_then(|effects| effects.iter()
                .find(|e| e.effect_type == effect_type))
    }
    
    /// 対象のすべてのエフェクトをクリア
    pub fn clear_effects(&mut self, target_id: &str) {
        self.active_effects.remove(target_id);
    }
    
    /// すべてのエフェクトをクリア
    pub fn clear_all_effects(&mut self) {
        self.active_effects.clear();
    }
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// シーンブレンドマネージャー（テーマ切り替え効果）
#[derive(Debug)]
pub struct SceneBlendManager {
    /// エフェクトマネージャー
    effect_manager: EffectManager,
    /// 現在のブレンド
    current_blend: Option<(String, String, f32)>,
    /// ブレンド設定
    blend_settings: HashMap<String, EffectSettings>,
}

impl SceneBlendManager {
    /// 新しいシーンブレンドマネージャーを作成
    pub fn new() -> Self {
        Self {
            effect_manager: EffectManager::new(),
            current_blend: None,
            blend_settings: HashMap::new(),
        }
    }
    
    /// ブレンド設定を追加
    pub fn add_blend_setting(&mut self, name: &str, settings: EffectSettings) {
        self.blend_settings.insert(name.to_string(), settings);
    }
    
    /// テーマ切り替えブレンドを開始
    pub fn start_theme_blend(&mut self, from_theme: &str, to_theme: &str, blend_type: &str) -> bool {
        if self.current_blend.is_some() {
            // すでにブレンド中
            return false;
        }
        
        let settings = match self.blend_settings.get(blend_type) {
            Some(s) => s.clone(),
            None => {
                // デフォルトのブレンド設定を使用
                let mut settings = EffectSettings::default();
                settings.duration_ms = 500;
                settings
            }
        };
        
        self.current_blend = Some((from_theme.to_string(), to_theme.to_string(), 0.0));
        
        let effect_type = match blend_type {
            "fade" => EffectType::FadeIn,
            "slide_left" => {
                let mut settings = settings.clone();
                settings.direction = Some(AnimationDirection::Left);
                self.effect_manager.apply_effect("theme_transition", EffectType::SlideIn, Some(settings));
                return true;
            },
            "zoom" => EffectType::ScaleIn,
            _ => EffectType::FadeIn,
        };
        
        self.effect_manager.apply_effect("theme_transition", effect_type, Some(settings));
        true
    }
    
    /// ブレンドを更新
    pub fn update(&mut self) -> Option<f32> {
        if self.current_blend.is_none() {
            return None;
        }
        
        self.effect_manager.update_all();
        
        // 進行状態を取得
        let progress = self.effect_manager
            .get_effect_progress("theme_transition", EffectType::FadeIn)
            .or_else(|| self.effect_manager.get_effect_progress("theme_transition", EffectType::SlideIn))
            .or_else(|| self.effect_manager.get_effect_progress("theme_transition", EffectType::ScaleIn));
        
        if let Some(progress) = progress {
            if let Some((_, _, ref mut blend_value)) = self.current_blend {
                *blend_value = progress.progress;
                
                if progress.completed {
                    // ブレンド完了
                    self.effect_manager.clear_effects("theme_transition");
                    let result = *blend_value;
                    self.current_blend = None;
                    return Some(result);
                }
                
                return Some(*blend_value);
            }
        } else {
            // エフェクトが見つからない
            self.current_blend = None;
        }
        
        None
    }
    
    /// 現在のブレンド情報を取得
    pub fn get_current_blend(&self) -> Option<(&str, &str, f32)> {
        self.current_blend.as_ref().map(|(from, to, value)| (from.as_str(), to.as_str(), *value))
    }
}

impl Default for SceneBlendManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    
    #[test]
    fn test_effect_progress() {
        let mut settings = EffectSettings::default();
        settings.duration_ms = 100; // テスト用に短く
        
        let mut effect = EffectProgress::new(EffectType::FadeIn, settings);
        
        // 初期状態
        assert_eq!(effect.progress, 0.0);
        assert!(!effect.completed);
        
        // 更新
        effect.update();
        assert!(effect.progress >= 0.0);
        
        // 時間経過をシミュレート
        sleep(Duration::from_millis(150));
        
        // 完了状態
        effect.update();
        assert_eq!(effect.progress, 1.0);
        assert!(effect.completed);
    }
    
    #[test]
    fn test_effect_manager() {
        let mut manager = EffectManager::new();
        
        // エフェクト設定
        let mut settings = EffectSettings::default();
        settings.duration_ms = 50; // テスト用に短く
        
        // エフェクト適用
        manager.apply_effect("test_button", EffectType::FadeIn, Some(settings.clone()));
        
        // エフェクトの存在を確認
        let effect = manager.get_effect_progress("test_button", EffectType::FadeIn);
        assert!(effect.is_some());
        
        // 更新
        manager.update_all();
        
        // 時間経過をシミュレート
        sleep(Duration::from_millis(100));
        
        // 完了後の更新
        manager.update_all();
        
        // エフェクトが削除されているか確認
        let effect = manager.get_effect_progress("test_button", EffectType::FadeIn);
        assert!(effect.is_none());
    }
    
    #[test]
    fn test_scene_blend() {
        let mut blend_manager = SceneBlendManager::new();
        
        // ブレンド設定
        let mut settings = EffectSettings::default();
        settings.duration_ms = 50; // テスト用に短く
        blend_manager.add_blend_setting("fade", settings);
        
        // ブレンド開始
        assert!(blend_manager.start_theme_blend("light", "dark", "fade"));
        
        // 初期状態
        let blend = blend_manager.get_current_blend();
        assert!(blend.is_some());
        let (from, to, value) = blend.unwrap();
        assert_eq!(from, "light");
        assert_eq!(to, "dark");
        assert_eq!(value, 0.0);
        
        // 更新
        blend_manager.update();
        
        // 時間経過をシミュレート
        sleep(Duration::from_millis(100));
        
        // 完了後の更新
        let final_value = blend_manager.update();
        
        // ブレンドが完了しているか確認
        assert!(final_value.is_some());
        assert_eq!(final_value.unwrap(), 1.0);
        assert!(blend_manager.get_current_blend().is_none());
    }
} 