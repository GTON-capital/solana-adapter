
// use std::collections::HashMap;

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