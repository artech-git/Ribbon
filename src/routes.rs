use std::sync::Arc;

use axum::{body::Body, response::IntoResponse, routing::get, Extension, Router};
use http::Response;

use crate::{
    kv::{Key, KeyValue},
    log::KvStoreShare,
};

type Handle = KvStoreShare;

pub async fn build_server(ext: KvStoreShare) -> Router {
    Router::new()
        .route("/set", get(update_value))
        .route("/get", get(get_value))
        .layer(Extension(ext))
}

pub async fn update_value(Extension(handle): Extension<Handle>, kv: KeyValue) -> impl IntoResponse {
    if kv.is_invalid() {
        return "Invalid content";
    }
    handle.set(kv.key, kv.value).await;
    "Accpeted"
}

pub async fn get_value(Extension(handle): Extension<Handle>, kv: Key) -> impl IntoResponse {
    let val = handle.get(&kv.key).await;
    let mut response = Response::new(Body::empty());

    if val.is_ok() {
        let body_mut = response.body_mut();
        *body_mut = Body::from(val.unwrap());
    }

    response
}
