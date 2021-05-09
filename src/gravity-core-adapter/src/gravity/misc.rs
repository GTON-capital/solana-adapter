use std::error;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use std::convert::TryInto;
use std::ops::Range;
use std::slice::SliceIndex;

use arrayref::array_ref;

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

pub fn map_to_address(x: &[u8]) -> Pubkey {
    // let address = extract_from_range(rest, address, |x: &[u8]| {
    //     Pubkey::new_from_array(*array_ref![x, 0, 32])
    // })?;

    Pubkey::new_from_array(*array_ref![x, 0, 32])
}

pub fn build_range_from_alloc(allocs: &Vec<usize>) -> Vec<Range<usize>> {
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
