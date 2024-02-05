use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyValue {
    pub key: String,
    pub value: Vec<u8>,
}
