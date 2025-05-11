# LumosDesktop テーマシステム

LumosDesktopの高度なテーマシステムは、GNOMEを超える柔軟性と拡張性を提供します。このシステムはリアルタイムの視覚効果、動的なテーマ変更、時間帯や季節に合わせた自動調整などの機能を備えています。

## 主な機能

- **高度なカラーマネジメント**: HSL色空間でのカラー操作、色温度の調整
- **アニメーションとエフェクト**: スムーズなトランジション、フェード、ブラー、スライドなどの視覚効果
- **時間ベースの動的テーマ**: 時間帯、季節、天気に合わせて自動的に変化
- **高DPIサポート**: 高解像度ディスプレイでの最適な表示
- **パフォーマンス最適化**: ベンチマークテスト済みの高速な実装

## 使用例

### 基本的なテーマの使用

```rust
// テーマエンジンのインスタンスを作成
let engine = ThemeEngine::new();

// テーマディレクトリを追加
engine.add_theme_path("/usr/share/lumos/themes");
engine.add_theme_path("~/.config/lumos/themes");

// 標準テーマを読み込んでインストール
let dark_theme = engine.load_theme("/usr/share/lumos/themes/dark.json").unwrap();
engine.install_theme(dark_theme);

// テーマを適用
engine.set_theme_by_name("Dark").unwrap();
```

### テーマ切り替えエフェクト

```rust
// フェード効果でテーマを切り替え
engine.set_theme_with_blend("Light", "fade").unwrap();

// スライド効果でテーマを切り替え
engine.set_theme_with_blend("Dark", "slide_left").unwrap();
```

### 動的テーマの有効化

```rust
// 動的テーマを有効化
engine.enable_dynamic_theme(None); // デフォルト設定を使用

// カスタム設定で動的テーマを有効化
let mut settings = DynamicThemeSettings::default();
settings.update_interval_sec = 300; // 5分ごとに更新
engine.enable_dynamic_theme(Some(settings));
```

### カスタムエフェクトの適用

```rust
// ボタンにフェードイン効果を適用
engine.with_effect_manager(|manager| {
    let mut settings = EffectSettings::default();
    settings.duration_ms = 500;
    settings.easing = EasingFunction::EaseOutCubic;
    
    manager.apply_effect("button_1", EffectType::FadeIn, Some(settings));
});
```

## テーマファイル形式

テーマはJSON形式で定義され、以下の主要セクションを含みます：

- **基本情報**: 名前、作者、バージョン
- **カラーパレット**: プライマリ、セカンダリ、アクセント、その他のカラー
- **フォント設定**: フォントファミリー、サイズ、レンダリング設定
- **ウィジェットスタイル**: ボタンの丸み、影の強さなど
- **アニメーション設定**: 速度、イージング関数
- **ディスプレイ設定**: 高DPI対応、スケーリング

```json
{
  "name": "LumosDark",
  "author": "Lumos Team",
  "version": "1.0.0",
  "mode": "Dark",
  "colors": {
    "primary": "#3f51b5",
    "secondary": "#7986cb",
    "accent": "#ff4081",
    "background": "#121212",
    "foreground": "#ffffff"
  },
  "fonts": {
    "family": "Noto Sans",
    "base_size": 14
  }
}
```

## 特殊効果について

LumosDesktopのテーマエンジンは、以下のような特殊効果をサポートしています：

- **テーマブレンド**: 2つのテーマ間をスムーズに遷移
- **カラーシフト**: 時間帯に応じた色温度と色味の変化
- **パーティクルエフェクト**: インタラクション時の視覚的なフィードバック
- **リフレクション**: 光沢効果と反射

## パフォーマンスについて

テーマエンジンは高度な最適化を行っており、低リソース環境でも高いパフォーマンスを発揮します。ベンチマークテストは `benches/theme_performance.rs` で確認できます。

```
cargo bench
```

## 貢献方法

新しいテーマやエフェクトの開発に貢献するには、標準的なテーマテンプレートを使用し、テストを追加してからプルリクエストを送信してください。すべてのテーマコンポーネントは単体テストでカバーされる必要があります。 