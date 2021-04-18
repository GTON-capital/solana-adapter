use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::convert::TryInto;
use std::slice::{SliceIndex};

use crate::error::GravityError::InvalidInstruction;


mod utils {
    use super::*;

    pub fn extract_from_range<'a, T: std::convert::From<&'a[u8]>, U, F: FnOnce(T) -> U>(
        input: &'a[u8],
        index: std::ops::Range<usize>,
        f: F
    ) -> Result<U, ProgramError> {
        let res = input
            .get(index)
            .and_then(|slice| slice.try_into().ok())
            .map(f)
            .ok_or(InvalidInstruction)?;
        Ok(res)
    }
}

pub enum GravityContractInstruction<'a> {
    GetConsuls,
    GetConsulsByRoundId {
        current_round: u64
    },
    UpdateConsuls {
        new_consuls: &'a[&'a Pubkey],
        current_round: u64
    }
}


impl<'a> GravityContractInstruction<'a> {
    const bft_alloc: usize = 8;
    const last_round_alloc: usize = 64;

    const bft_range: std::ops::Range<usize> = 0..Self::bft_alloc;
    const last_round_range: std::ops::Range<usize> = Self::bft_alloc..Self::last_round_alloc;

    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &'a[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::GetConsuls,
            // 
            // Args:
            // [u8 - instruction, u256 as u8 array - input round]
            //
            1 => Self::GetConsulsByRoundId {
                current_round: Self::unpack_round(rest)?,
            },
            // 
            // Args:
            // [u8 - instruction, u8 - bft value, bft value * address as u8 array(concated)]
            //
            2 => {
                let new_consuls: Result<&'a[&'a Pubkey], ProgramError> = Self::unpack_consuls(input);

                Self::UpdateConsuls {
                    current_round: Self::unpack_round(input)?,
                    new_consuls: new_consuls?
                }
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_consuls(input: &'a[u8]) -> Result<&'a[&Pubkey], ProgramError> {
        let bft: u8 = input
            .get(Self::bft_range)
            .and_then(|slice| slice.try_into().ok())
            .map(u8::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        let last_round: u64 = input
            .get(Self::last_round_range)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;

        let range_start = Self::bft_alloc + Self::last_round_alloc;
        let range_end = range_start * bft as usize;
        let consuls_slice = input
            .get(range_start..range_end)
            .ok_or(InvalidInstruction)?;
        
        let mut result: Vec<&Pubkey> = Vec::new();
        let address_alloc: usize = 32;

        // Pubkey::from_str(s: &str)
        for i in 0..bft as usize {
            let slice = consuls_slice.get(i * address_alloc..(i + 1) * address_alloc).ok_or(InvalidInstruction)?;
            let pubky = Pubkey::new(slice);
            result.push(&pubky);
        }

        Ok(result.as_slice())
    }

    /// Round is considered as first argument and as u256 data type
    fn unpack_round(input: &[u8]) -> Result<u64, ProgramError> {
        Ok(input
            .get(GravityContractInstruction::last_round_range)
            .and_then(|slice| slice.try_into().ok())
            .map(u8::from_le_bytes)
            .ok_or(InvalidInstruction)? as u64)
    }
}