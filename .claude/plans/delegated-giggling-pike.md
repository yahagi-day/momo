# Pipeline Start エラー復帰 + デバイス列挙UI改善

## Context

3つの問題がある:
1. **Pipeline::start()でエラー時にstateがStartingのまま戻らない** — `InputDriver::from_config()`が失敗すると状態がStoppedに復帰せず、UIからStart/Stopどちらも操作不能になる
2. **UVCデバイスが手動パス入力のみ** — 利用可能なUVCデバイスを列挙して選択できるようにしたい。更新ボタンも必要
3. **DeckLinkもデバイスが接続されてない場合の挙動確認** — `/api/devices`はDeckLinkのみ列挙。UVCは含まれていない

---

## 変更ファイル一覧

| ファイル | 操作 | 内容 |
|---|---|---|
| `crates/momo-pipeline/src/pipeline.rs` | 変更 | start()のエラー時にstate→Stoppedに復帰 |
| `crates/momo-web/src/handlers/devices.rs` | 変更 | UVCデバイス列挙を追加、DeviceInfo構造体拡張 |
| `crates/momo-web/Cargo.toml` | 変更 | momo-uvc依存追加 |
| `frontend/src/api/client.ts` | 変更 | getDevices()関数追加 |
| `frontend/src/api/types.ts` | 変更 | DeviceInfo型追加 |
| `frontend/src/components/InputPanel.tsx` | 変更 | UVCデバイス選択ドロップダウン+更新ボタン、DeckLinkデバイス選択ドロップダウン+更新ボタン |

---

## Step 1: Pipeline start()エラー復帰

`crates/momo-pipeline/src/pipeline.rs` の `start()` メソッド (L85-88):

```rust
// 現状: state = Starting → from_config()失敗 → state = Starting のまま
self.state = PipelineState::Starting;
self.emit(PipelineEvent::StateChanged { state: self.state });
let driver = InputDriver::from_config(&config.input)?;  // ← ここで失敗すると復帰しない
```

修正: `from_config()`失敗時に`state`を`Stopped`に戻してイベント発火してからエラー返却:

```rust
self.state = PipelineState::Starting;
self.emit(PipelineEvent::StateChanged { state: self.state });

let driver = match InputDriver::from_config(&config.input) {
    Ok(d) => d,
    Err(e) => {
        self.state = PipelineState::Stopped;
        self.emit(PipelineEvent::StateChanged { state: self.state });
        return Err(e);
    }
};
```

既存テスト `start_without_config_fails` はconfig無しエラー（Starting前に返る）なので影響なし。

---

## Step 2: デバイス列挙API拡張

### `crates/momo-web/Cargo.toml`
`momo-uvc` 依存を追加:
```toml
momo-uvc = { workspace = true }
```

### `crates/momo-web/src/handlers/devices.rs`

DeviceInfoに`device_type`フィールドを追加し、UVCデバイスも返すように拡張:

```rust
#[derive(Serialize)]
pub struct DeviceInfo {
    pub device_type: String,  // "decklink" or "uvc"
    pub index: u32,
    pub name: String,
    pub model_name: String,
    pub has_input: bool,
    pub has_output: bool,
    pub status: String,
}

pub async fn get_devices() -> Json<Vec<DeviceInfo>> {
    let mut infos = Vec::new();

    // DeckLink devices
    for d in momo_decklink::enumerate_devices() {
        infos.push(DeviceInfo {
            device_type: "decklink".into(),
            index: d.index,
            name: d.name,
            model_name: d.model_name,
            has_input: d.has_input,
            has_output: d.has_output,
            status: format!("{:?}", d.status),
        });
    }

    // UVC devices
    for (i, desc) in momo_uvc::enumerate_devices().into_iter().enumerate() {
        infos.push(DeviceInfo {
            device_type: "uvc".into(),
            index: i as u32,
            name: desc.clone(),
            model_name: desc,
            has_input: true,
            has_output: false,
            status: "Available".into(),
        });
    }

    Json(infos)
}
```

UVCの`enumerate_devices()`は `"0: HD Webcam"` 形式の文字列を返す。indexとデバイス名をパースしてUIに渡す。フィーチャー無効時は空配列（既存動作と同じ）。

---

## Step 3: フロントエンド型・APIクライアント追加

### `frontend/src/api/types.ts`
```typescript
export interface DeviceInfo {
  device_type: string;
  index: number;
  name: string;
  model_name: string;
  has_input: boolean;
  has_output: boolean;
  status: string;
}
```

### `frontend/src/api/client.ts`
```typescript
export async function getDevices(): Promise<DeviceInfo[]> {
  return request('/api/devices');
}
```

---

## Step 4: InputPanel UIのデバイス選択改善

### UVCセクション
- テキスト入力 → ドロップダウン（`<select>`）+ 手動入力フォールバック
- 「更新」ボタンで `getDevices()` を呼び出してUVCデバイス一覧を再取得
- デバイスが見つからない場合は「デバイスが見つかりません」表示 + 手動パス入力を残す

### DeckLinkセクション
- device_indexの数値入力 → ドロップダウン（利用可能デバイスから選択）
- 「更新」ボタンで再取得
- デバイスが見つからない場合は「DeckLinkデバイスが見つかりません」表示

### 実装イメージ（InputPanel.tsx）
- `devices` signal (`DeviceInfo[]`)を追加
- `fetchDevices()` — `getDevices()`を呼んでdevicesを更新
- editing開始時 + 更新ボタンクリック時にfetchDevices()
- UVC: `devices().filter(d => d.device_type === 'uvc')` でフィルタ → `<select>`
- DeckLink: `devices().filter(d => d.device_type === 'decklink' && d.has_input)` → `<select>`

UVCのdevice_pathの取得方法: `enumerate_devices()` が返す文字列は `"0: HD Webcam"` 形式。Linux上では `/dev/video{index}` がパスになるため、indexから `device_path` を構築する。ただし、バックエンドから直接パスを返すのがベター。

→ UVCの`enumerate_devices()`を拡張して構造化データを返す案もあるが、既存の`Vec<String>`をパースするシンプルな方法で進める。フロントエンドでindex部分をパースして `/dev/video{index}` を構築。

---

## 検証

1. `cd frontend && npm run build` — ビルド成功
2. `cargo build` — ビルド成功
3. `cargo test` — 全49テスト+αパス
4. `cargo clippy -- -D warnings` — 警告なし
5. 手動確認:
   - UVC/DeckLinkデバイスなしでStartした場合 → エラー表示後Stoppedに戻ること
   - デバイス更新ボタンで一覧取得されること（featureなしなら空配列）
   - Mock選択時は従来通り動作すること
