use std::io::{Write};


use crate::{inputs::InputData};
use anyhow::Error;

use log::KvStore;
use tokio::sync::watch::{Sender};

mod errors;
mod inputs;
mod kv;
mod log;

#[inline]
async fn get_inputs(buf: &mut String) -> Result<(), Error> {
    let val = std::io::stdin()
        .read_line(buf)
        .map(|_v| ())
        .map_err(|e| e.into());
    println!("{buf:?}");
    return val;
}

pub async fn process_inputs(tx: Sender<InputData>) {
    let mut buf = "".to_string();
    println!("< welcome to 🎗 >");

    loop {
        buf = "".to_string();
        print!("Ribbon > ");
        std::io::stdout().flush().unwrap();
        if let Ok(()) = get_inputs(&mut buf).await {
            let data: InputData = InputData::from(buf);

            if let InputData::Invalid = data {
                println!("Error: Incorrect command please retry again");
                continue;
            }

            if let InputData::ClearTerm = data {
                // std::io::stdout().lock().write("".as_bytes()).unwrap();
                clearscreen::clear();
                continue;
            }

            if let InputData::NewLine = data { 
                continue;  // enter the new line here
            }

            if let Err(_) = (&tx).send(data) {
                println!("Error: Value can't be sent across process");
                continue;
            }
        }
        println!("");
    }
}

#[tokio::main]
async fn main() {

    // Create message parsers
    let (tx, mut rx) = tokio::sync::watch::channel(InputData::Invalid);

    let task1 = tokio::task::spawn(async move {
        process_inputs(tx).await;
    });
    
    let task2 = tokio::task::spawn(async move {
        let mut store = KvStore::new("kvstore.log").await.unwrap();
        loop {
            if let Ok(true) = (rx).has_changed() {
                
                let data = {
                    let data = (rx).borrow_and_update();
                    let data = data.clone();
                    data
                };

                match data {
                    InputData::ReadInput(key) => { 
                        let i = store.get(&key).await.unwrap();
                        println!("key: {key}, value: {i:?}");
                    }
                    InputData::insert(field) => {
                        let k = field.key.clone(); 
                        let v = field.value.as_bytes().to_vec();  
                        let _t = store.set(k, v).await; 
                        println!("Insert succes");
                    }
                    InputData::remove(field) => {
                        let k = field.key.clone(); 
                        let val = store.remove(&k).await.unwrap(); 
                        println!("Removed key: {k}"); 
                    }
                    InputData::update(field) => {
                        let k = field.key.clone(); 
                        let v = field.updated_value.as_bytes().to_vec();  
                        let _t = store.set(k, v).await.unwrap();
                        println!("Updated key: {}", field.key);
                    }
                    _ => {
                        println!("Invalid buffer read");
                        continue;
                    }
                }
                std::io::stdout().flush().unwrap();

            }
        }
    });

    let _ = task2.await;
    let _ = task1.await;
}
