use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::state::Multisig;
use std::convert::TryInto;
use std::slice::SliceIndex;

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

                Self::UpdateConsuls {
                    current_round: Self::unpack_round(3, rest)?,
                    new_consuls: new_consuls,
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
        let raw_tx_input = "0003f872d107a7b14923cde74b1bd4db800bd1c8e760eeaacd4b62d91e8074f2f66b3be181103d34cbcc048bf08c4764880f01b77454d4f69f022f9befeb0de95ac148a3e124c22a138ec3037538cd72201fc4bfa92cdcb709f9c4218fe24eae41870000000000000000";

        let serialized_gravity_contract_bytes = hex::decode(raw_tx_input)?;
        println!("{:?}", serialized_gravity_contract_bytes);

        GravityContractInstruction::unpack(serialized_gravity_contract_bytes.as_slice())
            .expect("deser failed!");

        Ok(())
    }
}
