#![allow(dead_code, unused_variables)]

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::Crc;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

type ByteStr = [u8];
type ByteString = Vec<u8>;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValuePair {
    key: ByteString,
    value: ByteString,
}

// Algorithm for checksum
const CHECK: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_CKSUM);

/// Structure containing:
/// file: File opened to create buffers to read/write into the DB.
/// index: HashMap of keys and their byte positions in the file.
#[derive(Debug)]
pub struct ActionKV {
    file: File,
    index: HashMap<ByteString, u64>,
}

impl ActionKV {
    /// Open the file containing the database
    pub fn open(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(path)?;
        let index = HashMap::new();
        Ok(ActionKV { file, index })
    }

    /// Load the contents of the database to the index.
    pub fn load(&mut self) -> io::Result<()> {
        let mut f = BufReader::new(&mut self.file);

        // Read the contents until EOF is reached
        loop {
            let position = f.seek(SeekFrom::Current(0))?;
            let kv = match Self::read_record(&mut f) {
                Ok(kv) => kv,
                Err(err) => match err.kind() {
                    io::ErrorKind::UnexpectedEof => break,
                    _ => return Err(err),
                },
            };

            // Remove from index if value is empty
            if kv.value == b"" {
                self.index.remove(&kv.key);
            } else {
                self.index.insert(kv.key, position);
            }
        }
        Ok(())
    }

    /// Read a single record in the position of the buffer.
    fn read_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {
        // Read 12 bytes of metadata
        let checksum = f.read_u32::<LittleEndian>()?;
        let key_len = f.read_u32::<LittleEndian>()?;
        let value_len = f.read_u32::<LittleEndian>()?;

        // Read variable len key and value
        let data_len = key_len + value_len;
        let mut data = ByteString::with_capacity(data_len as usize);
        let _ = f.take(data_len.into()).read_to_end(&mut data);

        debug_assert_eq!(data_len as usize, data.len());

        // Verify checksum
        let data_checksum = CHECK.checksum(&data as &ByteStr);
        if data_checksum != checksum {
            panic!("Data corruption error: {} != {}", data_checksum, checksum);
        };

        // Split data and build KeyValuePair
        let value = data.split_off(key_len as usize);
        let key = data;
        Ok(KeyValuePair { key, value })
    }

    /// Writes a single record at the end of the file.
    fn write_record(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<u64> {
        // Calculate data payload
        let key_len = key.len();
        let value_len = value.len();
        let mut data = ByteString::with_capacity(key_len + value_len);
        key.iter().for_each(|k| data.push(*k));
        value.iter().for_each(|k| data.push(*k));

        // Calculate checksum of payload
        let checksum = CHECK.checksum(&data);

        // Write the data
        let mut f = BufWriter::new(&mut self.file);
        let position = f.seek(SeekFrom::End(0))?;
        f.write_u32::<LittleEndian>(checksum)?;
        f.write_u32::<LittleEndian>(key_len as u32)?;
        f.write_u32::<LittleEndian>(value_len as u32)?;
        f.write_all(&data)?;

        Ok(position)
    }

    /// List all keys present in the index.
    pub fn list_keys(&self) -> Vec<String> {
        self.index
            .keys()
            .map(|k| String::from_utf8_lossy(k).into())
            .collect()
    }

    /// Get the value of a key.
    pub fn get(&self, key: &ByteStr) -> io::Result<Option<ByteString>> {
        let position = if let Some(p) = self.index.get(key) {
            *p
        } else {
            return Ok(None);
        };

        let mut f = BufReader::new(&self.file);
        f.seek(SeekFrom::Start(position))?;
        Self::read_record(&mut f).map(|kv| Some(kv.value))
    }

    /// Append a new key-value payload to the database file and update the index.
    pub fn insert(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()> {
        let position = self.write_record(key, value)?;
        self.index.insert(key.into(), position);
        Ok(())
    }

    /// Same as appending the same key at the end with the new value.
    pub fn update(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()> {
        self.insert(key, value)?;
        Ok(())
    }

    /// Same as appending the same key with an empty value.
    pub fn delete(&mut self, key: &ByteStr) -> io::Result<()> {
        self.insert(key, b"")?;
        Ok(())
    }
}
