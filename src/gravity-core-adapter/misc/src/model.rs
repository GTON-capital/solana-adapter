
use std::time::{SystemTime};

use thiserror::Error;

use solana_program::{
    program_error::ProgramError,
};

use borsh::{BorshDeserialize, BorshSerialize};

use uuid::v1::{Context, Timestamp};
use uuid::Uuid;


#[derive(Error, Debug, Copy, Clone)]
pub enum ValidationError {
    #[error("Error during extraction")]
    ExtractionError
}

impl From<ValidationError> for ProgramError {
    fn from(e: ValidationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

pub type SubscriptionID = [u8; 16];
pub type PulseID = u64;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum DataType {
    Int64,
    String,
    Bytes,
}

impl Default for DataType {
    fn default() -> Self {
        DataType::Int64
    }
}


impl DataType {
    pub fn cast_from(i: u8) -> DataType {
        match i {
            0 => DataType::Int64,
            1 => DataType::String,
            2 => DataType::Bytes,
            _ => panic!("invalid data type"),
        }
    }
}

pub const MAX_RECORDS_COUNT: usize = 20;

pub trait AbstractRecordHandler<K, V> {
    fn insert(&mut self, _key: K, _val: V) {}

    fn contains_key(&self, _key: &K) -> bool {
        false
    }

    fn get(&self, _key: &K) -> Option<&V> {
        None
    }

    fn drop(&mut self, key: &K) -> Option<V>;
}


// No BorshSchema
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Default, Debug, Clone)]
pub struct RecordHandler<K, V> {
    k: Vec<K>,
    v: Vec<V>,
    // k: [K; MAX_RECORDS_COUNT],
    // v: [V; MAX_RECORDS_COUNT],
    // length: usize,
    // last_element_index: usize
}


impl<K: Default + Clone, V: Default + Clone> RecordHandler<K, V> {
    
    pub fn new() -> RecordHandler<K, V> {
        RecordHandler::default()
    }
    
    // return actual length
    pub fn len(&self) -> usize {
        self.k.len()
    }

    pub fn cap(&self) -> usize {
        MAX_RECORDS_COUNT
    }

    pub fn is_full(&self) -> bool {
        self.cap() == self.len()
    }
}

impl<K: PartialEq + Default + Clone, V: Default + Clone> AbstractRecordHandler<K, V> for RecordHandler<K, V> {
    fn insert(&mut self, key: K, val: V) {
        // overwrite logic
        for (pos, internal_key) in self.k.iter().enumerate() {
            if *internal_key == key {
                self.v[pos] = val.clone();
                return;
            }
        }

        self.k.push(key);
        self.v.push(val);
    }

    fn contains_key(&self, key: &K) -> bool {
        for (_pos, internal_key) in self.k.iter().enumerate() {
            if internal_key == key {
                return true;
            }
        }
        return false
    }

    // retrieve element or "None" is returned
    fn get(&self, key: &K) -> Option<&V> {
        for (pos, internal_key) in self.k.iter().enumerate() {
            if internal_key == key {
                return Some(&self.v[pos]);
            }
        }
        None
    }

    // drop value and return it, if nothing dropped - "None" is returned
    fn drop(&mut self, key: &K) -> Option<V> {
        if self.k.len() == 0 {
            return None;
        }

        for (pos, internal_key) in self.k.iter().enumerate() {
            if internal_key == key {
                let res = self.v[pos].clone();

                self.k.remove(pos);
                self.v.remove(pos);

                return Some(res);
            }
        }
        None
    }
}

pub type U256 = [u8; 32];


pub fn new_uuid(node_id: &[u8]) -> Uuid {
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

    let context = Context::new(777);

    let ts = Timestamp::from_unix(
        &context,
        current_time.as_secs(),
        current_time.subsec_nanos(),
    );

    let uuid = Uuid::new_v1(ts, node_id).expect("failed to generate UUID");
    uuid
}