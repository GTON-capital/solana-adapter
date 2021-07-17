use std::error;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};
use std::convert::TryInto;

pub type WrappedResult<T> = Result<T, Box<dyn error::Error>>;