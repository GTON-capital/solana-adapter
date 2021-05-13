use solana_program::{
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use spl_token::state::Multisig;
use std::convert::TryInto;
use std::ops::Range;
use std::str;
use std::slice::SliceIndex;

// use uuid::{Builder as UUIDBuilder, Uuid as UUID};
// use std::bytes::Bytes;

use arrayref::{array_ref, array_refs};
// use hex;

use crate::gravity::misc::{build_range_from_alloc, extract_from_range};

use crate::nebula::state::{DataType, PulseID, SubscriptionID};
use crate::subscriber::luport::state::RequestAmount;

// use hex;
// use crate::state::misc::WrappedResult;
use crate::gravity::error::GravityError::InvalidInstruction;
use crate::gravity::misc::map_to_address;

pub enum LUPortContractInstruction {
    InitContract {
        nebula_address: Pubkey,
        token_address: Pubkey,
    },
    AttachValue {
        byte_value: [u8; 32],
    },
    CreateTransferWrapRequest {
        amount: RequestAmount,
        receiver: String,
    },
    // TransferOwnership {
    //     new_owner: Pubkey,
    // },
}

impl LUPortContractInstruction {
    const PUBKEY_ALLOC: usize = 32;
    const REQUEST_AMOUNT_ALLOC: usize = 32;
    const DATA_HASH_ALLOC: usize = 32;

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            // InitContract
            0 => {
                let allocs = vec![Self::PUBKEY_ALLOC, Self::PUBKEY_ALLOC];

                let ranges = build_range_from_alloc(&allocs);
                let (nebula_range, token_range) = (ranges[0].clone(), ranges[1].clone());

                let nebula_address = extract_from_range(rest, nebula_range, map_to_address)?;
                let token_address = extract_from_range(rest, token_range, map_to_address)?;

                Self::InitContract {
                    nebula_address,
                    token_address,
                }
            },
            // AttachValue
            1 => {
                Self::AttachValue {
                    byte_value: *array_ref!(rest[0..Self::DATA_HASH_ALLOC], 0, 32)
                }
            },
            // CreateTransferWrapRequest
            2 => {
                let request_amount = *array_ref!(rest[0..Self::REQUEST_AMOUNT_ALLOC], 0, 32);
                let request_recipient= str::from_utf8(&rest[Self::REQUEST_AMOUNT_ALLOC..]).unwrap();

                Self::CreateTransferWrapRequest {
                    amount: request_amount, receiver: String::from(request_recipient)
                }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
