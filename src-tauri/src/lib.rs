use std::{path::PathBuf, task::Poll, thread::sleep};

use axum::{http::request::Parts, routing::get};
use rspc::Config;
use tauri::App;
use tokio::pin;
use tokio_stream::{
    wrappers::{BroadcastStream, ReceiverStream},
    Stream,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Clone)]
struct MyCtx {
    handle: tauri::AppHandle,
}

pin_project_lite::pin_project! {
    struct TokenBroadcast<T>{
        #[pin]
        inner: BroadcastStream<T>,
    }
}

impl Stream for TokenBroadcast<String> {
    type Item = String;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.inner.poll_next(cx) {
            Poll::Ready(Some(Ok(v))) => Poll::Ready(Some(v)),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(format!("Error: {:?}", e))),
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

fn setup_axum(app: &mut App) {
    let handle = app.handle().clone();
    let my_ctx = MyCtx {
        handle: handle.clone(),
    };
    let (tx, rx) = tokio::sync::broadcast::channel::<String>(32);
    let router = rspc::Router::<MyCtx>::new()
        .config(
            Config::new()
                // Doing this will automatically export the bindings when the `build` function is called.
                .export_ts_bindings(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
                ),
        )
        .query("version", |t| t(|ctx: MyCtx, _: ()| "1.0.0"))
        .mutation("greet", |t| {
            t(|ctx, input: String| async move {
                format!("Hello, {}! You've been greeted from Rust!", input)
            })
        })
        .subscription("test_event", |t| {
            let tx_ = tx.clone();
            t(move |ctx, input: ()| {
                let tx_ = tx_.clone();
                async_stream::stream! {
                    while let Ok(item) = tx_.subscribe().recv().await {
                        yield item
                    }
                }
            })
        })
        .build()
        .arced();

    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(e) = tx.send("Hello from Rust!".to_string()) {
                eprintln!("failed to send, {:#?}", e);
            }
            sleep(std::time::Duration::from_secs(5));
        }
    });
    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            rspc_axum::endpoint(router.clone(), move |parts: Parts| {
                println!("Client requested operation '{}'", parts.uri.path());
                my_ctx.clone()
            }),
        )
        .with_state(handle);

    tauri::async_runtime::spawn(async move {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            setup_axum(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
