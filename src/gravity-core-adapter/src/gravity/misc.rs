use std::error;

use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::gravity::error::GravityError::InvalidInstruction;

pub type WrappedResult<T> = Result<T, Box<dyn error::Error>>;

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

pub fn validate_contract_emptiness(target_contract: &[u8]) -> Result<(), ProgramError> {
    for byte in target_contract.iter() {
        if *byte != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
    }

    Ok(())
}