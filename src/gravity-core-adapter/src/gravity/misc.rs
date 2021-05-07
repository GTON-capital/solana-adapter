use std::error;


use solana_program::{
    program_error::ProgramError,
    entrypoint::ProgramResult,
    account_info::AccountInfo
};
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

pub fn is_contract_empty(target_contract: &[u8]) -> bool {
    for byte in target_contract.iter() {
        if *byte != 0 {
            return false;
        }
    }

    return true;
}

pub fn validate_contract_non_emptiness(target_contract: &[u8]) -> Result<(), ProgramError> {
    if is_contract_empty(target_contract) {
        return Err(ProgramError::UninitializedAccount);
    }

    Ok(())
}

pub fn validate_contract_emptiness(target_contract: &[u8]) -> Result<(), ProgramError> {
    if !is_contract_empty(target_contract) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}



pub trait ContractStateValidator {
    fn extract_account_data(accounts: Vec<AccountInfo>) -> Result<AccountInfo<'_>, ProgramError>;

    fn validate_initialized(accounts: &[AccountInfo]) -> ProgramResult;
    fn validate_non_initialized(accounts: &[AccountInfo]) -> ProgramResult;
}
