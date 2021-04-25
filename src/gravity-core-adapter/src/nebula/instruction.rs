
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

pub enum NebulaContractInstruction {
    SendHashValue {
        
    },
    UpdateOracles {
        current_round: u64,
    },
}

// impl<'a> NebulaContractInstruction {

//     /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
//     pub fn unpack(input: &'a [u8]) -> Result<Self, ProgramError> {
        
//     }

// }
