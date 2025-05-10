# ハプティクス（触覚フィードバック）システム

このモジュールは、LumosDesktopおよびAetherOSで利用可能なハプティクスフィードバックシステムの実装です。多様なデバイスと複数のプラットフォーム（Linux、Windows、macOS）をサポートしています。

## 機能

- 複数のプラットフォームをサポート（Linux、Windows、macOS）
- 多様なデバイスタイプをサポート：
  - 内蔵振動モーター
  - タッチパッド（Linuxタッチパッド、Windows Precision Touchpad、MacBook Force Touch）
  - タッチスクリーン
  - 外部コントローラー（ゲームパッド、XInput互換デバイスなど）
- 様々なフィードバックパターン：
  - クリック/タップ
  - ダブルクリック
  - 長押し
  - 成功/エラー/警告通知
  - カスタムパターン
- 複数の強度レベル（なし、非常に弱い、弱い、中程度、強い、非常に強い）
- イベントキューとプロセッサスレッドによる効率的な処理
- プラットフォーム固有の実装を簡単に拡張可能

## 構造

```
core/system/haptics/
├── mod.rs                 - メインモジュールと共通機能
├── linux.rs               - Linux固有の実装
├── windows.rs             - Windows固有の実装
├── macos.rs               - macOS固有の実装
└── README.md              - このファイル
```

## AetherOSとの統合方法

AetherOSにこのハプティクス機能を統合するには、以下の手順に従ってください：

1. **依存関係の追加**：
   - Cargo.tomlに必要な依存関係を追加：
   ```toml
   [dependencies]
   # 基本的な依存関係
   log = "0.4"
   
   # プラットフォーム固有の依存関係（必要に応じて）
   [target.'cfg(windows)'.dependencies]
   winapi = { version = "0.3", features = ["xinput"] }
   
   [target.'cfg(target_os = "macos")'.dependencies]
   objc = "0.2"
   
   [target.'cfg(unix)'.dependencies]
   glob = "0.3"
   ```

2. **モジュールの組み込み**：
   - AetherOSのシステムモジュールにハプティクスモジュールを追加：
   ```rust
   // aetheros/src/system/mod.rs
   pub mod haptics;
   ```

3. **システム起動時の初期化**：
   - システム起動時にハプティクスサブシステムを初期化：
   ```rust
   // aetheros/src/system/init.rs
   use crate::system::haptics::HapticFeedback;
   
   pub fn initialize_system() -> Result<(), Error> {
       // 他のサブシステム初期化...
       
       // ハプティクスシステム初期化
       let mut haptic_feedback = HapticFeedback::new_default();
       haptic_feedback.initialize()?;
       
       // システムコンテキストに追加
       system_context.add_subsystem("haptics", Box::new(haptic_feedback));
       
       Ok(())
   }
   ```

4. **ユーザーインターフェース統合**：
   - UI要素にハプティクスフィードバックを追加：
   ```rust
   // 例：ボタンクリック時のハプティクスフィードバック
   fn on_button_click(&self) {
       if let Some(haptics) = system_context.get_subsystem::<HapticFeedback>("haptics") {
           let _ = haptics.play_pattern(HapticPattern::Click);
       }
   }
   ```

5. **設定インターフェース追加**：
   - ユーザーがハプティクスの有効/無効や強度を設定できるUIの追加：
   ```rust
   // 例：ハプティクス設定の更新
   fn update_haptic_settings(&self, enabled: bool, intensity: HapticIntensity) {
       if let Some(haptics) = system_context.get_subsystem::<HapticFeedback>("haptics") {
           let _ = haptics.set_enabled(enabled);
           let _ = haptics.set_default_intensity(intensity);
       }
   }
   ```

## モバイルプラットフォーム統合

AetherOSがモバイルプラットフォーム（Android、iOS）をサポートする場合は、それぞれのプラットフォームに対応する実装を追加できます：

1. **Android統合**：
   - `android.rs` モジュールを作成し、Android Vibratorを使用した実装を提供
   - JNIを使用してJavaのVibratorAPIを呼び出す

2. **iOS統合**：
   - `ios.rs` モジュールを作成し、UIFeedbackGeneratorを使用した実装を提供
   - Objective-C/Swiftブリッジを使用してiOSのハプティクスAPIを呼び出す

## 使用例

基本的な使用例：

```rust
use aetheros::system::haptics::{HapticFeedback, HapticPattern, HapticIntensity};

// デフォルト設定でインスタンスを作成
let mut haptic_feedback = HapticFeedback::new_default();

// ハプティクスシステムを初期化
haptic_feedback.initialize().expect("Failed to initialize haptics");

// 単純なクリックフィードバックを再生
haptic_feedback.play_pattern(HapticPattern::Click).expect("Failed to play haptic");

// カスタム強度でフィードバックを再生
haptic_feedback.play_with_intensity(HapticPattern::Success, HapticIntensity::Strong)
    .expect("Failed to play haptic");

// カスタム持続時間でフィードバックを再生
use std::time::Duration;
haptic_feedback.play_with_duration(
    HapticPattern::LongPress, 
    Duration::from_millis(300)
).expect("Failed to play haptic");

// 利用可能なデバイスを表示
let devices = haptic_feedback.get_devices().expect("Failed to get devices");
for device in devices {
    println!("Device: {} ({})", device.name, device.id);
}

// システム終了時にシャットダウン
haptic_feedback.shutdown().expect("Failed to shutdown haptics");
```

## カスタマイズと拡張

システムは容易に拡張可能で、新しいデバイスタイプやプラットフォームの追加が可能です：

1. 新しいデバイスタイプの追加：
   - `HapticDeviceType` enumに新しいバリアントを追加
   - 検出関数と再生関数を実装

2. 新しいプラットフォームの追加：
   - 新しいプラットフォーム固有のファイル（例：`android.rs`）を作成
   - 検出、再生、停止関数を実装
   - `mod.rs`に条件付きコンパイル命令を追加

## ライセンス

このコードはAetherOSのライセンスに従って提供されます。 