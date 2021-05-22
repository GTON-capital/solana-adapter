use std::error;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};
use std::convert::TryInto;

// use self::error::GravityError::InvalidInstruction;

pub type WrappedResult<T> = Result<T, Box<dyn error::Error>>;

// pub trait ContractStateValidator {
//     fn extract_account_data(accounts: Vec<AccountInfo>) -> Result<AccountInfo<'_>, ProgramError>;

//     fn validate_initialized(accounts: &[AccountInfo]) -> ProgramResult;
//     fn validate_non_initialized(accounts: &[AccountInfo]) -> ProgramResult;
// }
