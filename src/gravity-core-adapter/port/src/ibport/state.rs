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
        for x in entities {
            if x >= Self::unprocessed_requests_limit() {
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

    pub swap_status: RecordHandler<[u8; 16], RequestStatus>,
    pub requests: RecordHandler<[u8; 16], UnwrapRequest>,
}

impl RequestCountConstrained for IBPortContract {
    const MAX_IDLE_REQUESTS_COUNT: usize = 15;

    fn count_constrained_entities(&self) -> Vec<usize> {
        let res = vec![
            self.swap_status.len()
        ];
        res
    }
} 

impl PartialStorage for IBPortContract {
    const DATA_RANGE: std::ops::Range<usize> = 0..1000;
}

impl Sealed for IBPortContract {}

impl IsInitialized for IBPortContract {
    fn is_initialized(&self) -> bool {
        return true;
    }
}


impl Pack for IBPortContract {
    const LEN: usize = 1000;

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


impl IBPortContract {

    fn validate_requests_count(&self) -> Result<(), PortError> {
        if !self.count_is_below_limit() {
            return Err(PortError::TransferRequestsCountLimit);
        }
        Ok(())
    }

    pub fn attach_data<'a>(&mut self, byte_data: &'a Vec<u8>, input_pubkey: &'a Pubkey, input_amount: &'a mut u64) -> Result<(), ProgramError> {
        let mut pos = 0;
        let action = byte_data[pos];
        pos += 1;

        if "m" == std::str::from_utf8(&[action]).unwrap() {
            let swap_id = array_ref![byte_data, pos, 16];

            pos += 16;
            
            let swap_status = self.swap_status.get(swap_id);

            if swap_status.is_some() {
                return Err(PortError::InvalidRequestStatus.into());
            }

            let raw_amount = array_ref![byte_data, pos, 8];
            let ui_amount = f64::from_le_bytes(*raw_amount);

            let decimals = 8;
            let amount = spl_token::ui_amount_to_amount(ui_amount, decimals);

            pos += 8;

            let receiver = array_ref![byte_data, pos, 32];

            if input_pubkey.to_bytes() != *receiver {
                return Err(PortError::ErrorOnReceiverUnpack.into());
            }
            
            *input_amount = amount;

            return Ok(());
        }

        Err(PortError::InvalidDataOnAttach.into())
    }

    pub fn create_transfer_unwrap_request(&mut self, amount: u64, sender_data_account: &Pubkey, receiver: &ForeignAddress) -> Result<(), PortError>  {
        let mut record_id: [u8; 16] = Default::default();

        // record_id.copy_from_slice(&sender_data_account.to_bytes()[0..16]);
        record_id.copy_from_slice(&receiver[0..16]);

        self.requests.insert(record_id, UnwrapRequest {
            destination_address: *receiver,
            origin_address: *sender_data_account,
            amount
        });
        self.swap_status.insert(record_id, RequestStatus::New);

        msg!("swap len: {:} \n", self.swap_status.len());
        msg!("requests len: {:} \n", self.requests.len());

        Ok(())
    }


}