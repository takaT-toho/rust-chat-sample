use leptos::*; // Leptos のコア API（Signal/コンポーネント等）
use leptos_meta::*; // <Title> や <Stylesheet> などのメタ情報
use leptos_router::*; // ルーティング（本サンプルでは 1 ルートのみ）

#[component] // Leptos のコンポーネント定義マクロ
pub fn App() -> impl IntoView {
    // ルートコンポーネント（SSR/Hydration 両対応）
    provide_meta_context(); // Meta 情報を子コンポーネントへ供給

    view! { // JSX ライクなテンプレート
        <Title text="Rust Chat (Axum + Leptos)"/> // ページタイトル
        <Router> // （必要最低限の）ルータ
            <main class="mx-auto max-w-xl p-4"> // シンプルなスタイル（Tailwind 前提風のクラス名）
                <h1 class="text-2xl font-bold mb-2">"Rust Chat"</h1> // 見出し
                <p class="text-sm text-gray-600 mb-4"> // 説明テキスト
                    "Axum + Leptos (SSR & Hydration) + WebSocket"
                </p>
                <ChatRoom/> // チャット UI 本体
                <footer class="text-xs text-gray-500 mt-6"> // フッタ
                    "Built with Leptos SSR + Hydration"
                </footer>
            </main>
        </Router>
    }
}

#[component] // チャット UI コンポーネント
fn ChatRoom() -> impl IntoView {
    // 入出力を管理
    let (messages, set_messages) = create_signal::<Vec<String>>(vec![]); // 表示するメッセージ一覧の状態
    let (input, set_input) = create_signal(String::new()); // 入力中テキストの状態

    #[cfg(feature = "hydrate")] // ブラウザ実行時のみ WebSocket を保持
    let ws_ref = store_value(None::<web_sys::WebSocket>); // WebSocket を 'static に保つためのストア

    create_effect(move |_| {
        // マウント相当のタイミングで一度だけ実行
        #[cfg(feature = "hydrate")] // SSR 中はブラウザ API がないため除外
        {
            use wasm_bindgen::closure::Closure; // JS クロージャラッパ
            use wasm_bindgen::JsCast; // 型キャストヘルパ
            use web_sys::{MessageEvent, WebSocket}; // Web API 型

            let location = web_sys::window().unwrap().location(); // 現在のロケーション取得
            let host = location.host().unwrap_or_else(|_| "127.0.0.1:3000".into()); // ホスト名:ポート
            let protocol = location.protocol().unwrap_or_else(|_| "http:".into()); // http/https
            let ws_scheme = if protocol.starts_with("https") {
                "wss"
            } else {
                "ws"
            }; // ws/wss 判定
            let url = format!("{ws_scheme}://{host}/ws"); // WebSocket エンドポイント URL

            let ws = WebSocket::new(&url).expect("failed to open websocket"); // ソケット接続

            let onmessage = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                // メッセージ受信ハンドラ
                if let Some(text) = e.data().as_string() {
                    // テキストに変換できれば
                    set_messages.update(|v| v.push(text)); // 末尾に追加して再描画
                }
            });
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref())); // ハンドラ登録
            onmessage.forget(); // JS 側に所有権を渡して生存させる

            ws_ref.set_value(Some(ws)); // ソケットをストアに保存（Drop されないように）
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // 送信ボタン/Enter 押下時のハンドラ
        ev.prevent_default(); // フォームのデフォルト送信を抑止
        let text = input.get_untracked().trim().to_string(); // 入力値を取得してトリム
        if text.is_empty() {
            // 空入力は無視
            return; // 何もしない
        }
        #[cfg(feature = "hydrate")] // ブラウザ時のみ送信
        {
            if let Some(ws) = ws_ref.get_value() {
                // ソケットが確立済みなら
                let _ = ws.send_with_str(&text); // テキストを送信（エラーは無視）
            }
        }
        set_input.set(String::new()); // 入力欄をクリア
    };

    view! { // UI 描画
        <div class="space-y-3"> // 縦の余白
            <form on:submit=on_submit class="flex gap-2"> // 入力フォーム
                <input
                    class="flex-1 border rounded px-2 py-1" // 見た目の調整
                    placeholder="メッセージを入力" // プレースホルダ
                    prop:value=input // Signal とバインド
                    on:input=move |ev| set_input.set(event_target_value(&ev)) // 入力で状態更新
                />
                <button class="border rounded px-3">"送信"</button> // 送信ボタン
            </form>

            <div class="border rounded p-2 h-80 overflow-auto bg-gray-50"> // メッセージ表示領域
                <For
                    each=move || messages.get() // 配列を監視
                    key=|msg| msg.clone() // key はメッセージ文字列（簡易）
                    children=move |msg| view! { <div class="py-1">{msg}</div> } // 1 行ずつ描画
                />
            </div>
        </div>
    }
}
