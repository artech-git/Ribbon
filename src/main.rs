use log::{KvStore, KvStoreShare};
use operations::process_terminal_input;
use routes::build_server;

mod errors;
mod inputs;
mod kv;
mod log;
mod operations;
mod routes;

#[tokio::main]
async fn main() {

    let _subs = tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stdout)
            .with_max_level(tracing::Level::TRACE)
            .init();

    let store = KvStore::new("kvstore.log").await.unwrap();

    let store_handle_one = KvStoreShare::new(store);
    let store_handle_two = store_handle_one.clone();

    let task1 = tokio::task::spawn(async move {
        let _event = process_terminal_input(store_handle_one).await;
    });

    let task2 = tokio::task::spawn(async move {
        let router = build_server(store_handle_two).await;
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    let _ = task1.await;
    let _ = task2.await;
}
