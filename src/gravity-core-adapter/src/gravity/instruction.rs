
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
    const BFT_ALLOC: usize = 1;
    const PUBKEY_ALLOC: usize = 32;
    const LAST_ROUND_ALLOC: usize = 8;

    const BFT_RANGE: std::ops::Range<usize> = 0..Self::BFT_ALLOC;

    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &'a [u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => {
                let bft: u8 = Self::unpack_bft(rest)?;
                println!("bft: {:}", bft);

                let mut new_consuls = vec![];
                Self::unpack_consuls(rest, &mut new_consuls)?;
                println!("consuls: {:?}", new_consuls);

                let current_round = Self::unpack_round(bft, rest)?;

                Self::InitContract {
                    current_round: current_round,
                    new_consuls: new_consuls,
                    bft: bft,
                }
            }
            1 => {
                let mut new_consuls = vec![];
                Self::unpack_consuls(rest, &mut new_consuls)?;
                println!("consuls: {:?}", new_consuls);

                Self::UpdateConsuls {
                    new_consuls: new_consuls,
                    current_round: Self::unpack_round(3, rest)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gravity::misc::WrappedResult;

    #[test]
    fn test_raw_input() -> WrappedResult<()> {
        let raw_tx_input = "01000104bfb92919a3a0f16abc73951e82c05592732e5514ffa5cdae5f77a96d04922c853b243370dff1af837da92b91fc34b6b25bc35c011fdc1061512a3a01ea324b064c9643f8e3c1418302a94791b588dfe9e50b6f31d13c605078c9a4497d0a3f7cbe8f3dc36da246f1c085fd38b1591451bde88f5681ad8418bc6098ae2852d8da46fff7293cd539558e9376ac765b5b2bc28f920eaba32f29550d22d6ee919f410103030001026a0003bfb92919a3a0f16abc73951e82c05592732e5514ffa5cdae5f77a96d04922c85a3b6d771e642ec6b7997c6013f6a822451f70064db491878fd05c27af94d49f598a4b405cd647c215e128e4bca5d736d3a09a82583e6981ed1cb4837a41f1b6c0000000000000000";

        let serialized_gravity_contract_bytes = hex::decode(raw_tx_input)?;
        println!("{:?}", GravityContract::unpack(&serialized_gravity_contract_bytes[1..139])?);

        GravityContractInstruction::unpack(serialized_gravity_contract_bytes.as_slice())
            .expect("deser failed!");

        Ok(())
    }
}
