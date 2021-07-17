use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};



use crate::nebula::processor::NebulaProcessor;

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    NebulaProcessor::process(program_id, accounts, instruction_data)
}

entrypoint!(process);
