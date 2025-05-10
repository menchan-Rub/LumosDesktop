// LumosDesktop エフェクトマネージャ
// 視覚効果の管理と適用を担当

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

use crate::core::window_manager::scene_graph::NodeId;

/// エフェクトの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectType {
    /// フェードイン
    FadeIn,
    /// フェードアウト
    FadeOut,
    /// スケールイン (拡大)
    ScaleIn,
    /// スケールアウト (縮小)
    ScaleOut,
    /// スライドイン
    SlideIn(SlideDirection),
    /// スライドアウト
    SlideOut(SlideDirection),
    /// ブラー (ぼかし)
    Blur,
    /// シャープ (鮮明化)
    Sharpen,
    /// 色変換
    ColorTransform,
    /// 波紋
    Ripple,
    /// エラスティック (弾性)
    Elastic,
    /// カスタムエフェクト
    Custom(u32),
}

/// スライド方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlideDirection {
    FromTop,
    FromBottom,
    FromLeft,
    FromRight,
}

/// エフェクト状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectState {
    /// 準備完了
    Ready,
    /// 実行中
    Running,
    /// 完了
    Completed,
    /// キャンセル
    Cancelled,
    /// 失敗
    Failed,
}

/// イージング関数の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingType {
    /// 線形
    Linear,
    /// イーズイン (徐々に加速)
    EaseIn,
    /// イーズアウト (徐々に減速)
    EaseOut,
    /// イーズインアウト (加速後減速)
    EaseInOut,
    /// バウンス
    Bounce,
    /// エラスティック
    Elastic,
    /// バック
    Back,
}

/// トランジションエフェクト
pub struct TransitionEffect {
    /// エフェクトの種類
    effect_type: EffectType,
    /// 開始時刻
    start_time: Option<Instant>,
    /// 期間
    duration: Duration,
    /// イージングタイプ
    easing: EasingType,
    /// 状態
    state: EffectState,
    /// 進行状況 (0.0 〜 1.0)
    progress: f32,
    /// カスタムパラメータ
    params: HashMap<String, f32>,
}

impl TransitionEffect {
    /// 新しいトランジションエフェクトを作成
    pub fn new(effect_type: EffectType, duration_ms: u32) -> Self {
        Self {
            effect_type,
            start_time: None,
            duration: Duration::from_millis(duration_ms as u64),
            easing: EasingType::EaseInOut,
            state: EffectState::Ready,
            progress: 0.0,
            params: HashMap::new(),
        }
    }
    
    /// イージングタイプを設定
    pub fn with_easing(mut self, easing: EasingType) -> Self {
        self.easing = easing;
        self
    }
    
    /// パラメータを追加
    pub fn with_param(mut self, name: &str, value: f32) -> Self {
        self.params.insert(name.to_string(), value);
        self
    }
    
    /// エフェクトを開始
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.state = EffectState::Running;
    }
    
    /// エフェクトを更新
    pub fn update(&mut self) -> bool {
        if self.state != EffectState::Running {
            return false;
        }
        
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            
            if elapsed >= self.duration {
                self.progress = 1.0;
                self.state = EffectState::Completed;
                return true;
            }
            
            let raw_progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
            self.progress = self.apply_easing(raw_progress);
            
            true
        } else {
            false
        }
    }
    
    /// エフェクトをキャンセル
    pub fn cancel(&mut self) {
        self.state = EffectState::Cancelled;
    }
    
    /// イージング関数を適用
    fn apply_easing(&self, t: f32) -> f32 {
        match self.easing {
            EasingType::Linear => t,
            EasingType::EaseIn => t * t,
            EasingType::EaseOut => t * (2.0 - t),
            EasingType::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            EasingType::Bounce => {
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
            }
            EasingType::Elastic => {
                let p = 0.3;
                let s = p / 4.0;
                let t = t - 1.0;
                -(2.0f32.powf(10.0 * t) * (t - s).sin() * (2.0 * std::f32::consts::PI / p).sin())
            }
            EasingType::Back => {
                let s = 1.70158;
                t * t * ((s + 1.0) * t - s)
            }
        }
    }
    
    /// 現在の進行状況を取得
    pub fn get_progress(&self) -> f32 {
        self.progress
    }
    
    /// 現在の状態を取得
    pub fn get_state(&self) -> EffectState {
        self.state
    }
    
    /// エフェクトタイプを取得
    pub fn get_effect_type(&self) -> EffectType {
        self.effect_type
    }
}

/// アクティブなエフェクト情報
struct ActiveEffect {
    /// エフェクト
    effect: TransitionEffect,
    /// 対象ノード
    target: Option<NodeId>,
    /// 優先度
    priority: u32,
    /// コールバック
    callback: Option<Box<dyn Fn(f32) -> bool + Send + Sync>>,
}

/// エフェクトマネージャ
pub struct EffectsManager {
    /// アクティブなエフェクト
    active_effects: VecDeque<ActiveEffect>,
    
    /// エフェクトリミッター (同時に適用できるエフェクト数)
    effect_limit: usize,
    
    /// 最後の更新時刻
    last_update: Instant,
    
    /// グローバルな有効/無効フラグ
    enabled: bool,
    
    /// エフェクトファクトリ
    effect_factories: HashMap<EffectType, Box<dyn Fn(u32) -> TransitionEffect + Send + Sync>>,
}

impl EffectsManager {
    /// 新しいエフェクトマネージャを作成
    pub fn new() -> Self {
        let mut manager = Self {
            active_effects: VecDeque::new(),
            effect_limit: 32,
            last_update: Instant::now(),
            enabled: true,
            effect_factories: HashMap::new(),
        };
        
        // デフォルトのエフェクトファクトリを登録
        manager.register_default_factories();
        
        manager
    }
    
    /// デフォルトのエフェクトファクトリを登録
    fn register_default_factories(&mut self) {
        // フェードイン
        self.register_effect_factory(EffectType::FadeIn, Box::new(|duration| {
            TransitionEffect::new(EffectType::FadeIn, duration)
                .with_easing(EasingType::EaseInOut)
        }));
        
        // フェードアウト
        self.register_effect_factory(EffectType::FadeOut, Box::new(|duration| {
            TransitionEffect::new(EffectType::FadeOut, duration)
                .with_easing(EasingType::EaseInOut)
        }));
        
        // スケールイン
        self.register_effect_factory(EffectType::ScaleIn, Box::new(|duration| {
            TransitionEffect::new(EffectType::ScaleIn, duration)
                .with_easing(EasingType::EaseOutBack)
                .with_param("start_scale", 0.8)
        }));
        
        // スケールアウト
        self.register_effect_factory(EffectType::ScaleOut, Box::new(|duration| {
            TransitionEffect::new(EffectType::ScaleOut, duration)
                .with_easing(EasingType::EaseInBack)
                .with_param("end_scale", 0.8)
        }));
    }
    
    /// エフェクトファクトリを登録
    pub fn register_effect_factory<F>(&mut self, effect_type: EffectType, factory: Box<F>)
    where
        F: Fn(u32) -> TransitionEffect + Send + Sync + 'static,
    {
        self.effect_factories.insert(effect_type, factory);
    }
    
    /// エフェクトを追加
    pub fn add_effect(&mut self, mut effect: TransitionEffect, target: Option<NodeId>) -> Result<(), String> {
        if !self.enabled {
            return Err("エフェクトマネージャが無効化されています".to_string());
        }
        
        // リミットに達していたら、最も古いエフェクトを削除
        if self.active_effects.len() >= self.effect_limit {
            self.active_effects.pop_front();
        }
        
        // エフェクトを開始
        effect.start();
        
        // アクティブエフェクトに追加
        self.active_effects.push_back(ActiveEffect {
            effect,
            target,
            priority: 0,
            callback: None,
        });
        
        Ok(())
    }
    
    /// エフェクトを追加（コールバック付き）
    pub fn add_effect_with_callback<F>(&mut self, mut effect: TransitionEffect, target: Option<NodeId>, callback: F) -> Result<(), String>
    where
        F: Fn(f32) -> bool + Send + Sync + 'static,
    {
        if !self.enabled {
            return Err("エフェクトマネージャが無効化されています".to_string());
        }
        
        // リミットに達していたら、最も古いエフェクトを削除
        if self.active_effects.len() >= self.effect_limit {
            self.active_effects.pop_front();
        }
        
        // エフェクトを開始
        effect.start();
        
        // アクティブエフェクトに追加
        self.active_effects.push_back(ActiveEffect {
            effect,
            target,
            priority: 0,
            callback: Some(Box::new(callback)),
        });
        
        Ok(())
    }
    
    /// ファクトリを使用してエフェクトを追加
    pub fn add_effect_from_factory(&mut self, effect_type: EffectType, duration_ms: u32, target: Option<NodeId>) -> Result<(), String> {
        if let Some(factory) = self.effect_factories.get(&effect_type) {
            let effect = factory(duration_ms);
            self.add_effect(effect, target)
        } else {
            Err(format!("エフェクトタイプ {:?} のファクトリが見つかりません", effect_type))
        }
    }
    
    /// 特定のターゲットのエフェクトをキャンセル
    pub fn cancel_effects_for_target(&mut self, target: NodeId) {
        for active_effect in &mut self.active_effects {
            if let Some(effect_target) = active_effect.target {
                if effect_target == target {
                    active_effect.effect.cancel();
                }
            }
        }
    }
    
    /// すべてのエフェクトをキャンセル
    pub fn cancel_all_effects(&mut self) {
        for active_effect in &mut self.active_effects {
            active_effect.effect.cancel();
        }
    }
    
    /// エフェクトマネージャを有効化/無効化
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.cancel_all_effects();
        }
    }
    
    /// エフェクトリミットを設定
    pub fn set_effect_limit(&mut self, limit: usize) {
        self.effect_limit = limit;
        
        // リミットを超えたエフェクトを削除
        while self.active_effects.len() > self.effect_limit {
            self.active_effects.pop_front();
        }
    }
    
    /// エフェクトを更新
    pub fn update(&mut self) {
        if !self.enabled {
            return;
        }
        
        let mut completed_indices = Vec::new();
        
        // 各エフェクトを更新
        for (i, active_effect) in self.active_effects.iter_mut().enumerate() {
            let updated = active_effect.effect.update();
            
            if updated {
                // コールバックがあれば実行
                if let Some(ref callback) = active_effect.callback {
                    let progress = active_effect.effect.get_progress();
                    if !callback(progress) {
                        active_effect.effect.cancel();
                    }
                }
                
                // 完了したエフェクトをマーク
                if active_effect.effect.get_state() == EffectState::Completed ||
                   active_effect.effect.get_state() == EffectState::Cancelled {
                    completed_indices.push(i);
                }
            }
        }
        
        // 完了したエフェクトを削除（後ろから削除）
        for i in completed_indices.into_iter().rev() {
            if i < self.active_effects.len() {
                self.active_effects.remove(i);
            }
        }
        
        self.last_update = Instant::now();
    }
    
    /// アクティブなエフェクト数を取得
    pub fn get_active_effect_count(&self) -> usize {
        self.active_effects.len()
    }
    
    /// 有効かどうかを取得
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_transition_effect() {
        let mut effect = TransitionEffect::new(EffectType::FadeIn, 100);
        
        // 初期状態
        assert_eq!(effect.get_state(), EffectState::Ready);
        assert_eq!(effect.get_progress(), 0.0);
        
        // 開始
        effect.start();
        assert_eq!(effect.get_state(), EffectState::Running);
        
        // 少し待機
        thread::sleep(Duration::from_millis(50));
        
        // 更新
        effect.update();
        let progress = effect.get_progress();
        assert!(progress > 0.0 && progress < 1.0); // 進行中
        
        // 完了まで待機
        thread::sleep(Duration::from_millis(60));
        
        // 更新
        effect.update();
        assert_eq!(effect.get_state(), EffectState::Completed);
        assert_eq!(effect.get_progress(), 1.0);
    }
    
    #[test]
    fn test_effects_manager() {
        let mut manager = EffectsManager::new();
        
        // エフェクト追加
        let fade_in = TransitionEffect::new(EffectType::FadeIn, 100);
        manager.add_effect(fade_in, Some(NodeId(1))).unwrap();
        
        assert_eq!(manager.get_active_effect_count(), 1);
        
        // 更新
        manager.update();
        assert_eq!(manager.get_active_effect_count(), 1); // まだ実行中
        
        // 完了まで待機
        thread::sleep(Duration::from_millis(110));
        
        // 更新
        manager.update();
        assert_eq!(manager.get_active_effect_count(), 0); // 完了して削除
    }
} 