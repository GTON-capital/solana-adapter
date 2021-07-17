


use arrayref::array_ref;
use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};


use gravity_misc::validation::{
    build_range_from_alloc, extract_from_range, retrieve_oracles as retrieve_consuls,
};

use crate::gravity::allocs::allocation_by_instruction_index;
use crate::gravity::error::GravityError::InvalidInstruction;


pub enum GravityContractInstruction {
    InitContract {
        new_consuls: Vec<Pubkey>,
        current_round: u64,
        bft: u8,
    },
    UpdateConsuls {
        new_consuls: Vec<Pubkey>,
        current_round: u64,
    },
}

impl GravityContractInstruction {
    pub const BFT_ALLOC: usize = 1;
    pub const PUBKEY_ALLOC: usize = 32;
    pub const LAST_ROUND_ALLOC: usize = 8;

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => {
                let bft = extract_from_range(rest, 0..1, |x: &[u8]| {
                    u8::from_le_bytes(*array_ref![x, 0, 1])
                })?;
                let allocs = allocation_by_instruction_index((*tag).into(), Some(bft as usize))?;
                let ranges = build_range_from_alloc(&allocs);

                let current_round = extract_from_range(rest, ranges[1].clone(), |x: &[u8]| {
                    u64::from_le_bytes(*array_ref![x, 0, 8])
                })?;

                let initial_consuls = retrieve_consuls(rest, ranges[2].clone(), bft)?;

                Self::InitContract {
                    new_consuls: initial_consuls,
                    current_round,
                    bft,
                }
            }
            1 => {
                let bft = extract_from_range(rest, 0..1, |x: &[u8]| {
                    u8::from_le_bytes(*array_ref![x, 0, 1])
                })?;
                let allocs = allocation_by_instruction_index((*tag).into(), Some(bft as usize))?;
                let ranges = build_range_from_alloc(&allocs);

                let current_round = extract_from_range(rest, ranges[1].clone(), |x: &[u8]| {
                    u64::from_le_bytes(*array_ref![x, 0, 8])
                })?;
                let new_consuls = retrieve_consuls(rest, ranges[2].clone(), bft)?;

                Self::UpdateConsuls {
                    new_consuls,
                    current_round,
                }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
