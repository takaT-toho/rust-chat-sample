# chat-leptos-axum

Axum と Leptos を組み合わせたシンプルなチャットアプリです。SSR + Hydration と WebSocket を同時に利用しています。

この README では WSL (Ubuntu 24.04) 上での構築手順をまとめています。

## 前提環境
- Windows 11 上の WSL2 + Ubuntu 24.04 を想定
- 初期状態の Ubuntu でも実行できるよう、依存パッケージの導入から説明します

## 1. 基本ツールと Rust のセットアップ
```bash
# 必要なビルドツールやユーティリティ
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev curl git

# Rust (rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# ツールチェーンと Wasm ターゲット
rustup update
rustup default stable
rustup target add wasm32-unknown-unknown

# Leptos 開発を補助する CLI
cargo install --locked cargo-leptos
```

> すでに Rust を導入済みでも、`rustup update` と `rustup target add wasm32-unknown-unknown` は実行しておくことを推奨します。

## 2. リポジトリの取得
```bash
cd ~
git clone https://github.com/<your-account>/chat-leptos-axum.git
cd chat-leptos-axum
```

`Cargo.toml` ではサーバー用依存を `optional` とし、`ssr` 機能で一括有効化しています。`cargo check --features ssr` を実行するとサーバー側依存の解決状況を確認できます。

## 3. ビルドと実行
### SSR サーバーのビルド確認
```bash
cargo check --features ssr
```

### Hydration (Wasm) ターゲットの確認
```bash
cargo check --target wasm32-unknown-unknown --features hydrate
```

### 開発サーバー (ホットリロード)
```bash
cargo leptos watch
```
`http://127.0.0.1:3000` をブラウザで開き、複数タブから操作して WebSocket の更新が共有されるか確認します。

### リリースビルド
```bash
cargo leptos build --release
cargo leptos serve --release
```

## 4. トラブルシューティング
- `linker 'cc' not found` → `build-essential` が導入されているか確認
- `wasm32-unknown-unknown not installed` → `rustup target add wasm32-unknown-unknown`
- `cargo clean` がパーミッションエラー → 過去に `sudo` でビルドした場合は `sudo chown -R $(whoami) target` で所有者を戻す

## 5. 参考情報
SSR と Hydration を統合する際の詳細な知見は `docs/ssr-notes.md` にまとめています。Leptos や Axum をアップデートする際は、`AppState` の `FromRef` 実装やフォールバック構成をあらためて確認してください。
