use std::pin::Pin;

use axum::{body::Body, extract::Request};
use futures_core::Future;
use serde::{Deserialize, Serialize};
use tracing::log::info;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: Vec<u8>,
}

impl KeyValue {
    pub fn is_invalid(&self) -> bool {
        self.key
            .chars()
            .any(|c| !(c.is_numeric() || c.is_alphabetic()))
    }
}

impl<T> axum::extract::FromRequest<T> for KeyValue {
    type Rejection = ();

    fn from_request<'life0, 'async_trait>(
        req: Request<Body>,
        _state: &'life0 T,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let (_header, body) = req.into_parts();

        let raw_bytes =
            futures::executor::block_on(axum::body::to_bytes(body, usize::MAX)).unwrap();
        info!(" block value: {:?}", raw_bytes);
        let hm: KeyValue = serde_json::from_slice(&raw_bytes).unwrap();

        Box::pin(async { Ok(hm) })
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Key {
    pub key: String,
}

impl Key {
    pub fn is_invalid(&self) -> bool {
        self.key
            .chars()
            .any(|c| !(c.is_numeric() || c.is_alphabetic()))
    }
}

impl<T> axum::extract::FromRequest<T> for Key {
    type Rejection = ();

    fn from_request<'life0, 'async_trait>(
        req: Request<Body>,
        _state: &'life0 T,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let (_header, body) = req.into_parts();
        let raw_bytes =
            futures::executor::block_on(axum::body::to_bytes(body, usize::MAX)).unwrap();
        let str = String::from_utf8(raw_bytes.to_vec()).unwrap();
        let k = Key { key: str };

        Box::pin(async { Ok(k) })
    }
}
