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
use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum RequestStatus {
    None,
    New,
    Completed,
}

impl Default for RequestStatus {
    fn default() -> Self {
        RequestStatus::None
    }
}

pub type RequestAmount = [u8; 32];
pub type RequestID = u64;

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
pub struct Request {
    pub origin_address: String,
    pub target_address: String,
    pub amount: RequestAmount,
    pub status: RequestStatus,
}

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
pub struct LUPortContract {
    pub nebula_address: Pubkey,
    pub token_address: Pubkey,

    requests_map: HashMap<RequestID, Request>,
    requests_queue: Vec<Request>,

    is_initialized: bool,
}


impl PartialStorage for LUPortContract {
    const DATA_RANGE: std::ops::Range<usize> = 0..2000;
}

impl Sealed for LUPortContract {}

impl IsInitialized for LUPortContract {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for LUPortContract {
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
