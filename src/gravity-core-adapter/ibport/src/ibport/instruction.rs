use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::mem::size_of;
use arrayref::array_ref;

use gravity_misc::validation::{build_range_from_alloc, extract_from_range, retrieve_oracles};

use crate::ibport::allocs::allocation_by_instruction_index;
use gravity_misc::ports::state::ForeignAddress;
use gravity_misc::ports::instruction::ATTACH_VALUE_INSTRUCTION_INDEX;

use solana_gravity_contract::gravity::error::GravityError::InvalidInstruction;


pub enum IBPortContractInstruction {
    InitContract {
        nebula_address: Pubkey,
        token_address: Pubkey,
        oracles: Vec<Pubkey>,
    },
    CreateTransferUnwrapRequest {
        request_id: [u8; 16],
        amount: f64,
        receiver: ForeignAddress,
    },
    AttachValue {
        byte_data: Vec<u8>,
    },
    ConfirmDestinationChainRequest {
        byte_data: Vec<u8>,
    },
    TransferTokenOwnership {
        new_authority: Pubkey,
        new_token: Pubkey,
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

                if rest.len() == 64 {
                    Self::InitContract {
                        nebula_address,
                        token_address,
                        oracles: Vec::new(),
                    }
                } else {
                    let oracles_bft = extract_from_range(rest, 64..65, |x: &[u8]| {
                        u8::from_le_bytes(*array_ref![x, 0, 1])
                    })?;
                    let oracles = retrieve_oracles(rest, 65..65 + (oracles_bft as usize * 32), oracles_bft)?;

                    Self::InitContract {
                        nebula_address,
                        token_address,
                        oracles,
                    }
                }
            }
            // CreateTransferUnwrapRequest
            1 => {
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);
                let (amount, receiver, request_id) = (
                    f64::from_le_bytes(*array_ref![rest[ranges[0].clone()], 0, 8]),
                    *array_ref![rest[ranges[1].clone()], 0, 32],
                    *array_ref![rest[ranges[2].clone()], 0, 16],
                );

                Self::CreateTransferUnwrapRequest {
                    request_id,
                    amount,
                    receiver
                }
            }
            // AttachValue
            ATTACH_VALUE_INSTRUCTION_INDEX => {
                let byte_data = rest.to_vec();

                Self::AttachValue { byte_data }
            }
            // ConfirmDestinationChainRequest
            3 => {
                let byte_data = rest.to_vec();

                Self::ConfirmDestinationChainRequest { byte_data }
            }
            4 => {
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);

                let (new_authority, new_token) = (
                    Pubkey::new(&rest[ranges[0].clone()]),
                    Pubkey::new(&rest[ranges[1].clone()])
                );

                Self::TransferTokenOwnership { new_authority, new_token }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }
}

impl IBPortContractInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::AttachValue {
                ref byte_data,
            } => {
                let mut buf = byte_data.clone();
                buf.insert(0, *ATTACH_VALUE_INSTRUCTION_INDEX);
                buf
            },
            _ => buf
        }
    }
}
