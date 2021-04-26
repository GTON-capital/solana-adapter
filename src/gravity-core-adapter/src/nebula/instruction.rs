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
use std::slice::SliceIndex;

use arrayref::{array_ref, array_refs};
// use hex;

use crate::gravity::misc::extract_from_range;
use crate::gravity::state::GravityContract;
use crate::nebula::state::DataType;

use crate::nebula::state::PulseID;

// use hex;
// use crate::state::misc::WrappedResult;
use crate::gravity::error::GravityError::InvalidInstruction;

mod utils {
    use super::*;

    pub fn extract_from_range<'a, T: std::convert::From<&'a [u8]>, U, F: FnOnce(T) -> U>(
        input: &'a [u8],
        index: std::ops::Range<usize>,
        f: F,
    ) -> Result<U, ProgramError> {
        let res = input
            .get(index)
            .and_then(|slice| slice.try_into().ok())
            .map(f)
            .ok_or(InvalidInstruction)?;
        Ok(res)
    }
}

pub enum NebulaContractInstruction<'a> {
    InitContract {
        nebula_data_type: DataType,
        gravity_contract_program_id: Pubkey,
        initial_oracles: Vec<Pubkey>,
        oracles_bft: u8,
    },
    UpdateOracles {
        new_oracles: Vec<Pubkey>,
        new_round: PulseID,
    },
    SendHashValue {
        data_hash: &'a [u8],
    },
    SendValueToSubs {
        data_type: DataType,
        pulse_id: PulseID,
        subscription_id: &'a [u8],
    },
}

fn build_range_from_alloc(allocs: &Vec<usize>) -> Vec<Range<usize>> {
    let mut res = vec![];

    let mut i = 0;
    let n = allocs.len();
    let mut start_index = 0;

    while i < n {
        let current = allocs[i];
        if i == 0 {
            let alloc = 0..current;
            res.push(alloc);
            start_index = current;
            i += 1;
            continue;
        }

        let alloc = start_index..start_index + current;
        res.push(alloc);
        start_index += current;

        i += 1;
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_from_range_alloc() {
        let allocs = vec![1, 1, 32 * 3, 8];

        let ranges = build_range_from_alloc(&allocs);

        assert_eq!(ranges.len(), allocs.len());
        assert_eq!(ranges[0], 0..1);
        assert_eq!(ranges[1], 1..1 + 1);
        assert_eq!(ranges[2], 1 + 1..1 + 1 + (32 * 3));
        assert_eq!(ranges[3], 1 + 1 + (32 * 3)..1 + 1 + (32 * 3) + 8);
    }

    #[test]
    fn test_bft_extraction() {
        let input: [u8; 1] = u8::to_le_bytes(3);

        let extracted = extract_from_range(&input, 0..1, |x: &[u8]| {
            u8::from_le_bytes(*array_ref![x, 0, 1])
        });
    }
}

impl<'a> NebulaContractInstruction<'a> {
    const BFT_ALLOC: usize = 1;
    const DATA_TYPE_ALLOC_RANGE: usize = 1;
    const PUBKEY_ALLOC: usize = 32;
    const ROUND_ALLOC: usize = 8;

    // pub fn match_instruction_byte_order(instruction: Self)

    pub fn unpack(input: &'a [u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => {
                let oracles_bft = extract_from_range(rest, 0..1, |x: &[u8]| {
                    u8::from_le_bytes(*array_ref![x, 0, 1])
                })?;
                let allocs = vec![
                    Self::BFT_ALLOC,
                    Self::DATA_TYPE_ALLOC_RANGE,
                    Self::PUBKEY_ALLOC,
                    Self::PUBKEY_ALLOC * oracles_bft as usize,
                ];
                let ranges = build_range_from_alloc(&allocs);
                let nebula_data_type = DataType::cast_from(extract_from_range(
                    rest,
                    ranges[1].clone(),
                    |x: &[u8]| u8::from_le_bytes(*array_ref![x, 0, 1]),
                )?);

                let gravity_contract_program_id =
                    extract_from_range(rest, ranges[2].clone(), |x| Pubkey::new(x))?;
                let initial_oracles = Self::retrieve_oracles(rest, ranges[3].clone(), oracles_bft)?;

                Self::InitContract {
                    nebula_data_type,
                    gravity_contract_program_id,
                    initial_oracles,
                    oracles_bft,
                }
            }
            1 => {
                let bft = extract_from_range(rest, 0..1, |x: &[u8]| {
                    u8::from_le_bytes(*array_ref![x, 0, 1])
                })?;
                let allocs = vec![
                    Self::BFT_ALLOC,
                    Self::PUBKEY_ALLOC * bft as usize,
                    Self::ROUND_ALLOC,
                ];
                let ranges = build_range_from_alloc(&allocs);

                let new_oracles = Self::retrieve_oracles(rest, ranges[1].clone(), bft)?;
                let new_round = extract_from_range(rest, ranges[2].clone(), |x: &[u8]| {
                    PulseID::from_le_bytes(*array_ref![x, 0, 8])
                })?;

                Self::UpdateOracles {
                    new_oracles,
                    new_round,
                }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn retrieve_oracles(
        bytes: &[u8],
        range: std::ops::Range<usize>,
        bft: u8,
    ) -> Result<Vec<Pubkey>, ProgramError> {
        extract_from_range(bytes, range, |x: &[u8]| {
            let consuls = array_ref![x, 32 * bft as usize, 8];
            let mut result = vec![];

            for i in 0..bft {
                let i = i as usize;
                result.push(Pubkey::new_from_array(*array_ref![
                    consuls[i * 32..(i + 1) * 32],
                    0,
                    32
                ]));
            }

            result
        })
    }

}
