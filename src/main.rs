use std::{io::Write, str::from_utf8};

use crate::inputs::InputData;
use anyhow::Error;


use log::KvStore;

use tokio::sync::watch::{Receiver, Sender};

mod errors;
mod inputs;
mod kv;
mod log;

#[inline]
async fn get_inputs(buf: &mut String) -> Result<(), Error> {
    std::io::stdin()
        .read_line(buf)
        .map(|_v| ())
        .map_err(|e| e.into())
}

pub async fn process_terminal_input(tx: Sender<InputData>) {
    println!("< welcome to 🎗 >");
    loop {
        let mut buf = "".to_string();
        print!("Ribbon > ");
        std::io::stdout().flush().unwrap();
        if let Ok(()) = get_inputs(&mut buf).await {
            let data: InputData = InputData::from(buf);

            if let InputData::Invalid = data {
                println!("Error: Incorrect command please retry again");
                continue;
            }

            if let InputData::ClearTerm = data {
                let _ = clearscreen::clear();
                continue;
            }

            if let InputData::NewLine = data {
                continue; // enter the new line here
            }

            if let Err(_) = (&tx).send(data) {
                println!("Error: Value can't be sent across process");
                continue;
            }
        }
        std::io::stdout().flush().unwrap();
    }
}

pub async fn process_incoming_inputs(mut store: KvStore, mut rx: Receiver<InputData>) {
    loop {
        if let Ok(true) = (rx).has_changed() {
            let data = {
                let data = (rx).borrow_and_update();
                let data = data.clone();
                data
            };

            match data {
                InputData::ReadInput(key) => {
                    let new_key = key.strip_prefix("read ").unwrap().trim();

                    let i = store.get(new_key).await;
                    if let Ok(u) = i {
                        println!(" value: {:?}", from_utf8(&u).unwrap());
                    }
                }
                InputData::Insert(field) => {
                    let k = field.key.clone();
                    let v = field.value.as_bytes().to_vec();
                    let _t = store.set(k, v).await;
                    println!(" Insert succes");
                }
                InputData::Remove(field) => {
                    let k = field.key.clone();
                    let _val = store.remove(&k).await.unwrap();
                    println!(" Removed key: {k}");
                }
                InputData::Update(field) => {
                    let k = field.key.clone();
                    let v = field.updated_value.as_bytes().to_vec();
                    let _t = store.set(k, v).await.unwrap();
                    println!(" Updated key: {}", field.key);
                }
                _ => {
                    println!(" Invalid buffer read");
                    continue;
                }
            }
            std::io::stdout().flush().unwrap();
        }
    }
}


use tracing::{span, Level};

#[tokio::main]
async fn main() {
    // Create message parsers
    let (tx, rx) = tokio::sync::watch::channel(InputData::Invalid);

    let task1 = tokio::task::spawn(async move {
        let event = 
        process_terminal_input(tx).await;
    });

    let task2 = tokio::task::spawn(async move {

        let subs = tracing_subscriber::fmt::Subscriber::builder()
            .with_writer(std::io::stdout)
            .with_max_level(tracing::Level::TRACE)
            .init(); 

        let mut my_span = span!(Level::INFO, "Key value store span");
        my_span.enter();  
        let store = KvStore::new("kvstore.log").await.unwrap();
        // my_span.exit(); 
 
        process_incoming_inputs(store, rx).await;
    });

    // let task3 = tokio::task::spawn(async move { 
    //     let route = Router::new()
    //         .route("/set", insert_key)
    //         .route("/get" , update_key);

    //     route
    // });

    let _ = task2.await;
    let _ = task1.await;
}

// async fn build_server() -> Router {
//     let route = Router::new()
//         .route("/set", insert_key)
//         .route("/get" , update_key);

//     route
// }

// async fn insert_key(
//     mut Json(val): axum::Json<HashMap<String, String>>, 
//     mut req: Request<Body>
// ) -> Impl IntoResponse { 



// }