use std::fmt;
use std::marker::PhantomData;

use std::time::{Duration, SystemTime};
use std::ops::Fn;

use solana_program::{
    msg,
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use spl_token::instruction::mint_to_checked;

use solana_gravity_contract::gravity::state::PartialStorage;
use gravity_misc::model::{AbstractRecordHandler, RecordHandler};

use arrayref::array_ref;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

use crate::ibport::error::PortError;

use gravity_misc::model::{U256, new_uuid};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum RequestStatus {
    None,
    New,
    Rejected,
    Success,
     
}

impl Default for RequestStatus {
    fn default() -> Self {
        RequestStatus::None
    }
}

// #[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone)]
pub type ForeignAddress = [u8; 32];

// by default:
// 16 bytes - swap id
// 32 bytes - amount
// 32 bytes - receiver

// #[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone)]
// pub type AttachedData = [u8; 80];

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Default, Copy)]
pub struct UnwrapRequest {
    pub destination_address: ForeignAddress,
    pub origin_address: Pubkey,
    pub amount: u64
}

pub type RequestsQueue<T> = Vec<T>;

trait RequestCountConstrained {
    const MAX_IDLE_REQUESTS_COUNT: usize;

    fn unprocessed_requests_limit() -> usize {
        Self::MAX_IDLE_REQUESTS_COUNT
    }                                                                                                                                                                                                             

    fn count_constrained_entities(&self) -> Vec<usize>;

    fn count_is_below_limit(&self) -> bool {
        let entities = self.count_constrained_entities();

        for entity_len in entities {
            if entity_len >= Self::unprocessed_requests_limit() {
                return false
            }
        }
        return true
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Default, Debug, Clone)]
pub struct IBPortContract {
    pub nebula_address: Pubkey, // distinct nebula address (not nebula data account)
    pub token_address: Pubkey, // common token info, (result of spl-token create-token or as it so called - 'the mint')
    pub initializer_pubkey: Pubkey,
    pub oracles: Vec<Pubkey>,

    pub swap_status: RecordHandler<[u8; 16], RequestStatus>,
    pub requests: RecordHandler<[u8; 16], UnwrapRequest>,

    pub is_state_initialized: bool,
}

impl RequestCountConstrained for IBPortContract {
    const MAX_IDLE_REQUESTS_COUNT: usize = 7;

    fn count_constrained_entities(&self) -> Vec<usize> {
        vec![
            self.swap_status.len()
        ]
    }
} 

impl PartialStorage for IBPortContract {
    const DATA_RANGE: std::ops::Range<usize> = 0..1500;
}

impl Sealed for IBPortContract {} 

impl IsInitialized for IBPortContract {
    fn is_initialized(&self) -> bool {
        self.is_state_initialized
    }
}


impl Pack for IBPortContract {
    const LEN: usize = 1500;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut mut_src: &[u8] = src;
        Self::deserialize(&mut mut_src).map_err(|err| {
            msg!(
                "Error: failed to deserialize IBPortContract instruction: {}",
                err
            );
            ProgramError::InvalidInstructionData
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let data = self.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }
}

pub struct PortOperation<'a> {
    pub action: u8,
    pub swap_id: &'a [u8; 16],
    pub amount: &'a [u8; 8],
    // receiver: &'a [u8; 32],
    pub receiver: &'a ForeignAddress,
}

impl<'a> PortOperation<'a> {
    pub fn decimals() -> u8 {
        8
    }

    pub fn amount_to_f64(&self) -> f64 {
        let raw_amount = array_ref![self.amount, 0, 8];
        f64::from_le_bytes(*raw_amount)
    }

    pub fn amount_to_u64(&self) -> u64 {
        let decimals = Self::decimals();
        spl_token::ui_amount_to_amount(self.amount_to_f64(), decimals)
    }
}


impl IBPortContract {

    fn validate_requests_count(&self) -> Result<(), PortError> {
        if !self.count_is_below_limit() {
            return Err(PortError::TransferRequestsCountLimit);
        }
        Ok(())
    }

    pub fn unpack_byte_array(byte_data: &Vec<u8>) -> Result<PortOperation, ProgramError> {
        if byte_data.len() < 57 {
            return Err(PortError::ByteArrayUnpackFailed.into());
        }

        let mut pos = 0;
        let action = byte_data[pos];
        pos += 1;

        let swap_id = array_ref![byte_data, pos, 16];
        pos += 16;
        
        let raw_amount = array_ref![byte_data, pos, 8];
        pos += 8;

        let receiver = array_ref![byte_data, pos, 32];

        return Ok(PortOperation {
            action,
            swap_id,
            amount: raw_amount,
            receiver
        });
    }

    pub fn attach_data<'a>(&mut self, byte_data: &'a Vec<u8>, input_pubkey: &'a Pubkey, input_amount: &'a mut u64) -> Result<(), ProgramError> {
        let mut pos = 0;
        let action = byte_data[pos];
        pos += 1;

        let port_operation = Self::unpack_byte_array(byte_data)?;

        if "m" != std::str::from_utf8(&[action]).unwrap() {
            return Err(PortError::InvalidDataOnAttach.into());
        }

        let swap_status = self.swap_status.get(port_operation.swap_id);

        if swap_status.is_some() {
            return Err(PortError::InvalidRequestStatus.into());
        }

        if input_pubkey.to_bytes() != *port_operation.receiver {
            return Err(PortError::ErrorOnReceiverUnpack.into());
        }
        
        *input_amount = port_operation.amount_to_u64();

        self.swap_status.insert(*port_operation.swap_id, RequestStatus::Success);

        Ok(())
    }

    pub fn drop_processed_request(&mut self, byte_array: &Vec<u8>) -> Result<(), ProgramError>  {
        let port_operation = Self::unpack_byte_array(byte_array)?;
        let request_id = port_operation.swap_id;

        let (request_drop_res, swap_status_drop_res) = (
            self.requests.drop(request_id),
            self.swap_status.drop(request_id)
        );

        if request_drop_res.is_none() || swap_status_drop_res.is_none() {
            return Err(PortError::RequestIDForConfirmationIsInvalid.into());
        }

        let (request_drop_res, swap_status_drop_res) = (request_drop_res.unwrap(), swap_status_drop_res.unwrap());

        if request_drop_res.destination_address != *port_operation.receiver {
            return Err(PortError::RequestReceiverMismatch.into());
        }

        if swap_status_drop_res != RequestStatus::New {
            return Err(PortError::RequestStatusMismatch.into());
        }
        
        let port_amount = port_operation.amount_to_u64();

        if request_drop_res.amount != port_amount {
            return Err(PortError::RequestAmountMismatch.into());
        }

        Ok(())
    }

    pub fn create_transfer_unwrap_request(&mut self, record_id: &[u8; 16], amount: u64, sender_data_account: &Pubkey, receiver: &ForeignAddress) -> Result<(), PortError>  {
        self.validate_requests_count()?;

        if self.requests.contains_key(record_id) {
            return Err(PortError::RequestIDIsAlreadyBeingProcessed.into());
        }

        self.requests.insert(*record_id, UnwrapRequest {
            destination_address: *receiver,
            origin_address: *sender_data_account,
            amount
        });
        self.swap_status.insert(*record_id, RequestStatus::New);

        Ok(())
    }


}