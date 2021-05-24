
// use std::collections::HashMap;
use thiserror::Error;
use std::fmt;
use std::marker::PhantomData;

use std::time::{Duration, SystemTime};

use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    msg,
};

// use bincode;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

// use serde::{Deserialize, Serialize};
use uuid::v1::{Context, Timestamp};
use uuid::Uuid;


// use nebula::{
//     instruction::NebulaContractInstruction,
//     state::{DataType, NebulaContract, PulseID},
// };

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

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone)]
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


pub trait AbstractHashMap<K, V> {
    fn insert(&mut self, key: &K, val: V) {}

    fn contains_key(&self, key: &K) -> bool {
        false
    }

    fn get(&self, key: &K) -> Option<&V> {
        None
    }
}

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Default, Debug, Clone)]
pub struct HashMap<K, V> {
    k: Vec<K>,
    v: Vec<V>,
}

impl<K, V> HashMap<K, V> {
    pub fn len(&self) -> usize {
        self.k.len()
    }
}

impl<K, V> AbstractHashMap<K, V> for HashMap<K, V> {
    fn insert(&mut self, key: &K, val: V) {
        // if self.k.is_empty() {
        //     self.k.push(key.clone());
        //     self.v.push(val.clone());
        //     return;
        // }

        // let (insertion_index, is_new) = if self.k.contains(key) {

        // } else {

        // }
    }

    fn contains_key(&self, key: &K) -> bool {
        false
    }

    fn get(&self, key: &K) -> Option<&V> {
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
    // let sub_id = uuid.as_bytes();
    // sub_id
    uuid
}