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

// use uuid::{Builder as UUIDBuilder, Uuid as UUID};
// use std::bytes::Bytes;

use arrayref::{array_ref, array_refs};
// use hex;

use crate::gravity::misc::extract_from_range;

use crate::nebula::state::{DataType, PulseID, SubscriptionID};

// use hex;
// use crate::state::misc::WrappedResult;
use crate::gravity::error::GravityError::InvalidInstruction;
use crate::nebula::allocs::allocation_by_instruction_index;

pub enum NebulaContractInstruction {
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
        data_hash: Vec<u8>,
    },
    SendValueToSubs {
        data_value: Vec<u8>,
        data_type: DataType,
        pulse_id: PulseID,
        subscription_id: SubscriptionID,
    },
    Subscribe {
        address: Pubkey,
        min_confirmations: u8,
        reward: u64,
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

impl NebulaContractInstruction {
    pub const BFT_ALLOC: usize = 1;
    pub const DATA_TYPE_ALLOC_RANGE: usize = 1;
    pub const PUBKEY_ALLOC: usize = 32;
    pub const PULSE_ID_ALLOC: usize = 8;
    pub const SUB_ID_ALLOC: usize = 16;
    pub const DATA_HASH_ALLOC: usize = 16;

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            // InitContract
            0 => {
                let oracles_bft = extract_from_range(rest, 0..1, |x: &[u8]| {
                    u8::from_le_bytes(*array_ref![x, 0, 1])
                })?;
                let allocs = allocation_by_instruction_index((*tag).into(), Some(oracles_bft as usize))?;
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
            // UpdateOracles
            1 => {
                let bft = extract_from_range(rest, 0..1, |x: &[u8]| {
                    u8::from_le_bytes(*array_ref![x, 0, 1])
                })?;
                let allocs = allocation_by_instruction_index((*tag).into(), Some(bft as usize))?;
                let ranges = build_range_from_alloc(&allocs);

                let new_oracles = Self::retrieve_oracles(rest, ranges[1].clone(), bft)?;
                let new_round = extract_from_range(rest, ranges[1].clone(), |x: &[u8]| {
                    PulseID::from_le_bytes(*array_ref![x, 0, 8])
                })?;

                Self::UpdateOracles {
                    new_round,
                    new_oracles
                }
            }
            // SendHashValue
            2 => {
                // let allocs = vec![Self::DATA_HASH_ALLOC];
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);

                let data_hash =
                    extract_from_range(rest, ranges[0].clone(), |x: &[u8]| *array_ref![x, 0, 16])?;
                let data_hash = data_hash.to_vec();

                Self::SendHashValue { data_hash }
            }
            // SendValueToSubs
            3 => {
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);

                let (data_value, data_type, new_round, subscription_id) = (
                    ranges[0].clone(),
                    ranges[1].clone(),
                    ranges[2].clone(),
                    ranges[3].clone(),
                );

                let data_value =
                    extract_from_range(rest, data_value, |x: &[u8]| *array_ref![x, 0, 16])?;
                let data_value = data_value.to_vec();

                let data_type =
                    DataType::cast_from(extract_from_range(rest, data_type, |x: &[u8]| {
                        u8::from_le_bytes(*array_ref![x, 0, 1])
                    })?);
                let new_round = extract_from_range(rest, new_round, |x: &[u8]| {
                    PulseID::from_le_bytes(*array_ref![x, 0, 8])
                })?;
                let subscription_id =
                    extract_from_range(rest, subscription_id, |x: &[u8]| *array_ref![x, 0, 16])?;

                Self::SendValueToSubs {
                    data_value,
                    data_type,
                    pulse_id: new_round,
                    subscription_id,
                }
            }
            // Subscribe
            4 => {
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;

                let built_range = build_range_from_alloc(&allocs);
                let (address, min_confirmations, reward) = (
                    built_range[0].clone(),
                    built_range[1].clone(),
                    built_range[2].clone(),
                );

                let address = extract_from_range(rest, address, |x: &[u8]| {
                    Pubkey::new_from_array(*array_ref![x, 0, 32])
                })?;

                let min_confirmations =
                    extract_from_range(rest, min_confirmations, |x: &[u8]| {
                        u8::from_le_bytes(*array_ref![x, 0, 1])
                    })?;

                let reward = extract_from_range(rest, reward, |x: &[u8]| {
                    u64::from_le_bytes(*array_ref![x, 0, 8])
                })?;

                Self::Subscribe {
                    address,
                    min_confirmations,
                    reward,
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
            let consuls = x[0..32 * bft as usize].to_vec();
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
