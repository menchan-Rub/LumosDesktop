// LumosDesktop エフェクトパイプライン
// 視覚効果処理のためのパイプラインシステム

//! # エフェクトパイプライン
//! 
//! LumosDesktopの視覚効果処理を担当するモジュールです。
//! ウィンドウやUIコンポーネントに対して様々な視覚効果を適用します。
//! 
//! ## 主な機能
//! 
//! - トランジションエフェクト（フェード、スケール、スライドなど）
//! - フィルターエフェクト（ブラー、シャープ、色変換など）
//! - アニメーションエフェクト（弾性、バウンスなど）
//! - カスタムエフェクトのサポート
//! 
//! ## 使用例
//! 
//! ```
//! use crate::core::window_manager::effects_pipeline::EffectsManager;
//! use crate::core::window_manager::effects_pipeline::EffectType;
//! 
//! let mut effects_manager = EffectsManager::new();
//! 
//! // フェードインエフェクトの追加
//! effects_manager.add_effect_from_factory(EffectType::FadeIn, 300, Some(node_id))
//!     .expect("エフェクト追加に失敗");
//! ```

mod effects_manager;
mod transition;
mod filter;
mod animation;
mod render_effects;

pub use effects_manager::{
    EffectsManager,
    EffectType,
    EffectState,
    EasingType,
    SlideDirection,
    TransitionEffect,
};

pub use filter::{
    FilterEffect,
    FilterType,
    BlurParams,
    ColorTransformParams,
};

pub use animation::{
    AnimationEffect,
    AnimationType,
    KeyframeAnimation,
    Keyframe,
};

pub use render_effects::{
    RenderEffect,
    RenderEffectType,
    ShaderEffect,
};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// エフェクトパイプライン
/// 
/// 複数のエフェクトを組み合わせて適用するためのパイプライン
pub struct EffectsPipeline {
    /// エフェクトマネージャー
    effects_manager: Arc<Mutex<EffectsManager>>,
    
    /// パイプラインステージ
    stages: Vec<PipelineStage>,
    
    /// エフェクトプリセット
    presets: HashMap<String, Vec<PipelineStage>>,
    
    /// 有効/無効フラグ
    enabled: bool,
}

/// パイプラインステージ
struct PipelineStage {
    /// ステージ名
    name: String,
    
    /// ステージタイプ
    stage_type: PipelineStageType,
    
    /// 有効/無効フラグ
    enabled: bool,
}

/// パイプラインステージタイプ
enum PipelineStageType {
    /// トランジション
    Transition(EffectType),
    
    /// フィルター
    Filter(FilterType),
    
    /// アニメーション
    Animation(AnimationType),
    
    /// レンダーエフェクト
    Render(RenderEffectType),
    
    /// カスタムエフェクト
    Custom(String),
}

impl EffectsPipeline {
    /// 新しいエフェクトパイプラインを作成
    pub fn new() -> Self {
        Self {
            effects_manager: Arc::new(Mutex::new(EffectsManager::new())),
            stages: Vec::new(),
            presets: HashMap::new(),
            enabled: true,
        }
    }
    
    /// エフェクトマネージャーを取得
    pub fn get_effects_manager(&self) -> Arc<Mutex<EffectsManager>> {
        self.effects_manager.clone()
    }
    
    /// ステージを追加
    pub fn add_stage(&mut self, name: &str, stage_type: PipelineStageType) -> &mut Self {
        self.stages.push(PipelineStage {
            name: name.to_string(),
            stage_type,
            enabled: true,
        });
        
        self
    }
    
    /// プリセットを登録
    pub fn register_preset(&mut self, name: &str, stages: Vec<PipelineStage>) -> &mut Self {
        self.presets.insert(name.to_string(), stages);
        self
    }
    
    /// プリセットを適用
    pub fn apply_preset(&mut self, name: &str) -> Result<(), String> {
        if let Some(stages) = self.presets.get(name) {
            self.stages = stages.clone();
            Ok(())
        } else {
            Err(format!("プリセット '{}' が見つかりません", name))
        }
    }
    
    /// パイプラインを有効化/無効化
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = enabled;
        
        if let Ok(mut manager) = self.effects_manager.lock() {
            manager.set_enabled(enabled);
        }
        
        self
    }
    
    /// パイプラインを更新
    pub fn update(&mut self) {
        if !self.enabled {
            return;
        }
        
        if let Ok(mut manager) = self.effects_manager.lock() {
            manager.update();
        }
    }
    
    /// すべてのエフェクトをクリア
    pub fn clear_all_effects(&mut self) {
        if let Ok(mut manager) = self.effects_manager.lock() {
            manager.cancel_all_effects();
        }
    }
}

/// デフォルトのエフェクトパイプラインを作成
pub fn create_default_pipeline() -> EffectsPipeline {
    let mut pipeline = EffectsPipeline::new();
    
    // デフォルトのステージを追加
    pipeline.add_stage("フェード", PipelineStageType::Transition(EffectType::FadeIn))
            .add_stage("スケール", PipelineStageType::Transition(EffectType::ScaleIn))
            .add_stage("ブラー", PipelineStageType::Filter(FilterType::Blur));
    
    // プリセットを登録
    let minimal_stages = vec![
        PipelineStage {
            name: "フェード".to_string(),
            stage_type: PipelineStageType::Transition(EffectType::FadeIn),
            enabled: true,
        },
    ];
    
    let performance_stages = vec![
        PipelineStage {
            name: "フェード".to_string(),
            stage_type: PipelineStageType::Transition(EffectType::FadeIn),
            enabled: true,
        },
        PipelineStage {
            name: "スケール".to_string(),
            stage_type: PipelineStageType::Transition(EffectType::ScaleIn),
            enabled: true,
        },
    ];
    
    let fancy_stages = vec![
        PipelineStage {
            name: "フェード".to_string(),
            stage_type: PipelineStageType::Transition(EffectType::FadeIn),
            enabled: true,
        },
        PipelineStage {
            name: "スケール".to_string(),
            stage_type: PipelineStageType::Transition(EffectType::ScaleIn),
            enabled: true,
        },
        PipelineStage {
            name: "ブラー".to_string(),
            stage_type: PipelineStageType::Filter(FilterType::Blur),
            enabled: true,
        },
        PipelineStage {
            name: "色変換".to_string(),
            stage_type: PipelineStageType::Filter(FilterType::ColorTransform),
            enabled: true,
        },
    ];
    
    pipeline.register_preset("最小限", minimal_stages)
            .register_preset("パフォーマンス", performance_stages)
            .register_preset("ファンシー", fancy_stages);
    
    pipeline
} 