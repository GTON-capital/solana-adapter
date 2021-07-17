use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::{Clock, Slot},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use std::{
    cell::{Ref, RefCell, RefMut},
    cmp, fmt,
    rc::Rc,
};

use spl_token::{
    error::TokenError,
    processor::Processor as TokenProcessor,
    instruction::{burn_checked, burn, mint_to, set_authority, is_valid_signer_index, TokenInstruction, AuthorityType},
    state::Multisig,
    state::Account as TokenAccount
};

use uuid::Uuid;

use gravity_misc::validation::{validate_contract_emptiness, extract_from_range, retrieve_oracles};

use solana_gravity_contract::gravity::{
    error::GravityError, instruction::GravityContractInstruction, processor::MiscProcessor,
    state::GravityContract,
};

use arrayref::array_ref;

use gravity_misc::model::{U256};
use crate::luport::state::ForeignAddress;

use crate::luport::instruction::LUPortContractInstruction;
use crate::luport::state::LUPortContract;
use crate::luport::token::susy_wrapped_gton_mint;
use crate::luport::error::PortError;
use crate::luport::bridge::Bridge;
use gravity_misc::model::{DataType, PulseID, SubscriptionID};
use gravity_misc::validation::PDAResolver;


fn get_mint_address_with_seed(target_address: &Pubkey, token_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&target_address.to_bytes(), br"mint"], token_program_id)
}

pub struct LUPortProcessor;

impl LUPortProcessor {
    fn process_init_luport_contract(
        accounts: &[AccountInfo],
        token_address: &Pubkey,
        nebula_address: &Pubkey,
        oracles: &Vec<Pubkey>,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let luport_contract_account = next_account_info(account_info_iter)?;

        validate_contract_emptiness(&luport_contract_account.try_borrow_data()?[..])?;

        let mut luport_contract_info = LUPortContract::default();

        luport_contract_info.is_state_initialized = true;
        luport_contract_info.token_address = *token_address;
        luport_contract_info.nebula_address = *nebula_address;
        luport_contract_info.oracles = oracles.clone();
        luport_contract_info.initializer_pubkey = *initializer.key;

        msg!("instantiated ib port contract");

        msg!("packing ib port contract");

        LUPortContract::pack(
            luport_contract_info,
            &mut luport_contract_account.try_borrow_mut_data()?[0..LUPortContract::LEN],
        )?;

        Ok(())
    }

    fn process_create_transfer_unwrap_request(
        accounts: &[AccountInfo],
        request_id: &[u8; 16],
        ui_amount: f64,
        receiver: &ForeignAddress,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        let luport_contract_account = next_account_info(account_info_iter)?;

        let mut luport_contract_info =
            LUPortContract::unpack(&luport_contract_account.data.borrow()[0..LUPortContract::LEN])?;

        let decimals = 8;
        let amount = spl_token::ui_amount_to_amount(ui_amount, decimals);

        // Get the accounts to mint
        let token_program_id = next_account_info(account_info_iter)?;

        if *token_program_id.key != luport_contract_info.token_address {
            return Err(PortError::InvalidInputToken.into());
        }

        let mint = next_account_info(account_info_iter)?;
        let token_holder = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        if *mint.key != susy_wrapped_gton_mint() {
            return Err(PortError::InvalidTokenMint.into());
        }

        let burn_ix = burn(
            &token_program_id.key,
            &token_holder.key,
            &mint.key,
            &pda_account.key,
            &[],
            amount,
        )?;

        invoke_signed(
            &burn_ix,
            &[
                token_holder.clone(),
                mint.clone(),
                pda_account.clone(),
                token_program_id.clone(),
            ],
            &[&[PDAResolver::LUPort.bump_seeds()]],
        )?;

        msg!("saving request info");
        luport_contract_info.create_transfer_unwrap_request(request_id, amount, token_holder.key, receiver)?;

        LUPortContract::pack(
            luport_contract_info,
            &mut luport_contract_account.try_borrow_mut_data()?[0..LUPortContract::LEN],
        )?;

        Ok(())
    }

    fn validate_data_provider(
        multisig_owner_keys: &Vec<Pubkey>,
        data_provider: &Pubkey,
    ) -> Result<(), PortError> {
        if multisig_owner_keys.len() == 0 {
            return Ok(());
        }

        for owner_key in multisig_owner_keys {
            if *owner_key == *data_provider {
                return Ok(());
            }
        }

        Err(PortError::AccessDenied)
    }

    fn process_attach_value<'a, 't: 'a>(
        accounts: &[AccountInfo<'t>],
        byte_data: &Vec<u8>,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        msg!("got the attach!");
        let initializer = next_account_info(account_info_iter)?;

        // TODO: Caller validation (1)
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let luport_contract_account = next_account_info(account_info_iter)?;

        let mut luport_contract_info =
            LUPortContract::unpack(&luport_contract_account.data.borrow()[0..LUPortContract::LEN])?;

        Self::validate_data_provider(
            &luport_contract_info.oracles,
            initializer.key,
        )?;

        // Get the accounts to mint
        let token_program_id = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;
        let recipient_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        if *mint.key != susy_wrapped_gton_mint() {
            return Err(PortError::InvalidTokenMint.into());
        }

        msg!("Creating mint instruction");

        let mut amount: u64 = 0;
        
        let operation = luport_contract_info.attach_data(byte_data, recipient_account.key, &mut amount)?;

        if operation == String::from("m") {
            let mint_ix = mint_to(
                &token_program_id.key,
                &mint.key,
                &recipient_account.key,
                &pda_account.key,
                &[],
                amount,
            )?;

            invoke_signed(
                &mint_ix,
                &[
                    mint.clone(),
                    recipient_account.clone(),
                    pda_account.clone(),
                    token_program_id.clone(),
                ],
                &[&[PDAResolver::LUPort.bump_seeds()]]
            )?;
        }

        LUPortContract::pack(
            luport_contract_info,
            &mut luport_contract_account.try_borrow_mut_data()?[0..LUPortContract::LEN],
        )?;

        Ok(())
    }

    fn process_confirm_destination_chain_request(
        accounts: &[AccountInfo],
        byte_data: &Vec<u8>,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let luport_contract_account = next_account_info(account_info_iter)?;

        let mut luport_contract_info =
            LUPortContract::unpack(&luport_contract_account.data.borrow()[0..LUPortContract::LEN])?;

        msg!("validating initializer");
        Self::validate_data_provider(
            &luport_contract_info.oracles,
            initializer.key,
        )?;

        msg!("dropping processed request");
        luport_contract_info.drop_processed_request(byte_data)?;
        
        LUPortContract::pack(
            luport_contract_info,
            &mut luport_contract_account.try_borrow_mut_data()?[0..LUPortContract::LEN],
        )?;


        Ok(())
    }

    fn process_transfer_ownership(
        accounts: &[AccountInfo],
        new_authority: &Pubkey,
        new_token_address: &Pubkey,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let luport_contract_account = next_account_info(account_info_iter)?;

        let mut luport_contract_info =
            LUPortContract::unpack(&luport_contract_account.data.borrow()[0..LUPortContract::LEN])?;

        msg!("validating initializer");
        Self::validate_data_provider(
            &luport_contract_info.oracles,
            initializer.key,
        )?;


        // pub fn set_authority(
        //     token_program_id: &Pubkey, 
        //     owned_pubkey: &Pubkey, 
        //     new_authority_pubkey: Option<&Pubkey>, 
        //     authority_type: AuthorityType, 
        //     owner_pubkey: &Pubkey, 
        //     signer_pubkeys: &[&Pubkey]
        // ) -> Result<Instruction, ProgramError>

        let mint = next_account_info(account_info_iter)?;
        let current_owner = next_account_info(account_info_iter)?;
        let token_program_id = next_account_info(account_info_iter)?;

        msg!("set new token owner");

        let set_authority_ix = set_authority(
            &spl_token::id(),
            mint.key,
            Some(new_authority),
            AuthorityType::MintTokens,
            current_owner.key,
            &[],
        )?;

        invoke_signed(
            &set_authority_ix,
            &[
                mint.clone(),
                current_owner.clone(),
                token_program_id.clone(),
            ],
            &[&[PDAResolver::LUPort.bump_seeds()]]
        )?;
        
        let empty_addr: [u8; 32] = [0; 32];
        if new_token_address.to_bytes() != empty_addr {
            luport_contract_info.token_address = *new_token_address;

            LUPortContract::pack(
                luport_contract_info,
                &mut luport_contract_account.try_borrow_mut_data()?[0..LUPortContract::LEN],
            )?;
        }

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
                token_address,
                nebula_address,
                oracles,
            } => {
                msg!("Instruction: Init IB Port Contract");

                Self::process_init_luport_contract(
                    accounts,
                    &token_address,
                    &nebula_address,
                    &oracles,
                    program_id,
                )
            }
            LUPortContractInstruction::CreateTransferUnwrapRequest {
                request_id,
                amount,
                receiver
            } => {
                msg!("Instruction: CreateTransferUnwrapRequest");

                Self::process_create_transfer_unwrap_request(
                    accounts,
                    &request_id,
                    amount,
                    &receiver,
                    program_id,
                )
            }
            LUPortContractInstruction::AttachValue {
                byte_data
            } => {
                msg!("Instruction: AttachValue");

                Self::process_attach_value(
                    accounts,
                    &byte_data,
                    program_id,
                )
            }
            LUPortContractInstruction::ConfirmDestinationChainRequest {
                byte_data
            } => {
                msg!("Instruction: ConfirmDestinationChainRequest");

                Self::process_confirm_destination_chain_request(
                    accounts,
                    &byte_data,
                    program_id,
                )
            }
            LUPortContractInstruction::TransferTokenOwnership {
                new_authority, new_token
            } => {
                msg!("Instruction: ConfirmDestinationChainRequest");

                Self::process_transfer_ownership(
                    accounts,
                    &new_authority,
                    &new_token,
                    program_id,
                )
            }
            _ => Err(GravityError::InvalidInstruction.into()),
        }
    }    
}
