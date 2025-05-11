# LumosDesktop テーマ開発者ガイド

## 目次

1. [はじめに](#はじめに)
2. [テーマの基本構造](#テーマの基本構造)
3. [カラーパレットの設計](#カラーパレットの設計)
4. [フォント設定](#フォント設定)
5. [ウィジェットスタイル](#ウィジェットスタイル)
6. [アニメーション設定](#アニメーション設定)
7. [動的テーマの開発](#動的テーマの開発)
8. [高DPI対応](#高dpi対応)
9. [テーマの検証とテスト](#テーマの検証とテスト)
10. [テーマのパッケージング](#テーマのパッケージング)
11. [マーケットプレイスへの公開](#マーケットプレイスへの公開)
12. [ベストプラクティス](#ベストプラクティス)

## はじめに

このガイドでは、LumosDesktop向けのテーマを開発するための詳細な情報を提供します。テーマ開発者として、ユーザーに美しく一貫性のあるビジュアル体験を提供するためのツールと知識を習得しましょう。

### 必要なスキルと知識

- JSONフォーマットの基本的な理解
- 色彩理論の基礎知識
- UIデザインの基本概念
- テキストエディタの使用経験

### 開発環境のセットアップ

1. **テーマエディタのインストール**:
   ```bash
   sudo apt install lumos-theme-editor
   ```

2. **テーマ検証ツールのインストール**:
   ```bash
   sudo apt install lumos-theme-validator
   ```

3. **開発ディレクトリの作成**:
   ```bash
   mkdir -p ~/.local/share/lumos/themes-dev
   ```

## テーマの基本構造

LumosDesktopのテーマはJSON形式で作成されます。以下は基本的なテーマファイルの構造です。

### 基本的なテーマファイル

```json
{
  "name": "MyTheme",
  "author": "Your Name",
  "description": "A beautiful theme for LumosDesktop",
  "version": "1.0.0",
  "mode": "light",
  "colors": {
    "primary": "#4285f4",
    "secondary": "#34a853",
    "accent": "#fbbc05",
    "background": "#ffffff",
    "foreground": "#212121",
    "success": "#34a853",
    "warning": "#fbbc05",
    "error": "#ea4335",
    "info": "#4285f4"
  },
  "fonts": {
    "family": "Noto Sans",
    "heading_family": "Noto Sans",
    "monospace_family": "Noto Sans Mono",
    "base_size": 14,
    "line_height": 1.5,
    "rendering": {
      "antialias": true,
      "subpixel": true,
      "hinting": "slight"
    }
  },
  "widget_style": {
    "button_radius": 4,
    "input_radius": 4,
    "card_radius": 8,
    "shadow_strength": 3,
    "border_width": 1,
    "focus_ring_width": 2,
    "control_padding": 8
  },
  "animations": {
    "enabled": true,
    "speed_factor": 1.0,
    "transition_ms": 250,
    "easing": "ease_out"
  },
  "display": {
    "hidpi_mode": "auto",
    "scale_factor": 1.0,
    "text_sharpness": 1.0
  },
  "wallpaper": null,
  "custom": {}
}
```

### 必須フィールド

- `name`: テーマの名前
- `mode`: "light", "dark", または "auto"
- `colors`: 基本カラーパレット
- `fonts`: フォント設定
- `widget_style`: ウィジェットの視覚的スタイル
- `animations`: アニメーション設定

### オプションフィールド

- `author`: テーマの作者名
- `description`: テーマの説明
- `version`: テーマのバージョン
- `display`: 高DPI表示設定
- `wallpaper`: デフォルト壁紙へのパス
- `custom`: カスタムプロパティ（拡張用）

## カラーパレットの設計

効果的なテーマは一貫性のあるカラーパレットに基づいています。

### 基本色の定義

- **primary**: メインカラー、ブランドを象徴する色
- **secondary**: サブカラー、プライマリと対比する色
- **accent**: アクセントカラー、特に強調したい要素に使用
- **background**: 背景色
- **foreground**: 前景色（通常はテキスト）

### ステータス色の定義

- **success**: 成功状態を示す色（通常は緑系）
- **warning**: 警告状態を示す色（通常は黄/橙系）
- **error**: エラー状態を示す色（通常は赤系）
- **info**: 情報を示す色（通常は青系）

### 色の表現形式

色は以下の形式で指定できます：

- **HEX**: `"#RRGGBB"` または `"#RRGGBBAA"`（アルファ値付き）
- **RGB**: `"rgb(R, G, B)"`（0-255の整数値）
- **RGBA**: `"rgba(R, G, B, A)"`（Aは0-1の小数）
- **HSL**: `"hsl(H, S%, L%)"`（H: 0-360, S/L: 0-100%）
- **HSLA**: `"hsla(H, S%, L%, A)"`（Aは0-1の小数）

### カラーハーモニーの作成

効果的なテーマを作成するためには、色彩理論に基づいたハーモニーを考慮してください：

1. **補色ハーモニー**: 色相環の反対側にある色を使用
2. **類似色ハーモニー**: 色相環で隣接する色を使用
3. **三色ハーモニー**: 色相環で等間隔に3色を選択
4. **四色ハーモニー**: 色相環で長方形パターンの4色を選択

### アクセシビリティへの配慮

- テキストと背景のコントラスト比は最低4.5:1を確保する
- カラーのみに依存せず、形状や位置でも情報を伝える
- 色覚異常を考慮した色の選択（色覚シミュレーターでテスト）

### 色設計の例（マテリアルデザイン風）

```json
"colors": {
  "primary": "#1976d2",
  "secondary": "#388e3c",
  "accent": "#f57c00",
  "background": "#f5f5f5",
  "foreground": "#212121",
  "success": "#388e3c",
  "warning": "#f57c00", 
  "error": "#d32f2f",
  "info": "#1976d2"
}
```

## フォント設定

フォント設定はテーマの可読性と全体的な印象に大きな影響を与えます。

### 基本設定

```json
"fonts": {
  "family": "Noto Sans",
  "heading_family": "Noto Sans",
  "monospace_family": "Noto Sans Mono",
  "base_size": 14,
  "line_height": 1.5,
  "rendering": {
    "antialias": true,
    "subpixel": true,
    "hinting": "slight"
  }
}
```

### フォントファミリーの選択

- **family**: 標準テキストに使用するフォント
- **heading_family**: 見出しに使用するフォント
- **monospace_family**: 等幅テキスト（コードなど）に使用するフォント

システムにインストールされているフォントまたはLumosDesktopに同梱されているフォントを指定できます。複数のフォントをフォールバックとして指定する場合はカンマ区切りで記述します：

```json
"family": "Noto Sans, Roboto, sans-serif"
```

### サイズと行間

- **base_size**: 基本フォントサイズ（ピクセル単位）
- **line_height**: 行の高さ（フォントサイズに対する倍率）

推奨値：
- デスクトップ用: 14-16px
- 高解像度: 16-18px
- 行の高さ: 1.4-1.6

### レンダリング設定

- **antialias**: アンチエイリアス（文字の輪郭を滑らかに表示）
- **subpixel**: サブピクセルレンダリング（LCD画面での可読性向上）
- **hinting**: ヒンティング強度
  - "none": ヒンティングなし
  - "slight": 軽度のヒンティング
  - "medium": 中程度のヒンティング
  - "full": 完全なヒンティング

## ウィジェットスタイル

ウィジェットスタイルでは、ボタンやカードなどのUI要素の形状と視覚効果を定義します。

### 基本設定

```json
"widget_style": {
  "button_radius": 4,
  "input_radius": 4,
  "card_radius": 8,
  "shadow_strength": 3,
  "border_width": 1,
  "focus_ring_width": 2,
  "control_padding": 8
}
```

### 角丸の設定

- **button_radius**: ボタンの角丸半径（ピクセル単位）
- **input_radius**: 入力フィールドの角丸半径
- **card_radius**: カードの角丸半径

スタイルの一貫性のために、これらの値を関連付けることを検討してください。例えば、`button_radius = input_radius`とし、`card_radius = button_radius * 2`とするなど。

### 影と境界線

- **shadow_strength**: 影の強さ（0-10）
  - 0: 影なし
  - 1-3: 軽い影
  - 4-7: 中程度の影
  - 8-10: 強い影
- **border_width**: 境界線の太さ（ピクセル単位）
- **focus_ring_width**: フォーカス時の強調リングの太さ

### コントロールのパディング

- **control_padding**: コントロール内部のパディング（ピクセル単位）

この値はボタンやフォーム要素など、インタラクティブなコントロールの内部余白を定義します。タッチ操作を考慮する場合は、より大きな値（10-12px以上）を使用してください。

### スタイルの例

#### シャープエッジ（フラットデザイン）
```json
"widget_style": {
  "button_radius": 0,
  "input_radius": 0,
  "card_radius": 0,
  "shadow_strength": 0,
  "border_width": 1,
  "focus_ring_width": 2,
  "control_padding": 8
}
```

#### 丸みを帯びたデザイン（マテリアルデザイン風）
```json
"widget_style": {
  "button_radius": 4,
  "input_radius": 4,
  "card_radius": 8,
  "shadow_strength": 3,
  "border_width": 0,
  "focus_ring_width": 2,
  "control_padding": 12
}
```

#### ニューモーフィズム
```json
"widget_style": {
  "button_radius": 16,
  "input_radius": 16,
  "card_radius": 24,
  "shadow_strength": 5,
  "border_width": 0,
  "focus_ring_width": 3,
  "control_padding": 16
}
```

## アニメーション設定

アニメーションはユーザーエクスペリエンスを向上させる重要な要素です。適切なアニメーションはインターフェースに生命を吹き込みます。

### 基本設定

```json
"animations": {
  "enabled": true,
  "speed_factor": 1.0,
  "transition_ms": 250,
  "easing": "ease_out"
}
```

### アニメーション制御

- **enabled**: アニメーションの有効/無効
- **speed_factor**: アニメーション速度の倍率
  - 1.0: 標準速度
  - 0.5: 50%遅く
  - 2.0: 2倍速く
- **transition_ms**: 標準トランジション時間（ミリ秒）

### イージング関数

- **easing**: アニメーションのイージング関数
  - "linear": 一定速度
  - "ease_in": ゆっくり始まり速くなる
  - "ease_out": 速く始まりゆっくり終わる
  - "ease_in_out": ゆっくり始まり、速くなり、またゆっくり終わる
  - "elastic": バウンド効果のある動き
  - "bounce": バウンスする動き
  - "cubic_bezier(x1, y1, x2, y2)": カスタムベジェ曲線

### アニメーション設計のヒント

- **迅速なフィードバック**: ユーザー操作に対するアニメーションは200ms以下が望ましい
- **自然な動き**: 現実世界の物理法則に沿った動きが直感的
- **一貫性**: システム全体で一貫したアニメーション体験を提供
- **目的を持つ**: 単なる装飾ではなく、状態変化やナビゲーションの理解を助けるアニメーションを設計

## 動的テーマの開発

動的テーマは時間帯、季節、天気などの外部要因に応じて自動的に変化するテーマです。

### 時間帯に基づく変化

```json
"dynamic": {
  "time_based": {
    "enabled": true,
    "transitions": [
      {
        "name": "dawn",
        "start_time": "05:00",
        "end_time": "07:00",
        "color_temperature": 4000,
        "brightness_adjust": 0.9
      },
      {
        "name": "day",
        "start_time": "07:00",
        "end_time": "16:00",
        "color_temperature": 6500,
        "brightness_adjust": 1.0
      },
      {
        "name": "evening",
        "start_time": "16:00",
        "end_time": "19:00",
        "color_temperature": 3800,
        "brightness_adjust": 0.9
      },
      {
        "name": "night",
        "start_time": "19:00",
        "end_time": "05:00",
        "color_temperature": 3200,
        "brightness_adjust": 0.8
      }
    ]
  }
}
```

### 季節に基づく変化

```json
"dynamic": {
  "seasonal": {
    "enabled": true,
    "seasons": [
      {
        "name": "spring",
        "months": [3, 4, 5],
        "hue_shift": 20,
        "saturation_adjust": 1.1,
        "accent_color": "#78C850"
      },
      {
        "name": "summer",
        "months": [6, 7, 8],
        "hue_shift": 0,
        "saturation_adjust": 1.2,
        "accent_color": "#F08030"
      },
      {
        "name": "autumn",
        "months": [9, 10, 11],
        "hue_shift": 30,
        "saturation_adjust": 0.9,
        "accent_color": "#F8D030"
      },
      {
        "name": "winter",
        "months": [12, 1, 2],
        "hue_shift": -20,
        "saturation_adjust": 0.8,
        "accent_color": "#98D8D8"
      }
    ]
  }
}
```

### 天気に基づく変化

```json
"dynamic": {
  "weather_based": {
    "enabled": true,
    "conditions": [
      {
        "name": "clear",
        "brightness_adjust": 1.1,
        "saturation_adjust": 1.1,
        "accent_color": "#FFD700"
      },
      {
        "name": "clouds",
        "brightness_adjust": 0.9,
        "saturation_adjust": 0.8,
        "accent_color": "#A9A9A9"
      },
      {
        "name": "rain",
        "brightness_adjust": 0.8,
        "saturation_adjust": 0.7,
        "accent_color": "#4682B4"
      },
      {
        "name": "snow",
        "brightness_adjust": 1.0,
        "saturation_adjust": 0.5,
        "accent_color": "#E0FFFF"
      },
      {
        "name": "fog",
        "brightness_adjust": 0.7,
        "saturation_adjust": 0.6,
        "accent_color": "#708090"
      }
    ]
  }
}
```

### 動的テーマの組み合わせ

時間帯、季節、天気の動的テーマ効果を組み合わせることができます。LumosDesktopはこれらの効果を適切にブレンドします。

## 高DPI対応

高解像度ディスプレイでのテーマの見え方を制御します。

### 基本設定

```json
"display": {
  "hidpi_mode": "auto",
  "scale_factor": 1.0,
  "text_sharpness": 1.0
}
```

### 高DPIモード

- **hidpi_mode**: 高DPI画面での表示モード
  - "auto": システム設定に従う
  - "native": ネイティブ解像度（シャープだがUI要素が小さい）
  - "scaled": スケーリング適用（UI要素のサイズを保持）

### スケール係数

- **scale_factor**: UI要素のスケール倍率
  - 1.0: 標準サイズ
  - 1.5: 1.5倍のサイズ
  - 2.0: 2倍のサイズ（HiDPIの一般的な値）

### テキストシャープネス

- **text_sharpness**: テキストのシャープネス調整
  - 1.0: 標準シャープネス
  - 0.5: よりソフトなテキスト
  - 1.5: よりシャープなテキスト

### 高DPI対応のヒント

- アイコンは常にSVGまたは複数解像度のビットマップを使用
- 境界線の幅は整数ピクセル値を使用（0.5pxなどの小数値は避ける）
- フォントサイズは十分に大きく設定（14px以上を推奨）
- 異なるスケール係数でテーマをテスト

## テーマの検証とテスト

テーマを公開する前に、品質と互換性のテストを行うことが重要です。

### テーマ検証ツールの使用

テーマ検証ツールを使用して、テーマがガイドラインに準拠しているか確認します：

```bash
lumos-theme-validate mytheme.json
```

検証ツールは以下の項目をチェックします：

- 必須フィールドの存在
- カラーコントラスト比
- アクセシビリティ要件
- レスポンシブデザインの問題
- JSON形式の正当性

### 視覚的テスト

テーマプレビューアプリでテーマを視覚的にテストします：

```bash
lumos-theme-preview mytheme.json
```

以下の項目を確認してください：

- すべてのUIコンポーネントが適切に表示される
- テキストが読みやすい
- インタラクティブ要素が明確に識別できる
- ライト/ダークモードの切り替えが機能する
- アニメーションが滑らかに動作する

### 異なる環境でのテスト

可能な限り多様な環境でテーマをテストしてください：

- 異なるディスプレイ解像度
- 高DPIと標準DPIディスプレイ
- 異なるGPUとレンダリングパイプライン
- 様々なアクセシビリティ設定

## テーマのパッケージング

テーマを配布するための準備を行います。

### ファイル構造

標準的なテーマパッケージは以下のような構造を持ちます：

```
mytheme/
├── mytheme.json      // メインテーマファイル
├── thumbnail.png     // テーマのサムネイル (256x144px推奨)
├── preview.png       // 大きなプレビュー画像 (1280x720px推奨)
├── wallpaper.jpg     // オプションの壁紙
├── LICENSE           // ライセンス情報
└── README.md         // テーマの説明とインストール方法
```

### メタデータの追加

テーマJSONファイルには、検索とフィルタリングを容易にするためのメタデータを含めてください：

```json
"metadata": {
  "tags": ["minimal", "professional", "light"],
  "target_version": "1.5.0",
  "license": "CC-BY-4.0",
  "website": "https://example.com/mytheme"
}
```

### ライセンスの選択

テーマのライセンスを明確に指定してください。一般的なライセンスオプション：

- **CC0**: パブリックドメイン
- **CC-BY**: 帰属表示
- **CC-BY-SA**: 帰属表示-継承
- **MIT**: 緩やかな商用利用許可
- **GPL**: コピーレフトライセンス

### パッケージの作成

テーマパッケージを作成します：

```bash
lumos-theme-package mytheme/
```

これにより、`mytheme.lumostheme`という配布可能なパッケージファイルが生成されます。

## マーケットプレイスへの公開

LumosDesktopマーケットプレイスを通じてテーマを共有します。

### アカウントの作成

1. [LumosDesktop Marketplace](https://marketplace.lumosdesktop.org)にアクセス
2. アカウントを作成またはログイン
3. 開発者プロファイルを設定

### テーマのアップロード

1. 「コンテンツを追加」>「テーマをアップロード」を選択
2. パッケージファイル（.lumostheme）をアップロード
3. テーマの説明、カテゴリ、タグを入力
4. スクリーンショットを追加（少なくとも1枚）
5. 利用規約に同意して「公開」をクリック

### テーマの更新

1. 開発者ダッシュボードから既存のテーマを選択
2. 「更新をアップロード」をクリック
3. 新しいバージョンのパッケージをアップロード
4. 変更内容を記述
5. 「更新を公開」をクリック

## ベストプラクティス

効果的なテーマを作成するためのガイドライン。

### 一貫性を保つ

- 視覚的言語を一貫させる（角丸や影のスタイルなど）
- カラーパレットを限定し、一貫して使用する
- 同様の要素には同様のスタイルを適用する

### アクセシビリティを確保

- WCAGガイドラインに従う（少なくともAA準拠）
- 色だけでなく形状でも情報を伝える
- キーボードフォーカス状態を明確に表示する
- 高コントラストモードをサポートする

### パフォーマンスを考慮

- 複雑なグラデーションやアニメーションの使用を控える
- テーマエンジンのキャッシング機能を活用する
- 重いエフェクトはユーザーが無効化できるようにする

### ユーザーテスト

- 多様なユーザーからフィードバックを収集
- 異なる使用環境でテーマをテスト
- アクセシビリティツールを使用してテーマを評価

### ドキュメント提供

- テーマの特徴と独自性を説明
- カスタマイズ可能なオプションを文書化
- 既知の問題や制限事項を明記

---

このガイドを参考に、LumosDesktopユーザーに喜ばれる素晴らしいテーマを作成してください。さらに詳しい情報やAPIリファレンスについては、[LumosDesktop開発者ドキュメント](https://dev.lumosdesktop.org/theming)をご覧ください。 