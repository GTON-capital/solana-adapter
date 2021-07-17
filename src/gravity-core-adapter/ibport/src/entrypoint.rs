use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::ibport::processor::IBPortProcessor;

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    IBPortProcessor::process(program_id, accounts, instruction_data)
}

entrypoint!(process);
