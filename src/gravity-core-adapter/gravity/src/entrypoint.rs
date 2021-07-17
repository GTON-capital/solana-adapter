use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};



use crate::gravity::processor::GravityProcessor;


pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    GravityProcessor::process(program_id, accounts, instruction_data)
}

entrypoint!(process);
