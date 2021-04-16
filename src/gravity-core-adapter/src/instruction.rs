use solana_program::program_error::ProgramError;
use std::convert::TryInto;
use std::slice::{SliceIndex};

use crate::error::GravityError::InvalidInstruction;


mod utils {
    use super::*;

    pub fn extract_from_range(input: &[u8], index: std::ops::Range<usize>) -> Result<u64, ProgramError> {
        let amount = input
            .get(index)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}

pub enum GravityContractInstruction<'a> {
    /// Starts the trade by creating and populating an escrow account and transferring ownership of the given temp token account to the PDA
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the escrow
    /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 2. `[]` The initializer's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
    /// 4. `[]` The rent sysvar
    /// 5. `[]` The token program
    // InitEscrow {
    //     /// The amount party A expects to receive of token Y
    //     amount: u64,
    // },
    /// Accepts a trade
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person taking the trade
    /// 1. `[writable]` The taker's token account for the token they send
    /// 2. `[writable]` The taker's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The PDA's temp token account to get tokens from and eventually close
    /// 4. `[writable]` The initializer's main account to send their rent fees to
    /// 5. `[writable]` The initializer's token account that will receive tokens
    /// 6. `[writable]` The escrow account holding the escrow info
    /// 7. `[]` The token program
    /// 8. `[]` The PDA account
    // Exchange {
    //     /// the amount the taker expects to be paid in the other token, as a u64 because that's the max possible supply of a token
    //     amount: u64,
    // },
    GetConsuls,
    GetConsulsByRoundId {
        current_round: u64
    },
    UpdateConsuls {
        new_consuls: &'a[&'a[u8]],
        current_round: u64
    }
}


impl<'a> GravityContractInstruction<'a> {
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
                let bft_alloc = 8;
                let bft = utils::extract_from_range(input, 256..256 + bft_alloc)?;

                let new_consuls: Result<&'a[&'a[u8]], ProgramError> = Self::unpack_consuls(input, bft as usize);

                Self::UpdateConsuls {
                    current_round: Self::unpack_round(input)?,
                    new_consuls: new_consuls?
                }
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_consuls(input: &'a[u8], bft_value: usize) -> Result<&'a[&'a [u8]], ProgramError> {
        let address_alloc = 256;
        let input_round_alloc = 256;
        let range_start = input_round_alloc + 8;
        let range_end = range_start * bft_value;
        let consuls_slice = input
            .get(range_start..range_end)
            .ok_or(InvalidInstruction)?;
        
        let mut result: Vec<_> = Vec::new();

        for i in 0..bft_value {
            result.push(
                // [i, i * 256]
                consuls_slice.get(i * address_alloc..(i + 1) * address_alloc)
                .ok_or(InvalidInstruction)?
            )
        }

        Ok(result.as_slice())
    }

    /// Round is considered as first argument and as u256 data type
    fn unpack_round(input: &[u8]) -> Result<u64, ProgramError> {
        utils::extract_from_range(input, 0..256)
    }
}
// impl EscrowInstruction {
//     /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
//     pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
//         let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

//         Ok(match tag {
//             0 => Self::InitEscrow {
//                 amount: Self::unpack_amount(rest)?,
//             },
//             1 => Self::Exchange {
//                 amount: Self::unpack_amount(rest)?,
//             },
//             _ => return Err(InvalidInstruction.into()),
//         })
//     }

//     fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
//         let amount = input
//             .get(..8)
//             .and_then(|slice| slice.try_into().ok())
//             .map(u64::from_le_bytes)
//             .ok_or(InvalidInstruction)?;
//         Ok(amount)
//     }
// }
