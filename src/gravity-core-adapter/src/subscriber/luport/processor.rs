use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

// use solana_client::rpc_client::RpcClient;

use spl_token::{
    // instruction::initialize_multisig,
    // state::Account as TokenAccount
    error::TokenError,
    instruction::is_valid_signer_index,

    // processor::Processor::process_initialize_multisig,
    // processor::Processor as TokenProcessor,
    state::Multisig,
};

use uuid::Uuid;

use crate::gravity::{
    error::GravityError,
    instruction::GravityContractInstruction,
    misc::{validate_contract_emptiness, validate_contract_non_emptiness},
    state::GravityContract,
};
use crate::subscriber::luport::instruction::LUPortContractInstruction;

// use crate::nebula::{
//     instruction::NebulaContractInstruction,
//     state::{DataType, NebulaContract, PulseID, SubscriptionID},
// };

use crate::gravity::{misc::ContractStateValidator, processor::MiscProcessor};

struct LUPortStateValidator;

impl ContractStateValidator for LUPortStateValidator {
    fn extract_account_data(accounts: Vec<AccountInfo>) -> Result<AccountInfo, ProgramError> {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        Ok(nebula_contract_account.clone())
    }

    fn validate_initialized(accounts: &[AccountInfo]) -> ProgramResult {
        let nebula_contract_account = Self::extract_account_data(accounts.to_vec())?;
        let borrowed_data = nebula_contract_account.try_borrow_data()?;
        validate_contract_non_emptiness(&borrowed_data[..])
    }

    fn validate_non_initialized(accounts: &[AccountInfo]) -> ProgramResult {
        let nebula_contract_account = Self::extract_account_data(accounts.to_vec())?;
        let borrowed_data = nebula_contract_account.try_borrow_data()?;
        validate_contract_emptiness(&borrowed_data[..])
    }
}

pub struct LUPortProcessor;

impl LUPortProcessor {
    fn process_init_lu_port(
        accounts: &[AccountInfo],
        nebula_address: &Pubkey,
        token_address: &Pubkey,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        Ok(())
    }

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = LUPortContractInstruction::unpack(instruction_data)?;

        match instruction {
            LUPortContractInstruction::InitContract {
                nebula_address,
                token_address,
            } => {
                msg!("Instruction: Init LU Port Contract");

                Self::process_init_lu_port(accounts, &nebula_address, &token_address, program_id)
            }
            _ => Err(GravityError::InvalidInstruction.into()),
        }
    }
}
