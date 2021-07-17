
use std::convert::TryInto;
use std::ops::Range;

use arrayref::{array_ref};

use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};
use crate::model::ValidationError;


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

pub fn extract_from_range<'a, T: std::convert::From<&'a [u8]>, U, F: FnOnce(T) -> U>(
    input: &'a [u8],
    index: std::ops::Range<usize>,
    f: F,
) -> Result<U, ProgramError> {
    let res = input
        .get(index)
        .and_then(|slice| slice.try_into().ok())
        .map(f)
        .ok_or(ValidationError::ExtractionError)?;
    Ok(res)
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

pub fn retrieve_oracles(
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


pub enum PDAResolver {
    IBPort,
    LUPort
}

impl PDAResolver {
    pub fn bump_seeds(&self) -> &[u8] {
        match self {
            PDAResolver::IBPort => br"ibport",
            PDAResolver::LUPort => br"luport"
        }
    }
}

