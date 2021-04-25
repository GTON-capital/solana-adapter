
use std::fmt;
use std::collections::{
    HashMap
};

use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
// extern crate sha2;
// use sha2::Sha256;

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

#[derive(PartialEq, Debug, Clone)]
pub enum DataType {
    Int64,
    String,
    Bytes,
}

impl Default for DataType {
    fn default() -> Self { DataType::Int64 }
}

impl DataType {
    pub fn cast_from(i: u8) -> DataType {
        match i {
            0 => DataType::Int64,
            1 => DataType::String,
            2 => DataType::Bytes,
            _ => panic!("invalid data type")
        }
    }
}

pub type SubscriptionID<'a> = &'a [u8];
pub type PulseID = u64;

#[derive(PartialEq, Default, Debug, Clone)]
pub struct Subscription {
    pub address: Pubkey,
    pub contract_address: Pubkey,
    pub min_confirmations: i8,
    pub reward: i64, // should be 2^256
}

#[derive(PartialEq, Default, Debug, Clone)]
pub struct Pulse<'a> {
    pub data_hash: SubscriptionID<'a>,
    pub height: i128,
}

#[derive(PartialEq, Default, Debug, Clone)]
pub struct Oracle<'a, A> {
    pub address: A,
    pub is_online: bool,
    pub id_in_queue: SubscriptionID<'a>,
}


pub type NebulaQueue<T> = Vec<T>;

#[derive(PartialEq, Default, Debug, Clone)]
pub struct NebulaContract<'a> {
    rounds_dict: HashMap<PulseID, bool>,
    subscriptions_queue: NebulaQueue<SubscriptionID<'a>>,
    oracles: Vec<Pubkey>,

    bft: u8,
    gravity_contract: Pubkey,
    data_type: DataType,
    last_round: PulseID,

    subscription_ids: Vec<SubscriptionID<'a>>,
    last_pulse_id: PulseID,
    
    subscriptions_map: HashMap<SubscriptionID<'a>, Subscription>,
    pulses_map: HashMap<PulseID, Pulse<'a>>,
    is_pulse_sent: HashMap<
        PulseID,
        HashMap<SubscriptionID<'a>, bool>
    >,

    is_initialized: bool,
}

// impl fmt::Display for NebulaContract {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "is_initialized: {:};
//              initializer_pubkey: {:};
//              consuls: {:?};
//              bft: {:};
//              last_round: {:}",
//             self.is_initialized, self.initializer_pubkey, self.consuls, self.bft, self.last_round
//         )
//     }
// }

// pub trait PartialStorage {
//     const DATA_RANGE: std::ops::Range<usize>;

//     fn store_at<'a>(raw_data: &'a [u8]) -> &'a [u8] {
//         return &raw_data[Self::DATA_RANGE]
//     }
// }

// impl PartialStorage for GravityContract {
//     const DATA_RANGE: std::ops::Range<usize> = 0..138;
// }

impl<'a> Sealed for NebulaContract<'a> {}

impl<'a> IsInitialized for NebulaContract<'a> {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

// impl Pack for GravityContract {
//     const LEN: usize = 138;
    
//     fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
//         let src = array_ref![src, 0, GravityContract::LEN];
//         let (is_initialized, initializer_pubkey, bft, consuls, last_round) =
//             array_refs![src, 1, 32, 1, 32 * 3, 8];
//         let is_initialized = is_initialized[0] != 0;

//         Ok(GravityContract {
//             is_initialized,
//             initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
//             bft: u8::from_le_bytes(*bft),
//             consuls: vec![
//                 Pubkey::new_from_array(*array_ref![consuls[0..32], 0, 32]),
//                 Pubkey::new_from_array(*array_ref![consuls[32..64], 0, 32]),
//                 Pubkey::new_from_array(*array_ref![consuls[64..96], 0, 32]),
//             ],
//             last_round: u64::from_le_bytes(*last_round),
//         })
//     }

//     fn pack_into_slice(&self, dst: &mut [u8]) {
//         let dst = array_mut_ref![dst, 0, GravityContract::LEN];
//         let (is_initialized_dst, initializer_pubkey_dst, bft_dst, consuls_dst, last_round_dst) =
//             mut_array_refs![dst, 1, 32, 1, 32 * 3, 8];

//         let GravityContract {
//             is_initialized,
//             initializer_pubkey,
//             bft,
//             consuls,
//             last_round,
//         } = self;

//         is_initialized_dst[0] = *is_initialized as u8;
//         initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
//         bft_dst[0] = *bft as u8;

//         let consuls_copy = consuls.clone();
//         consuls_dst.copy_from_slice(
//             consuls_copy
//                 .iter()
//                 .fold(vec![], |acc, x| vec![acc, x.to_bytes().to_vec()].concat())
//                 .as_slice(),
//         );

//         *last_round_dst = last_round.to_le_bytes();
//     }
// }

