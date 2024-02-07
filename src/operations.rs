use std::io::Write;

use crate::{inputs::InputData, kv::KeyValue, log::KvStoreShare};
use anyhow::Error;
use tracing::debug;

use crate::log::KvStore;

use tokio::sync::watch::Receiver;

#[inline]
async fn get_inputs(buf: &mut String) -> Result<(), Error> {
    std::io::stdin()
        .read_line(buf)
        .map(|_v| ())
        .map_err(|e| e.into())
}

pub async fn process_terminal_input(tx: KvStoreShare) {
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

            let sx = tx.set_data(data);
            if let Err(i) = sx.await {
                println!("Error: Value can't be sent across process: {i}");
                continue;
            }
        }
        std::io::stdout().flush().unwrap();
    }
}

#[inline(always)]
async fn accept_http_inputs(rx: &mut Receiver<KeyValue>, store: &mut KvStore) {
    // info!("started http compute");
    if let Ok(true) = rx.has_changed() {
        let data = {
            let data = (rx).borrow_and_update();
             //TODO utilize Cow<'_,T> for the job
            data.clone()
        };
        debug!("value: {:?}", data.value);
        store.set(data.key.clone(), data.value).await.unwrap();
        println!("inserted key: {}", data.key);
    }
}
