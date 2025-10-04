#[cfg(feature = "ssr")]
mod server; // サーバ実装（Axum + SSR）は feature "ssr" のときだけ有効

#[cfg(not(feature = "ssr"))]
fn main() {} // SSR でないビルド時は空 main（ビルドエラー回避）

#[cfg(feature = "ssr")]
#[tokio::main] // 非同期ランタイムエントリポイント
async fn main() -> anyhow::Result<()> {
    // エラー型に anyhow を使用
    server::run().await // サーバ起動
}
