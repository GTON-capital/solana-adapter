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
use gravity_misc::model::{AbstractHashMap, HashMap};
// use std::collections::BTreeMap as HashMap;
// use std::collections::HashMap;


// use bincode;
use arrayref::array_ref;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

use crate::ibport::error::PortError;

use gravity_misc::model::{U256, new_uuid};

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

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone, Default)]
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

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Default, Debug, Clone)]
pub struct IBPortContract {
    pub nebula_address: Pubkey,
    pub token_address: Pubkey,
    pub initializer_pubkey: Pubkey,

    pub swap_status: HashMap<[u8; 16], RequestStatus>,
    pub requests: HashMap<[u8; 16], UnwrapRequest>,
    // pub requests_queue: RequestsQueue<u8>,
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
    const DATA_RANGE: std::ops::Range<usize> = 0..777;
}

impl Sealed for IBPortContract {}

impl IsInitialized for IBPortContract {
    fn is_initialized(&self) -> bool {
        return true;
    }
}

impl Pack for IBPortContract {
    const LEN: usize = 777;

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

    pub fn attach_data<F: Fn(u64, &AccountInfo) -> ProgramResult>(&mut self, byte_data: &Vec<u8>, mint_callback_fn: F) -> ProgramResult {
        // let byte_data = byte_data.to_vec();
        let owner_bytes: [u8; 32] = [1; 32];
        let owner = &Pubkey::new(&owner_bytes);
        let mut pos = 0;
        
        /**
         * We use iterative approach
         * in order to process all the requests in one invocation
         */
        while pos < byte_data.len() {
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
                let amount = u64::from_le_bytes(*raw_amount);
                pos += 8;

                let receiver = array_ref![byte_data, pos, 32];
                pos += 32;

                let recipient = &Pubkey::new(&*receiver);
                let mut lamports: u64 = 0;
                let recipient_account = AccountInfo::new(
                    recipient,
                    false, // is_signer
                    true, // is_writable
                    &mut lamports, // lamports
                    &mut [], // data
                    owner, // owner
                    false, // executable,
                    0, // rent_epoch
                );

                match mint_callback_fn(amount, &recipient_account) {
                    Ok(_) => {
                        self.swap_status.insert(*swap_id, RequestStatus::Success);
                        return Ok(());
                    },
                    Err(err) => {
                        return Err(err);
                    }
                };
                continue;
                // return Ok(())
            }
        }
        

        Ok(())
    }

    pub fn create_transfer_unwrap_request(&mut self, amount: u64, sender_data_account: &Pubkey, receiver: &ForeignAddress) -> Result<(), PortError>  {
        // uint id = uint(keccak256(abi.encodePacked(msg.sender, receiver, block.number, amount)));
        let id = new_uuid(&receiver[0..6]);
        self.validate_requests_count()?;

        self.requests.insert(*id.as_bytes(), UnwrapRequest {
            destination_address: *receiver,
            origin_address: *sender_data_account,
            amount
        });
        self.swap_status.insert(*id.as_bytes(), RequestStatus::New);

        msg!("swap len: {:} \n", self.swap_status.len());
        msg!("requests len: {:} \n", self.requests.len());

        Ok(())
    }


}