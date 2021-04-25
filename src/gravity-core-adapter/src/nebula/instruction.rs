
use solana_program::{
    msg,
    account_info::AccountInfo,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use spl_token::state::Multisig;
use std::convert::TryInto;
use std::slice::SliceIndex;
use std::ops::Range;

use arrayref::{array_ref, array_refs};
// use hex;

use crate::gravity::misc::extract_from_range;
use crate::nebula::state::DataType;
use crate::gravity::state::GravityContract;



// use hex;
// use crate::state::misc::WrappedResult;
use crate::error::GravityError::InvalidInstruction;

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
        oracles_bft: u8
    },
    UpdateOracles {
        new_oracles: Vec<Pubkey>,
        new_round: u64,
    },
    SendHashValue {
        data_hash: &'a [u8]
    },
    SendValueToSubs {
        data_type: DataType,
        pulse_id: u64,
        subscription_id: &'a [u8]
    }
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
            continue
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
        let allocs = vec![
            1,
            1,
            32 * 3,
            8
        ];

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

        let extracted = extract_from_range(&input, 0..1, |x: &[u8]| { u8::from_le_bytes(*array_ref![x, 0, 1]) });
    }
}

impl<'a> NebulaContractInstruction<'a> {
    const BFT_ALLOC: usize = 1;
    const DATA_TYPE_ALLOC_RANGE: usize = 1;
    const PUBKEY_ALLOC: usize = 32;
    const LAST_ROUND_ALLOC: usize = 8;

    // pub fn match_instruction_byte_order(instruction: Self)



    pub fn unpack(input: &'a [u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => {
                let bft = extract_from_range(rest, 0..1, |x: &[u8]| { u8::from_le_bytes(*array_ref![x, 0, 1]) })?;
                let allocs = vec![
                    Self::BFT_ALLOC,
                    Self::DATA_TYPE_ALLOC_RANGE,
                    Self::PUBKEY_ALLOC,
                    Self::PUBKEY_ALLOC * bft as usize,
                ];
                let ranges = build_range_from_alloc(&allocs);
                let data_type = DataType::cast_from(
                    extract_from_range(rest, ranges[1].clone(), |x: &[u8]| { u8::from_le_bytes(*array_ref![x, 0, 1]) })?
                );
                let gravity_contract_program_id = extract_from_range(rest, ranges[2].clone(), |x| { Pubkey::new(x) })?;
                let initial_oracles = extract_from_range(rest, ranges[3].clone(), |x: &[u8]| {
                    let consuls = array_ref![x, 32 * bft as usize, 8];

                    vec![
                        Pubkey::new_from_array(*array_ref![consuls[0..32], 0, 32]),
                        Pubkey::new_from_array(*array_ref![consuls[32..64], 0, 32]),
                        Pubkey::new_from_array(*array_ref![consuls[64..96], 0, 32]),
                    ]
                })?;

                Self::InitContract {
                    nebula_data_type: data_type,
                    gravity_contract_program_id: gravity_contract_program_id,
                    initial_oracles: initial_oracles,
                    oracles_bft: bft
                }
            }
            // 1 => {
            //     let mut new_consuls = vec![];
            //     Self::unpack_consuls(rest, &mut new_consuls)?;
            //     println!("consuls: {:?}", new_consuls);

            //     Self::UpdateConsuls {
            //         new_consuls: new_consuls,
            //         current_round: Self::unpack_round(3, rest)?,
            //     }
            // }
            _ => return Err(InvalidInstruction.into()),
        })
    }

    // fn unpack_bft(input: &'a [u8]) -> Result<u8, ProgramError> {
    //     Ok(input
    //         .get(Self::BFT_RANGE)
    //         .and_then(|slice| slice.try_into().ok())
    //         .map(u8::from_le_bytes)
    //         .ok_or(InvalidInstruction)?)
    // }

    // fn unpack_oracles(input: &'a [u8], dst: &mut Vec<Pubkey>) -> Result<(), ProgramError> {
    //     let bft: u8 = Self::unpack_bft(input)?;

    //     let range_start = Self::BFT_ALLOC;
    //     let range_end = range_start + 32 * bft as usize;
    //     let consuls_slice = input
    //         .get(range_start..range_end)
    //         .ok_or(InvalidInstruction)?;

    //     // assert!(consuls_slice.len() == 3 * 32);

    //     let address_alloc: usize = 32;

    //     for i in 0..bft as usize {
    //         let pubky = Pubkey::new(
    //             consuls_slice
    //                 .get(i * address_alloc..(i + 1) * address_alloc)
    //                 .ok_or(InvalidInstruction)?,
    //         );
    //         dst.push(pubky);
    //     }

    //     Ok(())
    // }

    // /// Round is considered as first argument and as u256 data type
    // fn unpack_round(bft: u8, input: &[u8]) -> Result<u64, ProgramError> {
    //     let start_offset = Self::BFT_ALLOC + (Self::PUBKEY_ALLOC * bft as usize);
    //     Ok(input
    //         .get(start_offset..start_offset + Self::LAST_ROUND_ALLOC)
    //         .and_then(|slice| slice.try_into().ok())
    //         .map(u64::from_le_bytes)
    //         .ok_or(InvalidInstruction)?)
    // }
}