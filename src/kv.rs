
use serde::{Deserialize, Serialize};



// #[derive(Clone, Debug)]
// pub struct KeyValue { 
//     inner: Arc<Mutex<InnerValue>>
// } 

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyValue {
    pub key: String,
    pub value: Vec<u8>,
}

impl KeyValue {
    fn new(key: String, value: Vec<u8>) -> Self {
        Self { key, value }
    }

    // pub async fn insert(&self) -> Result<(), ()> {
    //     todo!()
    // }

    // pub async fn read(&self, _key: Key) -> Option<Cow<'_, Message>> {
    //     todo!()
    // }

    // pub async fn remove(&self, _key: Key) -> Option<()> {
    //     todo!()
    // }
}

// #[derive(Clone, Serialize, Deserialize, Debug, Deref, DerefMut)]
// pub struct Message(Vec<u8>); // size of data to hold

// #[derive(Clone, Serialize, Deserialize, Debug, Deref, DerefMut)]
// pub struct Key(String); // size of data to hold
