use std::{future::poll_fn, io::Write, ops::DerefMut, pin::Pin, str::from_utf8, task::Poll};

use crate::{inputs::InputData, kv::KeyValue};
use anyhow::Error;
use tracing::{debug, info};


use crate::log::KvStore;

use tokio::sync::watch::{Receiver, Sender};

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

            let sx = (&tx).send(data); 
            if let Err(i) = sx {
                println!("Error: Value can't be sent across process: {i}");
                continue;
            }
        }
        std::io::stdout().flush().unwrap();
    }
}

pub async fn process_incoming_inputs(
    mut store: KvStore,
    mut term_rx: Receiver<InputData>,
    mut http_rx: Receiver<KeyValue>,
) {
    // tokio::pin!(term_rx); // don't ask me why! 😭
    // tokio::pin!(http_rx); // don't ask me why! 😭
    // tokio::pin!(store); 
    
    loop {
        let mut mut_term_rx = &mut term_rx;
        let mut mut_http_rx = &mut http_rx;
        // info!(" loop running");

        let f1 = poll_fn(|_| { 
            // if let Ok(true) = mut_term_rx.has_changed() {
            //     info!(" 1 obtained "); 
            //     return Poll::Ready(());
            // }
            // Poll::Pending

            Poll::Ready(mut_term_rx.has_changed().unwrap_or(false))
        });

        let f2 = poll_fn(|_| { 
            // if let Ok(true) = mut_http_rx.has_changed() {
            //     info!(" 2 obtained "); 
            //     return Poll::Ready(());
            // }
            // Poll::Pending
            Poll::Ready(mut_http_rx.has_changed().unwrap_or(false))
        });
        
        tokio::select! {
             _ = f1 => {
                // info!(" Accepted the change");
                accept_terminal_input(mut_term_rx, &mut store).await;
            },
            _ = f2 => {
                // info!(" The change **");
                accept_http_inputs(mut_http_rx, &mut store).await;
            }
        };

    };
}

#[inline(always)]
async fn accept_terminal_input(rx: &mut Receiver<InputData>, store: &mut KvStore) {
    if let Ok(true) = (rx).has_changed() {
        info!(" terminal got input");
        let data = {
            let data = (rx).borrow_and_update();
            let data = data.clone();
            data
        };

        match data {
            InputData::ReadInput(key) => {
                let new_key = key.strip_prefix("read ").unwrap().trim();
                debug!(" read value");
                let i = store.get(new_key).await;
                debug!(" value we got: {:?}", i);
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
                // continue;
            }
        }
    }
}

#[inline(always)]
async fn accept_http_inputs(rx: &mut Receiver<KeyValue>, store: &mut KvStore) {
    // info!("started http compute");
    if let Ok(true) = rx.has_changed() {
        let data = {
            let data = (rx).borrow_and_update();
            let data = data.clone(); //TODO utilize Cow<'_,T> for the job
            data
        };
        debug!("value: {:?}", data.value);
        store.set(data.key.clone(), data.value).await.unwrap();
        println!("inserted key: {}", data.key);
    }
}
