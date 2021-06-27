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

use gravity_misc::model::{DataType, PulseID, SubscriptionID};
use gravity_misc::validation::{build_range_from_alloc, extract_from_range, retrieve_oracles};

use crate::nebula::allocs::allocation_by_instruction_index;
use solana_gravity_contract::gravity::error::GravityError::InvalidInstruction;

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
        subscription_id: SubscriptionID,
    },
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
    pub const DATA_HASH_ALLOC: usize = 64;

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            // InitContract
            0 => {
                let oracles_bft = extract_from_range(rest, 0..1, |x: &[u8]| {
                    u8::from_le_bytes(*array_ref![x, 0, 1])
                })?;
                let allocs =
                    allocation_by_instruction_index((*tag).into(), Some(oracles_bft as usize))?;
                let ranges = build_range_from_alloc(&allocs);
                let nebula_data_type = DataType::cast_from(extract_from_range(
                    rest,
                    ranges[1].clone(),
                    |x: &[u8]| u8::from_le_bytes(*array_ref![x, 0, 1]),
                )?);

                let gravity_contract_program_id =
                    extract_from_range(rest, ranges[2].clone(), |x| Pubkey::new(x))?;
                let initial_oracles = retrieve_oracles(rest, ranges[3].clone(), oracles_bft)?;

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

                let new_oracles = retrieve_oracles(rest, ranges[1].clone(), bft)?;
                let new_round = extract_from_range(rest, ranges[2].clone(), |x: &[u8]| {
                    PulseID::from_le_bytes(*array_ref![x, 0, 8])
                })?;

                Self::UpdateOracles {
                    new_round,
                    new_oracles,
                }
            }
            // SendHashValue
            2 => {
                // let allocs = vec![Self::DATA_HASH_ALLOC];
                let allocs = allocation_by_instruction_index((*tag).into(), None)?;
                let ranges = build_range_from_alloc(&allocs);

                let data_hash =
                    extract_from_range(rest, ranges[0].clone(), |x: &[u8]| *array_ref![x, 0, NebulaContractInstruction::DATA_HASH_ALLOC])?;
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
                    extract_from_range(rest, data_value, |x: &[u8]| *array_ref![x, 0, NebulaContractInstruction::DATA_HASH_ALLOC])?;
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
                let (address, min_confirmations, reward, subscription_id) = (
                    built_range[0].clone(),
                    built_range[1].clone(),
                    built_range[2].clone(),
                    built_range[3].clone(),
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
                let subscription_id = extract_from_range(rest, subscription_id, |x: &[u8]| *array_ref![x, 0, 16])?;

                Self::Subscribe {
                    address,
                    min_confirmations,
                    reward,
                    subscription_id,
                }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
