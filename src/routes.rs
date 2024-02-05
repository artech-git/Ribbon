use std::sync::Arc;

use axum::{
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use tokio::sync::watch::{Sender};

use crate::{kv::KeyValue};

type Handle = Arc<Sender<KeyValue>>;

pub async fn build_server(ext: Sender<KeyValue>) -> Router {
    let route = Router::new()
        .route("/set", get(update_value))
        .layer(Extension(Arc::new(ext)));

    route
}

pub async fn update_value(Extension(handle): Extension<Handle>, kv: KeyValue) -> impl IntoResponse {
    if kv.is_invalid() {
        return "Invalid content";
    }
    handle.send(kv).unwrap();
    return "Accpeted";
}


// pub async fn obtain_value(
//     Extension(handle): Extension<Handle>,
//     key: Request<String>
// ) -> impl IntoResponse {
//     let (header, body) = key.into_parts();
// }
