use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use spl_token::{
    // instruction::initialize_multisig,
    // state::Account as TokenAccount
    error::TokenError,
    instruction::is_valid_signer_index,

    // processor::Processor::process_initialize_multisig,
    // processor::Processor as TokenProcessor,
    state::Multisig,
};

use crate::gravity::{
    error::GravityError, instruction::GravityContractInstruction,
    misc::validate_contract_emptiness, state::GravityContract,
};

use crate::gravity::processor::GravityProcessor;
use crate::nebula::instruction::NebulaContractInstruction;
use crate::nebula::processor::NebulaProcessor;
use crate::nebula::state::{DataType, NebulaContract, PulseID};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        GravityProcessor::process_gravity_contract(program_id, accounts, instruction_data)
    }
}
