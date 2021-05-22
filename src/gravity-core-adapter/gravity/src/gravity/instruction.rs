
use std::convert::TryInto;
use std::slice::SliceIndex;

use solana_program::{
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use arrayref::array_ref;
use spl_token::state::Multisig;

use gravity_misc::validation::{extract_from_range, build_range_from_alloc, retrieve_oracles as retrieve_consuls};

use crate::gravity::state::GravityContract;
use crate::gravity::error::GravityError::InvalidInstruction;
use crate::gravity::allocs::allocation_by_instruction_index;


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

impl<'a> GravityContractInstruction {
    pub const BFT_ALLOC: usize = 1;
    pub const PUBKEY_ALLOC: usize = 32;
    pub const LAST_ROUND_ALLOC: usize = 8;

    pub const BFT_RANGE: std::ops::Range<usize> = 0..Self::BFT_ALLOC;

    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &'a [u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => {
                let bft = extract_from_range(rest, 0..1, |x: &[u8]| {
                    u8::from_le_bytes(*array_ref![x, 0, 1])
                })?;
                let allocs =
                    allocation_by_instruction_index((*tag).into(), Some(bft as usize))?;
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
                let allocs =
                    allocation_by_instruction_index((*tag).into(), Some(bft as usize))?;
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

    fn unpack_bft(input: &'a [u8]) -> Result<u8, ProgramError> {
        Ok(input
            .get(Self::BFT_RANGE)
            .and_then(|slice| slice.try_into().ok())
            .map(u8::from_le_bytes)
            .ok_or(InvalidInstruction)?)
    }

    fn unpack_consuls(input: &'a [u8], dst: &mut Vec<Pubkey>) -> Result<(), ProgramError> {
        let bft: u8 = Self::unpack_bft(input)?;

        let range_start = Self::BFT_ALLOC;
        let range_end = range_start + 32 * bft as usize;
        let consuls_slice = input
            .get(range_start..range_end)
            .ok_or(InvalidInstruction)?;

        // assert!(consuls_slice.len() == 3 * 32);

        let address_alloc: usize = 32;

        for i in 0..bft as usize {
            let pubky = Pubkey::new(
                consuls_slice
                    .get(i * address_alloc..(i + 1) * address_alloc)
                    .ok_or(InvalidInstruction)?,
            );
            dst.push(pubky);
        }

        Ok(())
    }

    /// Round is considered as first argument and as u256 data type
    fn unpack_round(bft: u8, input: &[u8]) -> Result<u64, ProgramError> {
        let start_offset = Self::BFT_ALLOC + (Self::PUBKEY_ALLOC * bft as usize);
        Ok(input
            .get(start_offset..start_offset + Self::LAST_ROUND_ALLOC)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?)
    }
}
