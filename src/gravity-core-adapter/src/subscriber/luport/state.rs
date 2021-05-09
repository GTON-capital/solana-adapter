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

pub enum RequestStatus {
    None,
    New,
    Completed,
}

pub type RequestAmount = [u8; 32];
pub type RequestID = u64;

pub struct Request {
    pub origin_address: String,
    pub target_address: String,
    pub amount: RequestAmount,
    pub status: RequestStatus,
}

pub struct LUPortContract {
    pub nebula_address: Pubkey,
    pub token_address: Pubkey,

    requests_map: HashMap<RequestID, Request>,
    requests_queue: Vec<Request>,
}
