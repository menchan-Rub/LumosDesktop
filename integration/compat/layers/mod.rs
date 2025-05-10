// 互換性レイヤーモジュール
// 各バージョンのAetherOSとの互換性レイヤーを提供します

pub mod aetheros_1_0;
pub mod aetheros_1_5;
pub mod aetheros_2_0;
pub mod aetheros_2_5;

// 各バージョンの互換性レイヤーを再エクスポート
pub use aetheros_1_0::AetherOS1_0Layer;
pub use aetheros_1_5::AetherOS1_5Layer;
pub use aetheros_2_0::AetherOS2_0Layer;
pub use aetheros_2_5::AetherOS2_5Layer;

/// すべてのデフォルト互換性レイヤーを作成して返します
pub fn create_default_layers() -> Vec<Box<dyn crate::integration::compat::CompatibilityLayer>> {
    vec![
        Box::new(AetherOS1_0Layer::new()),
        Box::new(AetherOS1_5Layer::new()),
        Box::new(AetherOS2_0Layer::new()),
        Box::new(AetherOS2_5Layer::new()),
    ]
} 