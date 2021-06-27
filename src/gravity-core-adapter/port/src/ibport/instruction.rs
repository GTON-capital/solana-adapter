use solana_program::{
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
};
use spl_token::state::Multisig;
use std::convert::TryInto;
use std::ops::Range;
use std::slice::SliceIndex;
use std::mem::size_of;

use arrayref::{array_ref, array_refs};
// use hex;

use gravity_misc::model::{U256};
use gravity_misc::validation::{build_range_from_alloc, extract_from_range, retrieve_oracles};

use crate::ibport::allocs::allocation_by_instruction_index;
use crate::ibport::state::ForeignAddress;

use solana_gravity_contract::gravity::error::GravityError::InvalidInstruction;


pub enum IBPortContractInstruction {
    InitContract {
        nebula_address: Pubkey,
        token_address: Pubkey,
    },
    CreateTransferUnwrapRequest {
        amount: f64,
        receiver: ForeignAddress
    },
    AttachValue {
        // byte_data: AttachedData
        byte_data: Vec<u8>
    },
    TransferTokenOwnership {
        new_owner: Pubkey
    },
    TestCrossMint {
        receiver: Pubkey,
        amount: f64,
    },
    TestCrossBurn {
        receiver: Pubkey,
        amount: f64,
    }
}

impl IBPortContractInstruction {
    pub const PUBKEY_ALLOC: usize = 32;
    pub const DEST_AMOUNT_ALLOC: usize = 8;
    pub const FOREIGN_ADDRESS_ALLOC: usize = 32;
    pub const ATTACHED_DATA_ALLOC: usize = 64;

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            // InitContract
            0 => {
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);

                let (nebula_address, token_address) = (
                    Pubkey::new(&rest[ranges[0].clone()]),
                    Pubkey::new(&rest[ranges[1].clone()])
                );

                Self::InitContract {
                    nebula_address,
                    token_address,
                }
            }
            // CreateTransferUnwrapRequest
            1 => {
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);
                let (amount, receiver) = (
                    f64::from_le_bytes(*array_ref![rest[ranges[0].clone()], 0, 8]),
                    *array_ref![rest[ranges[1].clone()], 0, 32]
                );

                Self::CreateTransferUnwrapRequest {
                    amount,
                    receiver
                }
            }
            // AttachValue
            2 => {
                // let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                // let ranges = build_range_from_alloc(&allocs);
                // let byte_data = *array_ref![rest[ranges[0].clone()], 0, 80];
                let byte_data = rest.to_vec();

                Self::AttachValue { byte_data }
            }
            // TransferTokenOwnership
            3 => {
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);
                let new_owner = Pubkey::new(&rest[ranges[0].clone()]);

                Self::TransferTokenOwnership { new_owner }
            }
            // TestCrossMint
            4 => {
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);

                let receiver = Pubkey::new(&rest[ranges[0].clone()]);
                let amount = f64::from_le_bytes(*array_ref![rest[ranges[1].clone()], 0, 8]);

                Self::TestCrossMint { receiver, amount }
            }
            5 => {
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);

                let receiver = Pubkey::new(&rest[ranges[0].clone()]);
                let amount = f64::from_le_bytes(*array_ref![rest[ranges[1].clone()], 0, 8]);

                Self::TestCrossBurn { receiver, amount }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }
}

impl IBPortContractInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::AttachValue {
                ref byte_data,
            } => {
                let mut buf = byte_data.clone();
                buf.insert(0, 2);
                buf
            },
            _ => buf
        }
    }
}

pub fn attach_value(
    byte_data: &Vec<u8>,
    oracle: &Pubkey,
    subscriber_data_account: &Pubkey,
    target_program_id: &Pubkey, 
    token_program_id: &Pubkey, // actually spl_token::id()
    mint: &Pubkey, // actually the result of spl-token create-token (cli)
    recipient_account: &Pubkey,
    ibport_pda_account: &Pubkey,
    signer_pubkeys: &[&Pubkey],
) -> Result<Instruction, ProgramError> {
    let data = IBPortContractInstruction::AttachValue { byte_data: byte_data.clone()  }.pack();

    let mut accounts = Vec::with_capacity(6 + signer_pubkeys.len());
    accounts.push(AccountMeta::new_readonly(*oracle, true));
    accounts.push(AccountMeta::new(*subscriber_data_account, false));
    accounts.push(AccountMeta::new_readonly(*token_program_id, false));
    accounts.push(AccountMeta::new(*mint, false));
    accounts.push(AccountMeta::new(*recipient_account, false));
    accounts.push(AccountMeta::new_readonly(*ibport_pda_account, false));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *target_program_id,
        accounts,
        data,
    })
}