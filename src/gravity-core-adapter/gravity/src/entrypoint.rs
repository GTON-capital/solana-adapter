use solana_program::{
    account_info::next_account_info,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use spl_token::{
    // state::Account as TokenAccount
    error::TokenError,
    instruction::initialize_multisig,
    instruction::is_valid_signer_index,

    state::Multisig,
};

use crate::gravity::{
    error::GravityError, instruction::GravityContractInstruction, state::GravityContract,
};
use crate::gravity::processor::GravityProcessor;

fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    GravityProcessor::process(program_id, accounts, instruction_data)
}

entrypoint!(process);


// pub fn extract_from_range<'a, T: std::convert::From<&'a [u8]>, U, F: FnOnce(T) -> U>(
//     input: &'a [u8],
//     index: std::ops::Range<usize>,
//     f: F,
// ) -> Result<U, ProgramError> {
//     let res = input
//         .get(index)
//         .and_then(|slice| slice.try_into().ok())
//         .map(f)
//         .ok_or(InvalidInstruction)?;
//     Ok(res)
// }
