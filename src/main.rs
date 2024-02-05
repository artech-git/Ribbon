use crate::inputs::InputData;
use kv::KeyValue;
use log::KvStore;
use operations::{process_incoming_inputs, process_terminal_input};
use routes::build_server;


mod errors;
mod inputs;
mod kv;
mod log;
mod operations;
mod routes;

use tracing::{span, Level};

#[tokio::main]
async fn main() {

    // Create message parsers
    let (term_tx, term_rx) = tokio::sync::watch::channel(InputData::Invalid);

    let task1 = tokio::task::spawn(async move {
        let _event = process_terminal_input(term_tx).await;
    });

    //sending values from the update handle
    let (http_tx, http_rx) = tokio::sync::watch::channel(KeyValue::default());

    let task2 = tokio::task::spawn(async move {
        let _subs = tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stdout)
            .with_max_level(tracing::Level::TRACE)
            .init();
        let my_span = span!(Level::INFO, "Key value store span");
        my_span.enter();
        let store = KvStore::new("kvstore.log").await.unwrap();
        process_incoming_inputs(store, term_rx, http_rx).await;
    });

    let task3 = tokio::task::spawn(async move {
        let router = build_server(http_tx).await;
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    let _ = task1.await;
    let _ = task2.await;
    let _ = task3.await;
}
