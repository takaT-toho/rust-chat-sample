use std::{net::SocketAddr, path::PathBuf};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRef, State,
    },
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use leptos::{provide_context, view, LeptosOptions};
use leptos_axum::{generate_route_list, LeptosRoutes};
use tokio::{net::TcpListener, sync::broadcast};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type WsSender = broadcast::Sender<String>;

use chat_leptos_axum::app::App;

#[derive(Clone)]
struct AppState {
    leptos_options: LeptosOptions,
    tx: WsSender,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> LeptosOptions {
        state.leptos_options.clone()
    }
}

impl FromRef<AppState> for WsSender {
    fn from_ref(state: &AppState) -> WsSender {
        state.tx.clone()
    }
}

pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (tx, _rx) = broadcast::channel::<String>(1000);

    let config = leptos::get_configuration(None).await?;
    let leptos_options = config.leptos_options;
    let routes = generate_route_list(|| view! { <App/> });
    let addr: SocketAddr = leptos_options.site_addr;

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        tx: tx.clone(),
    };

    let routes_state = app_state.clone();
    let routes_tx = tx.clone();
    let fallback_tx = tx.clone();

    let leptos_fallback = leptos_axum::render_app_to_stream_with_context(
        leptos_options.clone(),
        move || provide_context(fallback_tx.clone()),
        || view! { <App/> },
    );

    let site_root = PathBuf::from(&leptos_options.site_root);
    let pkg_dir = site_root.join(&leptos_options.site_pkg_dir);
    let static_service =
        get_service(ServeDir::new(site_root).not_found_service(get(leptos_fallback.clone())));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .leptos_routes_with_context(
            &routes_state,
            routes,
            move || provide_context(routes_tx.clone()),
            || view! { <App/> },
        )
        .nest_service("/pkg", get_service(ServeDir::new(pkg_dir)))
        .fallback_service(static_service)
        .with_state(app_state);

    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    let tx_incoming = state.tx.clone();

    loop {
        tokio::select! {
            maybe_msg = socket.recv() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        let _ = tx_incoming.send(text);
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
