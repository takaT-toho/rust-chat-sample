#![cfg(feature = "hydrate")] // ブラウザ側ビルド（Wasm）のときのみコンパイル

use crate::app::App;
use leptos::*; // Leptos API // ルートコンポーネント

#[wasm_bindgen::prelude::wasm_bindgen(start)] // Wasm モジュールのエントリポイント
pub fn main() {
    // ブラウザで最初に呼ばれる関数
    console_error_panic_hook::set_once(); // パニックを console にわかりやすく出す
    mount_to_body(|| view! { <App/> }); // 既存の SSR DOM に Hydration を適用
}
