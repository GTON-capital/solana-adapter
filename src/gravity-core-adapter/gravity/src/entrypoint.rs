use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use spl_token::{
    // state::Account as TokenAccount
    error::TokenError,
    instruction::initialize_multisig,
    instruction::is_valid_signer_index,

    state::Multisig,
};

use crate::gravity::processor::GravityProcessor;
use crate::gravity::{
    error::GravityError, instruction::GravityContractInstruction, state::GravityContract,
};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    GravityProcessor::process(program_id, accounts, instruction_data)
}

entrypoint!(process);
