// use std::collections::DashMap;
// use std::fs::{File, OpenOptions};
use std::io::{BufRead, Seek, SeekFrom};



use dashmap::DashMap;

use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt}; 
use tokio::io::{AsyncBufReadExt, BufStream};
use tokio::fs::{File, OpenOptions};
 


use crate::errors::{BResult, KvStoreError};
use crate::kv::KeyValue;


pub struct KvStore {
    log_file: BufStream<File>,
    index: DashMap<String, u64>, // Map keys to offsets in the log
}

impl KvStore {

    pub async fn new(filename: &str) -> BResult<Self> {

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
            .await
            .unwrap();

        let (index, log_file) = if file.metadata().await.unwrap().len() > 0 {
            let mut log_file = BufStream::new(file);
            // Load existing index from the log
            (Self::load_index(&mut log_file).await.unwrap(), log_file)
        } else {
            (DashMap::new(), BufStream::new(file))
        };

        Ok(Self { log_file: log_file, index: index })
    }

    pub async fn set(&mut self, key: String, value: Vec<u8>) -> BResult<()> {
        
        // let mut arc_log_file = self.log_file; 
        // let arc_index = self.index; 
        
        let serialized = serde_json::to_string(&KeyValue { key: key.clone(), value }).unwrap();
        self.log_file.write_all(&serialized.as_bytes()).await.unwrap();
        self.log_file.flush().await.unwrap();
        let offset = self.log_file.seek(SeekFrom::Current(0)).await.unwrap();
        self.index.insert(key, offset);
        Ok(())
    }

    pub async fn get(&mut self, key: &str) -> BResult<Vec<u8>> {
        let offset = self.index.get(key).ok_or(KvStoreError::KeyNotFound).unwrap();
        (self.log_file).seek(SeekFrom::Start(*offset)).await.unwrap();

        let mut buffer = Vec::new();
        self.log_file.read_to_end(&mut buffer).await.unwrap();

        let KeyValue { key: _, value }: KeyValue = serde_json::from_slice(&buffer).unwrap();
        Ok(value)
    }

    // pub fn compact(&mut self) -> BResult<()> {
    //     // ... (Implementation as described in previous response)
    //     todo!()
    // }

    // Helper function to load the index from the log
    async fn load_index(log_file: &mut BufStream<File>) -> BResult<DashMap<String, u64>> {
        // ... (Read and deserialize index entries)
        let index = DashMap::new();

        let mut buffer = "".to_string(); 

        // Iterate over the log file line by line
        while let Ok(_) = log_file.read_line(&mut buffer).await {

            // Deserialize the KeyValue struct from the line
            let key_value: KeyValue = serde_json::from_str(&buffer).unwrap();

            // Extract the key and its offset from the file
            let offset = log_file.stream_position().await.unwrap() - buffer.len() as u64 - 1; // Account for newline

            // Insert the key and offset into the index
            index.insert(key_value.key, offset);
        }

        Ok(index)
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

// Helper functions for serialization, deserialization, and data validity checks
