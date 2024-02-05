
use std::io::SeekFrom;
use std::ops::Deref;

use dashmap::DashMap;

use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, BufStream};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tracing::log::{info,debug}; 
use tracing::span;  
use tracing::Level; 
use crate::errors::{BResult, KvStoreError};

use crate::kv::KeyValue;

pub struct KvStore {
    log_file: BufStream<File>,
    pub index: DashMap<String, u64>,
}

impl KvStore {
    
    pub async fn new(filename: &str) -> BResult<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
            .await
            .map_err(|e| KvStoreError::IoError(e))?;

        let (index, log_file) = if file.metadata().await.unwrap().len() > 0 {
            // Load existing index from the log
            let mut log_file = BufStream::new(file);
            (Self::load_index(&mut log_file).await.unwrap(), log_file)
        } else {
            (DashMap::new(), BufStream::new(file))
        };

        Ok(Self {
            log_file: log_file,
            index: index,
        })
    }

    pub async fn get(&mut self, key: &str) -> BResult<Vec<u8>> {
        let offset = self.index.get(key).ok_or(KvStoreError::KeyNotFound)?;

        let _seek = (self.log_file)
            .seek(SeekFrom::Start(*offset.deref()))
            .await
            .map_err(|e| KvStoreError::IoError(e))?;

        let mut buffer = vec![];
        let _y = self.log_file.read_until(b'\n', &mut buffer).await;

        let KeyValue { key: _, value }: KeyValue = serde_json::from_slice(&buffer).unwrap();
        Ok(value)
    }

    pub async fn load_index(log_file: &mut BufStream<File>) -> BResult<DashMap<String, u64>> {
        // ... (Read and deserialize index entries)
        let mut index = DashMap::new();
        let mut buffer = "".to_string();
        let mut offset = 0;
        let mut line_count = 0; 
        info!("Entering file loop");
        // Iterate over the log file line by line
        let loop_span = span!(Level::INFO, "entering loop span");
        while let Ok(bytes_read) = log_file.read_line(&mut buffer).await {
            info!("line number: {} bytes read: {}", line_count, bytes_read); 
            if bytes_read == 0 {
                debug!("**breaked the program**");
                break;
            }
            if buffer == r#"\n"# || (buffer.len() == 1){ 
                debug!(" New Line found");
                buffer.clear(); 
                continue; 
            }
            let data_to_append = check_line_for_operation(&mut buffer).await?;

            let _res =
                trim_header_and_insert(&mut index, offset as u64, &data_to_append, &buffer)
                .await
                .unwrap();
            offset += bytes_read;
            line_count += 1; 
            buffer.clear();
        }

        let _ = log_file.write_all("\n".as_bytes()).await;
        log_file.flush().await.unwrap();
        Ok(index)
    }

    pub async fn set(&mut self, key: String, value: Vec<u8>) -> BResult<()> {
        let serialized = serde_json::to_string(&KeyValue {
            key: key.clone(),
            value,
        })
        .map_err(|_e| KvStoreError::KeyNotFound)?;

        let curr_offset = self
            .log_file
            .stream_position()
            .await
            .map_err(|e| KvStoreError::IoError(e))?;
        let line = format!("[read]:{}\n", serialized);
        self.log_file
            .write_all(&line.as_bytes())
            .await
            .map_err(|_e| KvStoreError::InvalidCommand)?;

        self.log_file.flush().await.unwrap(); //TODO: transfer the periodic flush method to a seperate task

        self.index
            .entry(key)
            .and_modify(|f| *f = curr_offset + 7)
            .or_insert(curr_offset + 7);

        Ok(())
    }

    pub async fn remove(&mut self, key: &str) -> BResult<()> {
        if let Some(_val) = self.index.remove(key) {
            let line = format!("[remove]:{}\n", key);
            self.log_file.write_all(line.as_bytes()).await.unwrap();
            self.log_file.flush().await.unwrap();
        }
        Ok(())
    }
}

enum Operation {
    Update,
    Read,
    Remove,
}

#[allow(private_interfaces, unreachable_code)]
pub async fn trim_header_and_insert(
    index: &mut DashMap<String, u64>,
    offset: u64,
    ops: &Operation,
    buf: &String,
) -> BResult<()> {
    match ops {
        Operation::Update => {
            let value = buf.strip_prefix("[update]:");
            let content = value.unwrap();
            let key_value: KeyValue = serde_json::from_str(&content).unwrap();
            //println!("*update: {content}");

            let _y = index
                .entry(key_value.key)
                .and_modify(|v| *v = offset + 9)
                .or_insert(offset + 9);
            return Ok(());
        }
        Operation::Read => {
            let value = buf.strip_prefix("[read]:");
            let content = value.unwrap();
            // //println!("*content: {content}");
            let key_value: KeyValue = serde_json::from_str(content).unwrap();

            let _y = index
                .entry(key_value.key)
                .and_modify(|v| *v = offset + 7)
                .or_insert(offset + 7);
            return Ok(());
        }
        Operation::Remove => {
            let value = buf.strip_prefix("[remove]:");
            let mut content = value.unwrap().trim().to_string();

            let _t = content.pop();
            content.remove(0);
            // //println!("^key : {content}");
            let _c = index.remove(content.deref());
            // //println!("&& res: {:?}", _c);
            return Ok(());
        }
    }
    return Err(KvStoreError::InvalidCommand);

}

#[allow(private_interfaces)]
#[inline]
pub async fn check_line_for_operation(buf: &mut String) -> BResult<Operation> {
    /*
        ops:
            [Update]
            [read] //insert & modify
            [remove]
    */
    if buf.contains("[update]") {
        debug!("data update");
        return Ok(Operation::Update);
    }
    if buf.contains("[read]") {
        debug!("read the data");
        return Ok(Operation::Read);
    }
    if buf.contains("[remove]") {
        debug!("remove data");
        return Ok(Operation::Remove);
    }
    debug!("unknown line discovered");
    return Err(KvStoreError::InvalidFileHeader);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ky_store() {
        let subs = tracing_subscriber::fmt::Subscriber::default(); 
        let mut kv = KvStore::new("kvstore_2.log").await.unwrap();
        println!("Data {:#?}", kv.index);
        let val = kv.get("key3").await.unwrap();
        // let val2 = kv.get("tea").await;
        assert_eq!(val, vec![38,34,101]);
        // assert_eq!(val2.is_err(), true);
    }
}

// // code for compaction of data.
// pub fn compact(&mut self) -> Result<()> {
//     let mut temp_file = OpenOptions::new()
//         .create(true)
//         .write(true)
//         .open(tempfile::NamedTempFile::new().unwrap());

//     let mut writer = BufWriter::new(temp_file);

//     // Iterate over the existing log, copying valid data
//     for (key, offset) in &self.index {
//         let mut buffer = Vec::new();
//         self.log_file.seek(SeekFrom::Start(*offset)).unwrap();
//         self.log_file.read_to_end(&mut buffer).unwrap();

//         // Deserialize and check if data is still relevant
//         let (entry_key, entry_value) = deserialize_from_buffer(&buffer).unwrap();
//         if entry_key == *key && is_valid(&entry_value) {
//             // Write to the new log and update index
//             writer.write_all(&serialize_to_buffer(key, &entry_value)).unwrap();
//             self.index.insert(key.to_owned(), writer.tell() - 1);
//         }
//     }

//     writer.flush().unwrap();
//     writer.into_inner().unwrap();

//     // Replace the old log with the new one
//     std::fs::rename(temp_file.into_path(), "kvstore.log").unwrap();

//     Ok(())
// }
