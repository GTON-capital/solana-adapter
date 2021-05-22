

use std::fmt;
use std::marker::PhantomData;

use std::time::{Duration, SystemTime};

use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use solana_gravity_contract::gravity::state::PartialStorage;
use gravity_misc::model::{AbstractHashMap, HashMap};

// use bincode;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use uuid::v1::{Context, Timestamp};
use uuid::Uuid;


// pub trait AbstractHashMap<K, V> {
//     fn insert(&mut self, key: &K, val: V) {}

//     fn contains_key(&self, key: &K) -> bool {
//         false
//     }

//     fn get(&self, key: &K) -> Option<&V> {
//         None
//     }
// }

// #[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Default, Debug, Clone)]
// pub struct HashMap<K, V> {
//     k: Vec<K>,
//     v: Vec<V>,
// }

// impl<K, V> AbstractHashMap<K, V> for HashMap<K, V> {}


#[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone)]
pub enum RequestStatus {
    None,
    New,
    Rejected,
    Success,
    Returned
}

// #[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone)]
pub type ForeignAddress = [u8; 32];

// #[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone)]
pub type AttachedData = [u8; 64];

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone)]
pub struct UnwrapRequest {
    pub destination_address: ForeignAddress,
    pub origin_address: Pubkey,
}


pub type RequestsQueue<T> = Vec<T>;

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Default, Debug, Clone)]
pub struct IBPortContract {
    pub nebula_address: Pubkey,
    pub token_address: Pubkey,

    // pub swap_status: HashMap<u8, RequestStatus>,
    // pub unwrap_requests: HashMap<u8, UnwrapRequest>,


}
