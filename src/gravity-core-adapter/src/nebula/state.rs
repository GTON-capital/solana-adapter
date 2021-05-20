// use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

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
use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

// extern crate sha2;
// use sha2::Sha256;

pub trait AbstractHashMap<K, V> {
    fn insert(&mut self, key: K, val: V) {}

    fn contains_key(&self, key: &K) -> bool {
        false
    }

    fn get(&self, key: &K) -> Option<&V> {
        None
    }
}

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
pub struct HashMap<K, V> {
    k: PhantomData<K>,
    v: PhantomData<V>,
}

impl<K, V> AbstractHashMap<K, V> for HashMap<K, V> {}


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
    pub height: u64,
}

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

    subscription_ids: Vec<SubscriptionID>,
    pub last_pulse_id: PulseID,

    subscriptions_map: HashMap<SubscriptionID, Subscription>,
    pulses_map: HashMap<PulseID, Pulse>,
    is_pulse_sent: HashMap<PulseID, bool>,

    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
}

impl PartialStorage for NebulaContract {
    const DATA_RANGE: std::ops::Range<usize> = 0..2000;
}

impl Sealed for NebulaContract {}

impl IsInitialized for NebulaContract {
    fn is_initialized(&self) -> bool {
        // self.is_initialized
        return true
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
    pub fn add_pulse(
        &mut self,
        new_pulse_id: PulseID,
        data_hash: Vec<u8>,
        block_number: u64,
    ) -> Result<(), NebulaError> {
        self.pulses_map.insert(
            new_pulse_id,
            Pulse {
                data_hash,
                height: block_number,
            },
        );

        let new_last_pulse_id = new_pulse_id + 1;
        self.last_pulse_id = new_last_pulse_id;

        Ok(())
    }

    const SERIALIZE_CONTEXT: u16 = 50;

    fn new_subscription_id(
        &self,
        node_id: &[u8],
    ) -> Result<SubscriptionID, Box<dyn std::error::Error>> {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        let context = Context::new(NebulaContract::SERIALIZE_CONTEXT);

        let ts = Timestamp::from_unix(
            &context,
            current_time.as_secs(),
            current_time.subsec_nanos(),
        );

        let uuid = Uuid::new_v1(ts, node_id).expect("failed to generate UUID");

        let sub_id = uuid.as_bytes();

        // an approach to avoid collision
        if self.subscriptions_map.contains_key(sub_id) {
            return self.new_subscription_id(node_id);
        }

        Ok(*sub_id)
    }

    pub fn subscribe(
        &mut self,
        sender: Pubkey,
        contract_address: Pubkey,
        min_confirmations: u8,
        reward: u64,
    ) -> Result<(), NebulaError> {
        let subscription = Subscription {
            sender,
            contract_address,
            min_confirmations,
            reward,
        };

        let serialized_subscription: Vec<u8> = bincode::serialize(&subscription).unwrap();

        let sub_id = match self.new_subscription_id(&serialized_subscription[0..6]) {
            Ok(val) => val,
            Err(_) => return Err(NebulaError::SubscribeFailed),
        };

        self.subscriptions_map.insert(sub_id, subscription);

        Ok(())
    }

    pub fn validate_data_provider(
        multisig_owner_keys: Vec<Pubkey>,
        data_provider: &Pubkey,
    ) -> Result<(), NebulaError> {
        for owner_key in multisig_owner_keys {
            if owner_key == *data_provider {
                return Ok(());
            }
        }

        Err(NebulaError::DataProviderForSendValueToSubsIsInvalid)
    }

    pub fn send_value_to_subs(
        &mut self,
        data_type: DataType,
        pulse_id: PulseID,
        subscription_id: SubscriptionID,
    ) -> Result<(), NebulaError> {
        // check is value has been sent
        // if self.subscriptions_map
        if let Some(pulse_sent) = self.is_pulse_sent.get(&pulse_id) {
            if *pulse_sent {
                return Err(NebulaError::SubscriberValueBeenSent);
            }
        }

        self.is_pulse_sent.insert(pulse_id, true);

        let subscription = match self.subscriptions_map.get(&subscription_id) {
            Some(v) => v,
            None => return Err(NebulaError::InvalidSubscriptionID),
        };

        // TODO - cross program invocation
        let destination_program_id = subscription.contract_address;

        Ok(())
    }
}
