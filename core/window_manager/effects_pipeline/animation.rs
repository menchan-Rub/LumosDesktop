// アニメーションエフェクト実装
// LumosDesktop エフェクトパイプライン用のアニメーション機能

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::window_manager::scene_graph::SceneNode;
use super::effects_manager::{EasingType, EffectState};

/// アニメーション種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationType {
    /// キーフレームアニメーション
    Keyframe,
    
    /// スプリングアニメーション（バネのような動き）
    Spring,
    
    /// 経路アニメーション（指定した経路に沿って移動）
    Path,
    
    /// パーティクルアニメーション
    Particle,
    
    /// 揺れアニメーション
    Wobble,
    
    /// バウンスアニメーション
    Bounce,
    
    /// 弾性アニメーション
    Elastic,
    
    /// カスタムアニメーション
    Custom,
}

/// アニメーション対象のプロパティ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationProperty {
    /// X座標
    X,
    
    /// Y座標
    Y,
    
    /// 幅
    Width,
    
    /// 高さ
    Height,
    
    /// 透明度
    Opacity,
    
    /// 回転（度）
    Rotation,
    
    /// スケールX
    ScaleX,
    
    /// スケールY
    ScaleY,
    
    /// 色（RGBA）
    Color,
    
    /// 変形行列
    Transform,
    
    /// カスタムプロパティ
    Custom(u32),
}

/// キーフレーム
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// 時間（ミリ秒）
    pub time: u32,
    
    /// プロパティ値（f64）
    pub value: f64,
    
    /// イージング関数
    pub easing: EasingType,
}

/// アニメーションパラメータ
#[derive(Debug, Clone)]
pub struct AnimationParams {
    /// アニメーション時間（ミリ秒）
    pub duration: u32,
    
    /// 繰り返し回数（0は無限）
    pub repeat_count: u32,
    
    /// 往復アニメーション（trueの場合、逆方向にも再生）
    pub yoyo: bool,
    
    /// 遅延時間（ミリ秒）
    pub delay: u32,
    
    /// デフォルトのイージング関数
    pub default_easing: EasingType,
}

impl Default for AnimationParams {
    fn default() -> Self {
        Self {
            duration: 300,
            repeat_count: 1,
            yoyo: false,
            delay: 0,
            default_easing: EasingType::Linear,
        }
    }
}

/// キーフレームアニメーション
#[derive(Debug, Clone)]
pub struct KeyframeAnimation {
    /// アニメーションID
    pub id: String,
    
    /// アニメーション対象のプロパティ
    pub property: AnimationProperty,
    
    /// キーフレームリスト
    pub keyframes: Vec<Keyframe>,
    
    /// アニメーションパラメータ
    pub params: AnimationParams,
}

impl KeyframeAnimation {
    /// 新しいキーフレームアニメーションを作成
    pub fn new(
        id: &str,
        property: AnimationProperty,
        keyframes: Vec<Keyframe>,
        params: Option<AnimationParams>,
    ) -> Self {
        Self {
            id: id.to_string(),
            property,
            keyframes,
            params: params.unwrap_or_default(),
        }
    }
    
    /// キーフレームを追加
    pub fn add_keyframe(&mut self, time: u32, value: f64, easing: Option<EasingType>) -> &mut Self {
        self.keyframes.push(Keyframe {
            time,
            value,
            easing: easing.unwrap_or(self.params.default_easing),
        });
        
        // 時間順にソート
        self.keyframes.sort_by_key(|k| k.time);
        
        self
    }
    
    /// 指定時間の値を計算
    pub fn get_value_at(&self, time: u32) -> f64 {
        if self.keyframes.is_empty() {
            return 0.0;
        }
        
        if time <= self.keyframes[0].time {
            return self.keyframes[0].value;
        }
        
        if time >= self.keyframes.last().unwrap().time {
            return self.keyframes.last().unwrap().value;
        }
        
        // 前後のキーフレームを探す
        let mut prev_frame = &self.keyframes[0];
        
        for frame in &self.keyframes[1..] {
            if frame.time >= time {
                let progress = (time - prev_frame.time) as f64 / (frame.time - prev_frame.time) as f64;
                let eased_progress = apply_easing(progress, prev_frame.easing);
                
                return prev_frame.value + (frame.value - prev_frame.value) * eased_progress;
            }
            
            prev_frame = frame;
        }
        
        prev_frame.value
    }
}

/// アニメーションエフェクト
pub struct AnimationEffect {
    /// アニメーションID
    id: String,
    
    /// アニメーション種類
    animation_type: AnimationType,
    
    /// 開始時間
    start_time: Instant,
    
    /// 現在の状態
    state: EffectState,
    
    /// ターゲットノード
    target: Option<Arc<Mutex<SceneNode>>>,
    
    /// アニメーションデータ
    data: AnimationData,
    
    /// 現在の繰り返し回数
    current_repeat: u32,
    
    /// 現在の再生方向（true: 順方向、false: 逆方向）
    forward: bool,
    
    /// アップデートコールバック
    update_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
    
    /// 完了コールバック
    complete_callback: Option<Box<dyn FnOnce() + Send + Sync>>,
}

/// アニメーションデータ（各アニメーション種類に応じたデータ）
enum AnimationData {
    /// キーフレームアニメーション
    Keyframe(KeyframeAnimation),
    
    /// スプリングアニメーション
    Spring {
        /// 目標値
        target_value: f64,
        /// 初期値
        initial_value: f64,
        /// バネ定数
        stiffness: f64,
        /// 減衰係数
        damping: f64,
        /// 対象プロパティ
        property: AnimationProperty,
    },
    
    /// 経路アニメーション
    Path {
        /// 制御点（x, y座標のリスト）
        control_points: Vec<(f64, f64)>,
        /// アニメーションパラメータ
        params: AnimationParams,
    },
    
    /// パーティクルアニメーション
    Particle {
        /// パーティクル数
        count: u32,
        /// 重力
        gravity: f64,
        /// 寿命（ミリ秒）
        lifetime: u32,
        /// 速度範囲
        velocity_range: (f64, f64),
        /// 方向範囲（ラジアン）
        direction_range: (f64, f64),
    },
    
    /// 揺れアニメーション
    Wobble {
        /// 振幅
        amplitude: f64,
        /// 周波数
        frequency: f64,
        /// 減衰
        decay: f64,
        /// 対象プロパティ
        property: AnimationProperty,
    },
    
    /// カスタムアニメーション
    Custom {
        /// カスタムアップデーター関数
        updater: Box<dyn Fn(u32, f64) -> f64 + Send + Sync>,
        /// アニメーションパラメータ
        params: AnimationParams,
        /// 対象プロパティ
        property: AnimationProperty,
    },
}

impl AnimationEffect {
    /// 新しいキーフレームアニメーションエフェクトを作成
    pub fn new_keyframe(
        id: &str,
        animation: KeyframeAnimation,
        target: Option<Arc<Mutex<SceneNode>>>,
    ) -> Self {
        Self {
            id: id.to_string(),
            animation_type: AnimationType::Keyframe,
            start_time: Instant::now(),
            state: EffectState::Ready,
            target,
            data: AnimationData::Keyframe(animation),
            current_repeat: 0,
            forward: true,
            update_callback: None,
            complete_callback: None,
        }
    }
    
    /// 新しいスプリングアニメーションエフェクトを作成
    pub fn new_spring(
        id: &str,
        property: AnimationProperty,
        target_value: f64,
        initial_value: f64,
        stiffness: f64,
        damping: f64,
        target: Option<Arc<Mutex<SceneNode>>>,
    ) -> Self {
        Self {
            id: id.to_string(),
            animation_type: AnimationType::Spring,
            start_time: Instant::now(),
            state: EffectState::Ready,
            target,
            data: AnimationData::Spring {
                target_value,
                initial_value,
                stiffness,
                damping,
                property,
            },
            current_repeat: 0,
            forward: true,
            update_callback: None,
            complete_callback: None,
        }
    }
    
    /// 新しい揺れアニメーションエフェクトを作成
    pub fn new_wobble(
        id: &str,
        property: AnimationProperty,
        amplitude: f64,
        frequency: f64,
        decay: f64,
        target: Option<Arc<Mutex<SceneNode>>>,
    ) -> Self {
        Self {
            id: id.to_string(),
            animation_type: AnimationType::Wobble,
            start_time: Instant::now(),
            state: EffectState::Ready,
            target,
            data: AnimationData::Wobble {
                amplitude,
                frequency,
                decay,
                property,
            },
            current_repeat: 0,
            forward: true,
            update_callback: None,
            complete_callback: None,
        }
    }
    
    /// 新しいカスタムアニメーションエフェクトを作成
    pub fn new_custom<F>(
        id: &str,
        property: AnimationProperty,
        updater: F,
        params: AnimationParams,
        target: Option<Arc<Mutex<SceneNode>>>,
    ) -> Self
    where
        F: Fn(u32, f64) -> f64 + Send + Sync + 'static,
    {
        Self {
            id: id.to_string(),
            animation_type: AnimationType::Custom,
            start_time: Instant::now(),
            state: EffectState::Ready,
            target,
            data: AnimationData::Custom {
                updater: Box::new(updater),
                params,
                property,
            },
            current_repeat: 0,
            forward: true,
            update_callback: None,
            complete_callback: None,
        }
    }
    
    /// アニメーションを開始
    pub fn start(&mut self) {
        self.start_time = Instant::now();
        self.state = EffectState::Running;
        self.current_repeat = 0;
        self.forward = true;
    }
    
    /// アニメーションを更新
    pub fn update(&mut self) -> bool {
        if self.state != EffectState::Running {
            return false;
        }
        
        let elapsed = self.start_time.elapsed();
        let elapsed_ms = elapsed.as_millis() as u32;
        
        let (progress, completed) = match &self.data {
            AnimationData::Keyframe(animation) => {
                let params = &animation.params;
                let active_time = elapsed_ms.saturating_sub(params.delay);
                
                if active_time == 0 {
                    return false;
                }
                
                let cycle_time = if self.forward {
                    active_time % params.duration
                } else {
                    params.duration - (active_time % params.duration)
                };
                
                let current_value = animation.get_value_at(cycle_time);
                
                // ターゲットノードにプロパティを設定
                if let Some(target) = &self.target {
                    if let Ok(mut node) = target.lock() {
                        match animation.property {
                            AnimationProperty::X => node.set_position_x(current_value),
                            AnimationProperty::Y => node.set_position_y(current_value),
                            AnimationProperty::Width => node.set_width(current_value),
                            AnimationProperty::Height => node.set_height(current_value),
                            AnimationProperty::Opacity => node.set_opacity(current_value),
                            AnimationProperty::Rotation => node.set_rotation(current_value),
                            AnimationProperty::ScaleX => node.set_scale_x(current_value),
                            AnimationProperty::ScaleY => node.set_scale_y(current_value),
                            AnimationProperty::Color => {
                                // RGBA色値の変換は別途実装が必要
                            },
                            AnimationProperty::Transform => {
                                // 変形行列の設定は別途実装が必要
                            },
                            AnimationProperty::Custom(prop_id) => {
                                node.set_custom_property(prop_id, current_value);
                            },
                        }
                    }
                }
                
                // コールバックを呼び出し
                if let Some(callback) = &self.update_callback {
                    callback(current_value);
                }
                
                // 繰り返し処理
                let cycle_completed = active_time > 0 && active_time % params.duration == 0;
                
                if cycle_completed {
                    if params.yoyo {
                        self.forward = !self.forward;
                    }
                    
                    if params.repeat_count > 0 {
                        self.current_repeat += 1;
                        
                        if self.current_repeat >= params.repeat_count {
                            return true;
                        }
                    }
                }
                
                (active_time as f64 / params.duration as f64, 
                 params.repeat_count > 0 && self.current_repeat >= params.repeat_count)
            },
            AnimationData::Spring { target_value, initial_value, stiffness, damping, property } => {
                // スプリングアニメーションの計算
                let dt = elapsed.as_secs_f64();
                let x = *initial_value;
                let target = *target_value;
                
                // スプリングモデルに基づく位置計算
                // x" + 2*zeta*omega*x' + omega^2*x = omega^2*target
                let omega = (*stiffness).sqrt();
                let zeta = *damping / (2.0 * (*stiffness).sqrt());
                
                let mut current_value = 0.0;
                
                // 減衰の種類に応じた計算
                if zeta < 1.0 {
                    // 減衰振動
                    let omega_d = omega * (1.0 - zeta * zeta).sqrt();
                    let a = target - x;
                    let exp_term = (-zeta * omega * dt).exp();
                    
                    current_value = target - exp_term * (a * (zeta * omega / omega_d).cos() + 
                                                       (a * zeta * omega + (target - x)) / omega_d * (zeta * omega / omega_d).sin());
                } else if zeta == 1.0 {
                    // 臨界減衰
                    let a = target - x;
                    let b = (target - x) + a * omega;
                    
                    current_value = target - (a + b * dt) * (-omega * dt).exp();
                } else {
                    // 過減衰
                    let omega_d = omega * (zeta * zeta - 1.0).sqrt();
                    let r1 = -omega * (zeta + omega_d);
                    let r2 = -omega * (zeta - omega_d);
                    let a = (target - x) * r2 / (r2 - r1);
                    let b = (target - x) * r1 / (r1 - r2);
                    
                    current_value = target - (a * (r1 * dt).exp() + b * (r2 * dt).exp());
                }
                
                // ターゲットノードにプロパティを設定
                if let Some(target_node) = &self.target {
                    if let Ok(mut node) = target_node.lock() {
                        match property {
                            AnimationProperty::X => node.set_position_x(current_value),
                            AnimationProperty::Y => node.set_position_y(current_value),
                            AnimationProperty::Width => node.set_width(current_value),
                            AnimationProperty::Height => node.set_height(current_value),
                            AnimationProperty::Opacity => node.set_opacity(current_value),
                            AnimationProperty::Rotation => node.set_rotation(current_value),
                            AnimationProperty::ScaleX => node.set_scale_x(current_value),
                            AnimationProperty::ScaleY => node.set_scale_y(current_value),
                            AnimationProperty::Color => {
                                // RGBA色値の変換は別途実装が必要
                            },
                            AnimationProperty::Transform => {
                                // 変形行列の設定は別途実装が必要
                            },
                            AnimationProperty::Custom(prop_id) => {
                                node.set_custom_property(prop_id, current_value);
                            },
                        }
                    }
                }
                
                // コールバックを呼び出し
                if let Some(callback) = &self.update_callback {
                    callback(current_value);
                }
                
                // 完了判定（値が十分に近づいたら）
                let is_complete = (current_value - target_value).abs() < 0.01;
                
                (dt, is_complete)
            },
            AnimationData::Wobble { amplitude, frequency, decay, property } => {
                let dt = elapsed.as_secs_f64();
                
                // 減衰する正弦波
                let decay_factor = (-*decay * dt).exp();
                let angle = *frequency * dt * std::f64::consts::PI * 2.0;
                let current_value = *amplitude * decay_factor * angle.sin();
                
                // ターゲットノードにプロパティを設定
                if let Some(target_node) = &self.target {
                    if let Ok(mut node) = target_node.lock() {
                        match property {
                            AnimationProperty::X => node.set_position_x(current_value),
                            AnimationProperty::Y => node.set_position_y(current_value),
                            AnimationProperty::Width => node.set_width(current_value),
                            AnimationProperty::Height => node.set_height(current_value),
                            AnimationProperty::Opacity => node.set_opacity(current_value),
                            AnimationProperty::Rotation => node.set_rotation(current_value),
                            AnimationProperty::ScaleX => node.set_scale_x(current_value),
                            AnimationProperty::ScaleY => node.set_scale_y(current_value),
                            AnimationProperty::Color => {
                                // RGBA色値の変換は別途実装が必要
                            },
                            AnimationProperty::Transform => {
                                // 変形行列の設定は別途実装が必要
                            },
                            AnimationProperty::Custom(prop_id) => {
                                node.set_custom_property(prop_id, current_value);
                            },
                        }
                    }
                }
                
                // コールバックを呼び出し
                if let Some(callback) = &self.update_callback {
                    callback(current_value);
                }
                
                // 完了判定（振幅が十分に小さくなったら）
                let is_complete = decay_factor * *amplitude < 0.01;
                
                (dt, is_complete)
            },
            AnimationData::Custom { updater, params, property } => {
                let active_time = elapsed_ms.saturating_sub(params.delay);
                
                if active_time == 0 {
                    return false;
                }
                
                let cycle_time = if self.forward {
                    active_time % params.duration
                } else {
                    params.duration - (active_time % params.duration)
                };
                
                // カスタムアップデーター関数を呼び出し
                let current_value = updater(cycle_time, active_time as f64 / params.duration as f64);
                
                // ターゲットノードにプロパティを設定
                if let Some(target_node) = &self.target {
                    if let Ok(mut node) = target_node.lock() {
                        match property {
                            AnimationProperty::X => node.set_position_x(current_value),
                            AnimationProperty::Y => node.set_position_y(current_value),
                            AnimationProperty::Width => node.set_width(current_value),
                            AnimationProperty::Height => node.set_height(current_value),
                            AnimationProperty::Opacity => node.set_opacity(current_value),
                            AnimationProperty::Rotation => node.set_rotation(current_value),
                            AnimationProperty::ScaleX => node.set_scale_x(current_value),
                            AnimationProperty::ScaleY => node.set_scale_y(current_value),
                            AnimationProperty::Color => {
                                // RGBA色値の変換は別途実装が必要
                            },
                            AnimationProperty::Transform => {
                                // 変形行列の設定は別途実装が必要
                            },
                            AnimationProperty::Custom(prop_id) => {
                                node.set_custom_property(prop_id, current_value);
                            },
                        }
                    }
                }
                
                // コールバックを呼び出し
                if let Some(callback) = &self.update_callback {
                    callback(current_value);
                }
                
                // 繰り返し処理
                let cycle_completed = active_time > 0 && active_time % params.duration == 0;
                
                if cycle_completed {
                    if params.yoyo {
                        self.forward = !self.forward;
                    }
                    
                    if params.repeat_count > 0 {
                        self.current_repeat += 1;
                        
                        if self.current_repeat >= params.repeat_count {
                            return true;
                        }
                    }
                }
                
                (active_time as f64 / params.duration as f64, 
                 params.repeat_count > 0 && self.current_repeat >= params.repeat_count)
            },
            _ => (0.0, false), // 未実装のアニメーションタイプ
        };
        
        if completed {
            self.state = EffectState::Completed;
            
            // 完了コールバックを呼び出し
            if let Some(callback) = self.complete_callback.take() {
                callback();
            }
            
            true
        } else {
            false
        }
    }
    
    /// アニメーションを一時停止
    pub fn pause(&mut self) {
        if self.state == EffectState::Running {
            self.state = EffectState::Paused;
        }
    }
    
    /// アニメーションを再開
    pub fn resume(&mut self) {
        if self.state == EffectState::Paused {
            self.state = EffectState::Running;
        }
    }
    
    /// アニメーションを停止
    pub fn stop(&mut self) {
        self.state = EffectState::Stopped;
    }
    
    /// アップデートコールバックを設定
    pub fn set_update_callback<F>(&mut self, callback: F)
    where
        F: Fn(f64) + Send + Sync + 'static,
    {
        self.update_callback = Some(Box::new(callback));
    }
    
    /// 完了コールバックを設定
    pub fn set_complete_callback<F>(&mut self, callback: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.complete_callback = Some(Box::new(callback));
    }
    
    /// アニメーション情報を取得
    pub fn get_info(&self) -> (String, AnimationType, EffectState) {
        (self.id.clone(), self.animation_type, self.state)
    }
}

/// イージング関数を適用
fn apply_easing(progress: f64, easing_type: EasingType) -> f64 {
    match easing_type {
        EasingType::Linear => progress,
        EasingType::EaseIn => progress * progress,
        EasingType::EaseOut => progress * (2.0 - progress),
        EasingType::EaseInOut => {
            if progress < 0.5 {
                2.0 * progress * progress
            } else {
                -1.0 + (4.0 - 2.0 * progress) * progress
            }
        },
        EasingType::Sine => -(progress * std::f64::consts::PI * 0.5).cos() + 1.0,
        EasingType::Cubic => progress * progress * progress,
        EasingType::Elastic => {
            let p = 0.3;
            (-pow(2.0, 10.0 * (progress - 1.0)) * ((progress - 1.0 - p / 4.0) * (2.0 * std::f64::consts::PI) / p).sin())
        },
        EasingType::Bounce => {
            let mut progress = progress;
            if progress < 1.0 / 2.75 {
                7.5625 * progress * progress
            } else if progress < 2.0 / 2.75 {
                progress -= 1.5 / 2.75;
                7.5625 * progress * progress + 0.75
            } else if progress < 2.5 / 2.75 {
                progress -= 2.25 / 2.75;
                7.5625 * progress * progress + 0.9375
            } else {
                progress -= 2.625 / 2.75;
                7.5625 * progress * progress + 0.984375
            }
        },
    }
}

/// 累乗計算のヘルパー関数
fn pow(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

/// アニメーション管理クラス
pub struct AnimationManager {
    /// アクティブなアニメーション
    animations: HashMap<String, AnimationEffect>,
    
    /// アニメーション追加数の上限
    max_animations: usize,
    
    /// 有効/無効フラグ
    enabled: bool,
}

impl AnimationManager {
    /// 新しいアニメーション管理クラスを作成
    pub fn new(max_animations: Option<usize>) -> Self {
        Self {
            animations: HashMap::new(),
            max_animations: max_animations.unwrap_or(100),
            enabled: true,
        }
    }
    
    /// アニメーションを追加
    pub fn add_animation(&mut self, animation: AnimationEffect) -> Result<(), String> {
        if !self.enabled {
            return Err("アニメーション管理は無効です".to_string());
        }
        
        if self.animations.len() >= self.max_animations {
            return Err("アニメーション数の上限に達しました".to_string());
        }
        
        let (id, _, _) = animation.get_info();
        
        // 同じIDのアニメーションがあれば置き換え
        self.animations.insert(id, animation);
        
        Ok(())
    }
    
    /// アニメーションを更新
    pub fn update(&mut self) {
        if !self.enabled {
            return;
        }
        
        // 完了または停止したアニメーションを削除する
        let mut completed_ids = Vec::new();
        
        for (id, animation) in self.animations.iter_mut() {
            let completed = animation.update();
            
            if completed || animation.state == EffectState::Stopped {
                completed_ids.push(id.clone());
            }
        }
        
        for id in completed_ids {
            self.animations.remove(&id);
        }
    }
    
    /// アニメーションを取得
    pub fn get_animation(&self, id: &str) -> Option<&AnimationEffect> {
        self.animations.get(id)
    }
    
    /// アニメーションを取得（可変）
    pub fn get_animation_mut(&mut self, id: &str) -> Option<&mut AnimationEffect> {
        self.animations.get_mut(id)
    }
    
    /// アニメーションを削除
    pub fn remove_animation(&mut self, id: &str) -> bool {
        self.animations.remove(id).is_some()
    }
    
    /// すべてのアニメーションを削除
    pub fn clear_animations(&mut self) {
        self.animations.clear();
    }
    
    /// アニメーション数を取得
    pub fn get_animation_count(&self) -> usize {
        self.animations.len()
    }
    
    /// 管理クラスを有効化/無効化
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// キーフレームアニメーションを作成
    pub fn create_keyframe_animation(
        &mut self,
        id: &str,
        property: AnimationProperty,
        keyframes: Vec<Keyframe>,
        params: Option<AnimationParams>,
        target: Option<Arc<Mutex<SceneNode>>>,
    ) -> Result<(), String> {
        let animation = AnimationEffect::new_keyframe(
            id,
            KeyframeAnimation::new(id, property, keyframes, params),
            target,
        );
        
        self.add_animation(animation)
    }
    
    /// スプリングアニメーションを作成
    pub fn create_spring_animation(
        &mut self,
        id: &str,
        property: AnimationProperty,
        target_value: f64,
        initial_value: f64,
        stiffness: f64,
        damping: f64,
        target: Option<Arc<Mutex<SceneNode>>>,
    ) -> Result<(), String> {
        let animation = AnimationEffect::new_spring(
            id,
            property,
            target_value,
            initial_value,
            stiffness,
            damping,
            target,
        );
        
        self.add_animation(animation)
    }
    
    /// 揺れアニメーションを作成
    pub fn create_wobble_animation(
        &mut self,
        id: &str,
        property: AnimationProperty,
        amplitude: f64,
        frequency: f64,
        decay: f64,
        target: Option<Arc<Mutex<SceneNode>>>,
    ) -> Result<(), String> {
        let animation = AnimationEffect::new_wobble(
            id,
            property,
            amplitude,
            frequency,
            decay,
            target,
        );
        
        self.add_animation(animation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_keyframe_animation() {
        let mut animation = KeyframeAnimation::new(
            "test_animation",
            AnimationProperty::Opacity,
            vec![],
            None,
        );
        
        animation.add_keyframe(0, 0.0, None)
                 .add_keyframe(100, 0.5, Some(EasingType::Linear))
                 .add_keyframe(200, 1.0, Some(EasingType::EaseOut));
        
        assert_eq!(animation.get_value_at(0), 0.0);
        assert_eq!(animation.get_value_at(50), 0.25);
        assert_eq!(animation.get_value_at(100), 0.5);
        assert_eq!(animation.get_value_at(150), 0.75);
        assert_eq!(animation.get_value_at(200), 1.0);
        assert_eq!(animation.get_value_at(300), 1.0);
    }
    
    #[test]
    fn test_animation_manager() {
        let mut manager = AnimationManager::new(None);
        
        // キーフレームアニメーションの作成
        let keyframes = vec![
            Keyframe { time: 0, value: 0.0, easing: EasingType::Linear },
            Keyframe { time: 100, value: 1.0, easing: EasingType::Linear },
        ];
        
        let params = AnimationParams {
            duration: 100,
            repeat_count: 1,
            yoyo: false,
            delay: 0,
            default_easing: EasingType::Linear,
        };
        
        manager.create_keyframe_animation(
            "test_animation",
            AnimationProperty::Opacity,
            keyframes,
            Some(params),
            None,
        ).unwrap();
        
        assert_eq!(manager.get_animation_count(), 1);
        
        // アニメーションを取得
        let animation = manager.get_animation_mut("test_animation").unwrap();
        animation.start();
        
        // アニメーション更新のシミュレーション
        thread::sleep(Duration::from_millis(50));
        manager.update();
        
        thread::sleep(Duration::from_millis(100));
        manager.update();
        
        // アニメーションが完了していることを確認
        assert_eq!(manager.get_animation_count(), 0);
    }
} 