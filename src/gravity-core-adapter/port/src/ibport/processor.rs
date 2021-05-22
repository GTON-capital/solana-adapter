use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::{Clock, Slot},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use spl_token::{
    error::TokenError,
    instruction::is_valid_signer_index,
    state::Multisig,
};

use uuid::Uuid;

use gravity_misc::validation::{validate_contract_emptiness, validate_contract_non_emptiness};
use solana_gravity_contract::gravity::{
    error::GravityError, instruction::GravityContractInstruction, processor::MiscProcessor,
    state::GravityContract,
};

use crate::ibport::instruction::IBPortContractInstruction;
use crate::ibport::state::IBPortContract;
use gravity_misc::model::{DataType, PulseID, SubscriptionID};

pub struct IBPortProcessor;

impl IBPortProcessor {
    fn process_init_ibport_contract(
        accounts: &[AccountInfo],
        nebula_address: Pubkey,
        token_address: Pubkey,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ibport_contract_account = next_account_info(account_info_iter)?;

        validate_contract_emptiness(&ibport_contract_account.try_borrow_data()?[..])?;

        let mut ibport_contract_info = IBPortContract::default();

        ibport_contract_info.token_address = token_address;
        ibport_contract_info.nebula_address = nebula_address;
        ibport_contract_info.initializer_pubkey = *initializer.key;

        msg!("instantiated ib port contract");

        msg!("nebula contract len: {:} \n", IBPortContract::LEN);
        msg!("get packet len: {:} \n", IBPortContract::get_packed_len());

        msg!("packing ib port contract");

        // return Ok(());
        IBPortContract::pack(
            ibport_contract_info,
            &mut ibport_contract_account.try_borrow_mut_data()?[0..IBPortContract::LEN],
        )?;

        Ok(())
    }


}
