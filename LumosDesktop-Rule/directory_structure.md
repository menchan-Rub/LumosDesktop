### LumosDesktop: ディレクトリ構造

```
/LumosDesktop/
├── core/                          # コア機能
│   ├── window_manager/            # ウィンドウマネージャ関連
│   │   ├── compositor/            # コンポジター (Wayland 拡張プロトコル含む)
│   │   ├── scene_graph/           # シーングラフ
│   │   ├── layout_engine/         # ウィンドウレイアウトエンジン (タイリング, フローティング)
│   │   ├── effects_pipeline/      # 視覚エフェクトパイプライン
│   │   ├── input_translator/      # 入力イベント処理/変換
│   │   └── gesture_recognizer/    # ジェスチャー認識エンジン
│   ├── graphics/                  # グラフィックスシステム
│   │   ├── renderer/              # 抽象レンダリングインターフェース
│   │   ├── vulkan_backend/        # Vulkan バックエンド
│   │   ├── metal_backend/         # Metal バックエンド
│   │   ├── dx_backend/            # DirectX バックエンド
│   │   ├── shader_manager/        # シェーダー管理・コンパイル
│   │   └── resource_manager/      # GPU リソース管理
│   ├── settings/                  # 設定マネージャ
│   │   ├── registry/              # 分散設定レジストリ (dconf 代替)
│   │   ├── profile_manager/       # ユーザープロファイル管理
│   │   ├── schema/                # 設定スキーマ定義
│   │   └── sync_agent/            # 設定同期エージェント
│   └── system/                    # システム統合
│       ├── power_interface/       # 電源管理インターフェース (AetherOS 連携)
│       ├── notification_service/  # 通知サービス (D-Bus/独自プロトコル)
│       ├── security_context/      # セキュリティコンテキスト管理
│       ├── hardware_monitor/      # ハードウェア状態監視
│       └── haptics/               # 触覚フィードバック制御
├── ui/                            # ユーザーインターフェース
│   ├── shell/                     # デスクトップシェル
│   │   ├── panel/                 # パネル・タスクバー
│   │   ├── dock/                  # ドック
│   │   ├── dashboard/             # ダッシュボード／ウィジェットエリア
│   │   ├── overlay_manager/       # オーバーレイ UI 管理 (通知, OSD)
│   │   ├── lock_screen/           # ロック画面
│   │   └── login_manager/         # ログインマネージャ (Greeter)
│   ├── toolkit/                   # 独自 UI ツールキット (Lumos UI)
│   │   ├── core/                  # ウィジェット基本クラス, イベントループ
│   │   ├── controls/              # 標準コントロールウィジェット
│   │   ├── containers/            # レイアウトコンテナ
│   │   ├── dialogs/               # 標準ダイアログ
│   │   ├── styling/               # CSS ベーススタイリングエンジン
│   │   └── accessibility/         # アクセシビリティ実装 (ATK/UIAutomation 相当)
│   ├── theming/                   # テーマシステム
│   │   ├── engine/                # テーマエンジンコア
│   │   ├── themes/                # 標準テーマ (Light, Dark, Adaptive)
│   │   ├── icons/                 # アイコン管理・キャッシュ
│   │   ├── cursors/               # カーソルテーマ管理
│   │   └── font_manager/          # フォントレンダリング・管理
│   ├── animations/                # アニメーションエンジン
│   │   ├── engine/                # 物理ベースアニメーションコア
│   │   ├── transitions/           # 標準トランジション効果
│   │   └── effects_library/       # エフェクトライブラリ
│   ├── workspaces/                # 仮想デスクトップ／空間 UI
│   │   ├── manager/               # ワークスペース管理ロジック
│   │   ├── spatial_navigator/     # 空間ナビゲーション UI (VR/AR 対応)
│   │   └── context_organizer/     # コンテキストベース自動整理
│   └── i18n/                      # 国際化サポート
│       ├── locale_manager/        # ロケール管理
│       ├── translation_service/   # 翻訳サービス (gettext 互換)
│       └── input_methods/         # 入力メソッドフレームワーク (IBus/Fcitx)
├── apps/                          # 標準デスクトップアプリケーション
│   ├── launcher/                  # アプリケーションランチャー
│   │   ├── search/                # 検索エンジン
│   │   ├── categories/            # カテゴリ分類
│   │   └── voice/                 # 音声起動機能
│   ├── file_manager/              # ファイル管理ツール
│   │   ├── browser/               # ファイルブラウザ
│   │   ├── transfer/              # ファイル転送マネージャ
│   │   ├── search/                # 高度な検索
│   │   └── preview/               # プレビュー機能
│   ├── settings_center/           # 設定センター
│   │   ├── panels/                # 設定パネル群
│   │   ├── profiles/              # プロファイル管理
│   │   └── wizard/                # ウィザード形式設定
│   └── system_monitor/            # システムモニター
│       ├── resources/             # リソースモニタリング
│       ├── processes/             # プロセス管理
│       └── diagnostics/           # 診断ツール
├── services/                      # バックグラウンドサービス
│   ├── session_manager/           # ユーザーセッション管理
│   ├── sync_service/              # 設定・データ同期サービス
│   ├── eco_mode_service/          # 省エネモード管理
│   ├── accessibility_manager/     # アクセシビリティサービス管理
│   ├── clipboard_manager/         # クリップボード履歴管理
│   └── context_awareness/         # コンテキスト認識エンジン
├── integration/                   # 外部統合レイヤ
│   ├── nexus_bridge/              # NexusShell 連携ブリッジ
│   ├── compat/                    # X11 / Wayland / Win32 互換レイヤ
│   ├── device_portal/             # XDG Desktop Portal 相当
│   ├── aether_services/           # AetherOS システムサービス連携
│   └── cloud_providers/           # クラウドプロバイダ統合
├── plugins/                       # プラグインシステム
│   ├── framework/                 # WASM ベースプラグインフレームワーク
│   ├── extensions/                # 標準拡張プラグイン群
│   ├── plugin_manager/            # プラグイン管理 UI / CLI
│   └── examples/                  # サンプルプラグイン
├── tools/                         # 開発・管理用ツール
│   ├── ui_designer/               # UI デザイナー
│   ├── theme_editor/              # テーマエディタ
│   ├── performance_analyzer/      # パフォーマンス分析ツール
│   └── debug_console/             # デバッグコンソール
├── docs/                          # ドキュメント
│   ├── user/                      # ユーザーマニュアル
│   ├── developer/                 # 開発者向けドキュメント
│   ├── api/                       # API 参照
│   └── ux_guidelines/             # UX ガイドライン
├── tests/                         # テスト
│   ├── unit/                      # ユニットテスト
│   ├── integration/               # 統合テスト
│   ├── ui/                        # UI テスト
│   ├── performance/               # パフォーマンステスト
│   └── accessibility/             # アクセシビリティテスト
├── build/                         # ビルドシステム設定
│   ├── CMakeLists.txt             # CMake ビルド定義
│   ├── WORKSPACE                  # Bazel ワークスペース定義
│   └── .bazelrc                   # Bazel 設定
├── Cargo.toml                     # Rust プロジェクト定義
├── CMakeLists.txt                 # ルート CMake 定義
├── WORKSPACE                      # Bazel ルート定義
├── .gitignore                     # Git 無視設定
├── README.md                      # プロジェクト概要
├── LICENSE_MIT                    # MIT ライセンス
├── LICENSE_APACHE                 # Apache ライセンス
└── CONTRIBUTING.md                # 貢献ガイドライン
```

#### 主要ディレクトリの説明 (更新)

1. **core/**:
   - **window_manager**: シーングラフ、レイアウトエンジン、入力変換などを詳細化。
   - **graphics**: 各グラフィックAPIバックエンド、シェーダー/リソース管理を明記。
   - **settings**: 分散レジストリ、スキーマ、同期エージェントを追加。
   - **system**: 各システム連携インターフェース、触覚フィードバック制御を追加。

2. **ui/**:
   - **shell**: ロック画面、ログインマネージャを追加。
   - **toolkit**: Lumos UIツールキットとして明確化、スタイリング、アクセシビリティ実装を追加。
   - **theming**: テーマエンジン、フォント管理などを詳細化。
   - **animations**: 物理ベースアニメーションエンジン、エフェクトライブラリ。
   - **workspaces**: 空間ナビゲーション、コンテキスト整理機能を追加。
   - **i18n**: ロケール管理、翻訳サービス、入力メソッドフレームワークを追加。

3. **apps/**: (変更なし、個別アプリの構造は省略)

4. **services/**:
   - 各サービスをより具体的に記述。
   - 高度クリップボードサービス、コンテキスト認識エンジンを追加。

5. **integration/**:
   - NexusShell連携を明確化。
   - デバイスアクセスポータル (XDG Desktop Portal相当) を追加。
   - クラウドプロバイダ統合を追加。

6. **plugins/**:
   - WASMベースフレームワークを明記。
   - プラグイン管理ツールを追加。

7. **ai/**: 新規追加。デスクトップ固有のAI機能 (予測、要約、パーソナライズ) を格納。

8. **tools/**:
   - UIデザイナー、テーマエディタなどを具体化。

9. **docs/**: UXガイドラインを追加。

10. **tests/**: アクセシビリティテストを明記。

11. **build/**: ビルドシステム設定を追加。

12. **Cargo.toml**: Rust プロジェクト定義を追加。

13. **CMakeLists.txt**: ルート CMake 定義を追加。

14. **WORKSPACE**: Bazel ルート定義を追加。

15. **.gitignore**: Git 無視設定を追加。

16. **README.md**: プロジェクト概要を追加。

17. **LICENSE_MIT**: MIT ライセンスを追加。

18. **LICENSE_APACHE**: Apache ライセンスを追加。

19. **CONTRIBUTING.md**: 貢献ガイドラインを追加。
