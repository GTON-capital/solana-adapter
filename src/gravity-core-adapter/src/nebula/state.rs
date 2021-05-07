use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};

use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::gravity::state::PartialStorage;
use crate::nebula::error::NebulaError;

use bincode;
use serde::{Deserialize, Serialize};
use uuid::v1::{Timestamp, Context};
use uuid::Uuid;

// extern crate sha2;
// use sha2::Sha256;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

// pub type SubscriptionID<'a> = &'a [u8];
// pub type SubscriptionID = Vec<u8>;
pub type SubscriptionID = [u8; 16];
pub type PulseID = u64;

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
pub struct Subscription {
    pub sender: Pubkey,
    pub contract_address: Pubkey,
    pub min_confirmations: u8,
    pub reward: u64, // should be 2^256
}

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
pub struct Pulse {
    pub data_hash: Vec<u8>,
    pub height: u128,
}

// #[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
// pub struct Oracle<A> {
//     pub address: A,
//     pub is_online: bool,
//     pub id_in_queue: SubscriptionID<'a>,
// }

pub type NebulaQueue<T> = Vec<T>;

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
pub struct NebulaContract {
    pub rounds_dict: HashMap<PulseID, bool>,
    subscriptions_queue: NebulaQueue<SubscriptionID>,
    pub oracles: Vec<Pubkey>,

    pub bft: u8,
    pub multisig_account: Pubkey,
    pub gravity_contract: Pubkey,
    pub data_type: DataType,
    pub last_round: PulseID,

    // subscription_ids: Vec<SubscriptionID>,
    pub last_pulse_id: PulseID,

    subscriptions_map: HashMap<SubscriptionID, Subscription>,
    pulses_map: HashMap<PulseID, Pulse>,
    is_pulse_sent: HashMap<PulseID, HashMap<SubscriptionID, bool>>,

    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
}

impl PartialStorage for NebulaContract {
    const DATA_RANGE: std::ops::Range<usize> = 0..2000;
}

impl Sealed for NebulaContract {}

impl IsInitialized for NebulaContract {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for NebulaContract {
    const LEN: usize = 2000;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        bincode::deserialize(&src[..]).unwrap()
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let encoded_nebula: Vec<u8> = bincode::serialize(&self).unwrap();
        let nebula_sliced = encoded_nebula.as_slice();

        for (i, val) in nebula_sliced.iter().enumerate() {
            dst[i] = *val;
        }
    }
}

impl NebulaContract {

    pub fn add_pulse(&mut self, new_pulse_id: PulseID, data_hash: Vec<u8>, block_number: u128) -> Result<(), NebulaError> {
        self.pulses_map.insert(new_pulse_id, Pulse {
            data_hash,
            height: block_number
        });

        Ok(())
    }

    pub fn subscribe(
        &mut self,
        // sub_id: &SubscriptionID,
        sender: Pubkey,
        contract_address: Pubkey,
        min_confirmations: u8,
        reward: u64,
    ) -> Result<(), NebulaError> {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let context = Context::new(50);

        let ts = Timestamp::from_unix(&context, current_time.as_secs(), current_time.subsec_nanos());
        let subscription = Subscription {
            sender,
            contract_address,
            min_confirmations,
            reward,
        };

        let serialized_subscription: Vec<u8> = bincode::serialize(&subscription).unwrap();
        let uuid = Uuid::new_v1(ts, &serialized_subscription[0..6]).expect("failed to generate UUID");

        let sub_id = uuid.as_bytes();

        self.subscriptions_map.insert(*sub_id, subscription);

        Ok(())
    }
}
