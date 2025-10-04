# SSR統合で得た知見

## 背景
- Leptos + Axum 構成のチャットアプリで SSR ビルドが失敗していた。
- `axum` や `tokio` などの依存が解決されず、ランタイム初期化やフォールバック処理が旧 API のままになっていた。
- Rust 1.80.1 では依存クレート (特に Leptos が内部で利用する ICU 系) のビルド要件を満たせず、ツールチェーン更新が必須だった。

## Cargo 設定で得た学び
- サーバー専用の依存 (axum / tokio / tower-http / leptos_axum) は `optional = true` にし、`ssr` 機能で `"dep:xxx"` を指定すると条件付きビルドが明確になる。
- Leptos 系クレートは `default-features = false` で追加し、`ssr` と `hydrate` で必要な機能だけを有効化することで、サーバー・クライアント双方のバンドルを最小化できる。
- `[[bin]]` の `required-features` に `ssr` を指定しておくと、間違ってサーバー機能を無効にした状態でのビルドを防げる。

## サーバー構成のポイント
- Axum 0.7 では `axum::Server` ではなく `TcpListener` + `axum::serve` を使う。API 変更に追随することで将来のメンテナンス負荷を減らせる。
- Leptos が要求する `LeptosOptions: FromRef<State>` の制約を満たすため、`AppState` に `LeptosOptions` を保持し `FromRef` を実装する必要がある。同様に WebSocket 用 `Sender` も `FromRef` にしておくと、`provide_context` で再利用しやすい。
- `render_app_to_stream_with_context` をフォールバックとして組み込み、`ServeDir::not_found_service` と併用することで静的ファイルと SSR レスポンスをシンプルに切り替えられる。
- WebSocket の同時送受信は `tokio::select!` で 1 ループにまとめると、所有権エラーを避けつつクリーンに記述できる。

## ビルド環境と運用上の注意
- Rust 1.82 以上 (今回 1.90.0) が必要。`rustup update stable` で更新し、Windows 環境では MSVC の `link.exe` もしくは GNU ツールチェーンを用意する。
- 以前に管理者権限でビルドしていた場合、`target` ディレクトリの所有権が変わり `cargo clean` が失敗することがある。`sudo chown -R $(whoami) target` などで権限を戻す。

## 今後のチェックリスト
1. `rustup show` でツールチェーンを確認し、必要なら Build Tools も更新する。
2. `cargo check --features ssr` と `cargo check --target wasm32-unknown-unknown --features hydrate` を継続的に回してサーバー・クライアント双方をカバーする。
3. 依存更新時は `Cargo.toml` の optional 設定と `FromRef` 実装の制約が変わっていないかをレビューする。
