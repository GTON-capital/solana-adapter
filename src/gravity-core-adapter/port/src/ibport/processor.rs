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

use gravity_misc::validation::{validate_contract_emptiness, validate_contract_non_emptiness, extract_from_range, retrieve_oracles};

use solana_gravity_contract::gravity::{
    error::GravityError, instruction::GravityContractInstruction, processor::MiscProcessor,
    state::GravityContract,
};

use arrayref::array_ref;

use gravity_misc::model::{U256};
use crate::ibport::state::ForeignAddress;

use crate::ibport::instruction::IBPortContractInstruction;
use crate::ibport::state::IBPortContract;
use crate::ibport::error::PortError;
use crate::ibport::bridge::Bridge;
use gravity_misc::model::{DataType, PulseID, SubscriptionID};


fn get_mint_address_with_seed(target_address: &Pubkey, token_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&target_address.to_bytes(), br"mint"], token_program_id)
}

pub struct IBPortProcessor;

impl IBPortProcessor {
    fn process_init_ibport_contract(
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

        let ibport_contract_account = next_account_info(account_info_iter)?;

        validate_contract_emptiness(&ibport_contract_account.try_borrow_data()?[..])?;

        let mut ibport_contract_info = IBPortContract::default();

        ibport_contract_info.token_address = *token_address;
        ibport_contract_info.nebula_address = *nebula_address;
        ibport_contract_info.oracles = oracles.clone();
        ibport_contract_info.initializer_pubkey = *initializer.key;

        msg!("instantiated ib port contract");

        msg!("nebula contract len: {:} \n", IBPortContract::LEN);
        msg!("get packet len: {:} \n", IBPortContract::get_packed_len());

        msg!("packing ib port contract");

        IBPortContract::pack(
            ibport_contract_info,
            &mut ibport_contract_account.try_borrow_mut_data()?[0..IBPortContract::LEN],
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

        let ibport_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&ibport_contract_account.try_borrow_data()?[..])?;

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_account.data.borrow()[0..IBPortContract::LEN])?;

        let decimals = 8;
        let amount = spl_token::ui_amount_to_amount(ui_amount, decimals);

        // Get the accounts to mint
        let token_program_id = next_account_info(account_info_iter)?;

        if *token_program_id.key != ibport_contract_info.token_address {
            return Err(PortError::InvalidInputToken.into());
        }

        let mint = next_account_info(account_info_iter)?;
        let token_holder = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        let burn_ix = burn(
            &token_program_id.key,
            &token_holder.key,
            &mint.key,
            &pda_account.key,
            &[],
            amount,
        )?;

        msg!(format!("token_program_id: {:} \n", token_program_id.key).as_str());
        msg!(format!("mint: {:} \n", mint.key).as_str());
        msg!(format!("token_holder: {:} \n", token_holder.key).as_str());
        msg!(format!("pda_account: {:} \n", pda_account.key).as_str());

        invoke_signed(
            &burn_ix,
            &[
                token_holder.clone(),
                mint.clone(),
                pda_account.clone(),
                token_program_id.clone(),
            ],
            &[&[&b"ibport"[..]]],
        )?;

        msg!("saving request info");
        ibport_contract_info.create_transfer_unwrap_request(request_id, amount, token_holder.key, receiver)?;

        IBPortContract::pack(
            ibport_contract_info,
            &mut ibport_contract_account.try_borrow_mut_data()?[0..IBPortContract::LEN],
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

        let ibport_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&ibport_contract_account.try_borrow_data()?[..])?;

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_account.data.borrow()[0..IBPortContract::LEN])?;

        Self::validate_data_provider(
            &ibport_contract_info.oracles,
            initializer.key,
        )?;

        // Get the accounts to mint
        let token_program_id = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;
        let recipient_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        msg!("Creating mint instruction");

        let mut amount: u64 = 0;
        
        ibport_contract_info.attach_data(byte_data, recipient_account.key, &mut amount)?;

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
            &[&[&b"ibport"[..]]],
        )?;

        IBPortContract::pack(
            ibport_contract_info,
            &mut ibport_contract_account.try_borrow_mut_data()?[0..IBPortContract::LEN],
        )?;

        Ok(())
    }

    fn process_confirm_destination_chain_request(
        accounts: &[AccountInfo],
        request_id: &[u8; 16],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ibport_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&ibport_contract_account.try_borrow_data()?[..])?;

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_account.data.borrow()[0..IBPortContract::LEN])?;

        msg!("validating initializer");
        Self::validate_data_provider(
            &ibport_contract_info.oracles,
            initializer.key,
        )?;

        msg!("dropping processed request");
        ibport_contract_info.drop_processed_request(request_id)?;
        
        IBPortContract::pack(
            ibport_contract_info,
            &mut ibport_contract_account.try_borrow_mut_data()?[0..IBPortContract::LEN],
        )?;


        Ok(())
    }

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = IBPortContractInstruction::unpack(instruction_data)?;

        match instruction {
            IBPortContractInstruction::InitContract {
                token_address,
                nebula_address,
                oracles,
            } => {
                msg!("Instruction: Init IB Port Contract");

                Self::process_init_ibport_contract(
                    accounts,
                    &token_address,
                    &nebula_address,
                    &oracles,
                    program_id,
                )
            }
            IBPortContractInstruction::CreateTransferUnwrapRequest {
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
            IBPortContractInstruction::AttachValue {
                byte_data
            } => {
                msg!("Instruction: AttachValue");

                Self::process_attach_value(
                    accounts,
                    &byte_data,
                    program_id,
                )
            }
            IBPortContractInstruction::ConfirmDestinationChainRequest {
                request_id
            } => {
                msg!("Instruction: ConfirmDestinationChainRequest");

                Self::process_confirm_destination_chain_request(
                    accounts,
                    &request_id,
                    program_id,
                )
            }
            _ => Err(GravityError::InvalidInstruction.into()),
        }
    }    
}
