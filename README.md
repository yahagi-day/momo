# momo - 映像スプリッター/ルーター

1系統の映像入力（DeckLink / UVC）を分割・変換して、複数のDeckLinkから同時出力するライブ用途のソフトウェア。

## システム構成

```
[DeckLink/UVC 入力 1系統]
    │
    ▼
[映像処理エンジン (GPU: CUDA)]
    │  クロップ → スケール → 反転
    │
    ├→ [DeckLink出力1] 個別設定
    ├→ [DeckLink出力2] 個別設定
    ├→ ...N台          個別設定
    │
    ├→ [Webプレビュー配信]
    │
[Web UI (ブラウザ)]
    │  設定・操作・プレビュー
    │
[JSON設定ファイル]
    │  永続化・起動時復元
```

## ビルド・実行

```bash
cargo build
cargo run
cargo test
cargo clippy
```

## プロジェクト構成

```
momo/
├── Cargo.toml                  # ワークスペースルート
├── crates/
│   ├── momo-core/              # 共有型、設定、エラー型
│   ├── momo-decklink/          # DeckLink C++ FFI (cxx crate)
│   │   └── cpp/                # C++ブリッジコード
│   ├── momo-uvc/               # UVC (v4l2 / MediaFoundation)
│   ├── momo-gpu/               # CUDAカーネル + GPUメモリ管理
│   │   └── kernels/            # crop.cu, scale.cu, flip.cu
│   ├── momo-pipeline/          # フレームルーティング: input → GPU → outputs
│   ├── momo-web/               # axum + REST API + WebSocket + MJPEGプレビュー
│   └── momo-app/               # バイナリ、全体の結合
└── frontend/                   # SolidJS + Vite
```

### 各クレートの役割

| クレート | 役割 |
|---|---|
| `momo-core` | `Frame`, `Config`, `Error` 等の共有型。JSON設定のserde処理・バリデーション |
| `momo-decklink` | DeckLink SDK FFI。`VideoInput`/`VideoOutput`トレイト、デバイス列挙 |
| `momo-uvc` | UVCカメラ入力（Linux: v4l2、Windows: MediaFoundation） |
| `momo-gpu` | CUDAコンテキスト管理、crop/scale/flipカーネル（PTXロード） |
| `momo-pipeline` | 入力→GPU→N出力のフレームルーティング管理 |
| `momo-web` | axum REST API + WebSocket + MJPEGプレビュー配信 |
| `momo-app` | CLIバイナリ。clap引数解析、全クレートの結合・起動 |

## 入力仕様

- 入力ソース: DeckLink または UVC（切替可能）
- 同時入力: 1系統
- 解像度・フレームレート: 可変（自動検出）

## 出力仕様

各DeckLink出力ごとに以下を個別設定可能:

- **フル表示**: 入力映像をそのまま出力
- **クロップ**: ピクセル座標指定（x, y, width, height）
- **スケーリング**: クロップ後に出力フォーマットに合わせてリサイズ
- **反転**: 水平 / 垂直
- **出力フォーマット**: 解像度・FPSを出力ごとに個別設定
- **出力数**: スケーラブル（5系統以上想定）

## 映像変換パイプライン

```
入力(CPU) → GPU upload → [出力1: crop→scale→flip (CUDA stream 1)] → D2H → DeckLink出力1
                        → [出力2: crop→scale→flip (CUDA stream 2)] → D2H → DeckLink出力2
                        → [出力N: ...]                              → ...
                        → [Preview: downscale→JPEG]                 → MJPEG配信
```

- GPU処理（CUDA / GTX1080以上）
- 全出力が同一GPUソースバッファを共有し、並列CUDAストリームで処理
- 将来的に回転（90° / 180° / 270°）追加予定

## API

```
GET    /api/devices              デバイス一覧+状態
GET    /api/config               現在の設定
PUT    /api/config               設定全体の適用
PATCH  /api/config/output/:id    単一出力の設定変更
POST   /api/config/save          JSONに永続化
POST   /api/config/load          JSONから復元
GET    /api/status               パイプライン状態
POST   /api/pipeline/start       開始
POST   /api/pipeline/stop        停止
GET    /api/preview/input        入力MJPEGストリーム
GET    /api/preview/output/:id   出力MJPEGストリーム
WS     /ws/status                WebSocket: デバイスイベント、FPS
```

## Web UI

- **設定画面**: 入力ソース選択、各出力のパラメータ設定
- **プレビュー**: 入力映像 + 各出力映像をブラウザで確認
- **適用ボタン**: パラメータ変更は明示的に適用
- **ステータス表示**: 各デバイスの状態監視

## 設定永続化

- JSON形式で保存・読み込み
- 起動時に前回設定を復元

## エラーハンドリング

- DeckLink切断時にクラッシュしない（graceful degradation）
- デバイス状態をWeb UIに反映

## 対応OS

- Ubuntu
- Windows

## 技術スタック

| 要素 | 技術 | 理由 |
|---|---|---|
| 言語 | Rust | 安全性・パフォーマンス |
| DeckLink FFI | `cxx` crate | COM-like APIにbindgenは不向き。C++ブリッジで安全にフラット化 |
| GPU | `cudarc` + nvcc PTX | Rustから安全なCUDA操作。カーネルはPTXとしてロード |
| Web | axum + SolidJS | axumはtokioネイティブ。SolidJSはリアルタイムUIに最適 |
| プレビュー | MJPEG over HTTP | 実装がシンプル。将来WebRTC対応可 |
| スレッド間通信 | crossbeam-channel | 映像スレッドとtokioの橋渡し |
| 設定保存 | serde_json | |
