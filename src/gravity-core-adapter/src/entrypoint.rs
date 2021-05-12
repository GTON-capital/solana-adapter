use solana_program::{
    account_info::next_account_info,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use cfg_if;

use spl_token::{
    // state::Account as TokenAccount
    error::TokenError,
    instruction::initialize_multisig,
    instruction::is_valid_signer_index,

    // processor::Processor::process_initialize_multisig,
    // processor::Processor as TokenProcessor,
    state::Multisig,
};

use crate::gravity::{
    error::GravityError, instruction::GravityContractInstruction,
    misc::validate_contract_emptiness, state::GravityContract,
};
use crate::nebula::instruction::NebulaContractInstruction;

#[cfg(feature = "nebula-contract")]
use crate::nebula::processor::NebulaProcessor;

// cfg_if::cfg_if! {
//     if #[cfg(feature = "gravity-contract")] {
//         fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
//             GravityProcessor::process(program_id, accounts, instruction_data)
//         }
//     } else if #[cfg(feature = "nebula-contract")] {
//         fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
//             NebulaProcessor::process(program_id, accounts, instruction_data)
//         }
//     } else {
//         panic!("invalid endpoint provided");
//     }
// }

#[cfg(feature = "nebula-contract")]
fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    NebulaProcessor::process(program_id, accounts, instruction_data)
}

entrypoint!(process);
