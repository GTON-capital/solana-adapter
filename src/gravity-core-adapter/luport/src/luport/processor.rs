use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};

use spl_token::{
    instruction::transfer,
};


use crate::luport::instruction::LUPortContractInstruction;
use crate::luport::state::LUPortContract;
use gravity_misc::ports::error::PortError;
use gravity_misc::ports::state::{PortOperationIdentifier, ForeignAddress};
use gravity_misc::validation::{PDAResolver, TokenMintConstrained, validate_pubkey_match, validate_contract_emptiness};


pub struct LUPortProcessor;

impl LUPortProcessor {
    fn process_init_luport_contract(
        accounts: &[AccountInfo],
        token_address: &Pubkey,
        token_mint: &Pubkey,
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

        validate_contract_emptiness(&luport_contract_account.try_borrow_data()?[0..3000])?;

        let mut luport_contract_info = LUPortContract::default();

        luport_contract_info.is_state_initialized = true;
        luport_contract_info.token_address = *token_address;
        luport_contract_info.token_mint = *token_mint;
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
        foreign_receiver: &ForeignAddress,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let luport_contract_account = next_account_info(account_info_iter)?;

        let mut luport_contract_info =
            LUPortContract::unpack(&luport_contract_account.data.borrow()[0..LUPortContract::LEN])?;

        let decimals = 8;
        let amount = spl_token::ui_amount_to_amount(ui_amount, decimals);

        let token_program_id = next_account_info(account_info_iter)?;

        if *token_program_id.key != luport_contract_info.token_address {
            return Err(PortError::InvalidInputToken.into());
        }

        // common token info
        let mint = next_account_info(account_info_iter)?;
        let token_holder = next_account_info(account_info_iter)?;
        let token_receiver = next_account_info(account_info_iter)?;

        luport_contract_info.validate_token_mint(mint.key)?;

        // lock tockens
        let transfer_ix = transfer(
            &token_program_id.key,
            &token_holder.key,
            &token_receiver.key,
            &initializer.key,
            &[],
            amount
        )?;

        invoke_signed(
            &transfer_ix,
            &[
                token_holder.clone(),
                token_receiver.clone(),
                initializer.clone(),
                token_program_id.clone(),
            ],
            &[&[PDAResolver::Gravity.bump_seeds()]],
        )?;

        msg!("saving request info");
        luport_contract_info.create_transfer_wrap_request(request_id, amount, token_holder.key, foreign_receiver)?;

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

        validate_pubkey_match(
            multisig_owner_keys,
            data_provider,
            PortError::AccessDenied
        )
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

        // Get the accounts to unlock
        let token_program_id = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;
        let recipient_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        luport_contract_info.validate_token_mint(mint.key)?;

        msg!("Creating unlock IX");

        let mut amount: u64 = 0;
        
        let operation = luport_contract_info.attach_data(byte_data, recipient_account.key, &mut amount)?;

        if operation == PortOperationIdentifier::UNLOCK {
            let mint_ix = transfer(
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
                &[&[PDAResolver::Gravity.bump_seeds()]]
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
        _program_id: &Pubkey,
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

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = LUPortContractInstruction::unpack(instruction_data)?;

        match instruction {
            LUPortContractInstruction::InitContract {
                token_address,
                token_mint,
                nebula_address,
                oracles,
            } => {
                msg!("Instruction: Init IB Port Contract");

                Self::process_init_luport_contract(
                    accounts,
                    &token_address,
                    &token_mint,
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
        }
    }    
}
