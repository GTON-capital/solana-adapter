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
    instruction::{burn, mint_to, set_authority, AuthorityType},
};

use gravity_misc::validation::validate_contract_emptiness;



use gravity_misc::ports::state::ForeignAddress;

use crate::ibport::instruction::IBPortContractInstruction;
use crate::ibport::state::IBPortContract;

use gravity_misc::ports::error::PortError;
use gravity_misc::ports::state::PortOperationIdentifier;
use gravity_misc::validation::{PDAResolver, validate_pubkey_match, TokenMintConstrained};


// fn get_mint_address_with_seed(target_address: &Pubkey, token_program_id: &Pubkey) -> (Pubkey, u8) {
//     Pubkey::find_program_address(&[&target_address.to_bytes(), br"mint"], token_program_id)
// }

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

        validate_contract_emptiness(&ibport_contract_account.try_borrow_data()?[0..3000])?;

        let mut ibport_contract_info = IBPortContract::default();

        ibport_contract_info.is_state_initialized = true;
        ibport_contract_info.token_address = *token_address;
        ibport_contract_info.nebula_address = *nebula_address;
        ibport_contract_info.oracles = oracles.clone();
        ibport_contract_info.initializer_pubkey = *initializer.key;

        msg!("instantiated ib port contract");

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
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ibport_contract_account = next_account_info(account_info_iter)?;

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

        ibport_contract_info.validate_token_mint(mint.key)?;
        // if *mint.key != susy_wrapped_gton_mint() {
        //     return Err(PortError::InvalidTokenMint.into());
        // }

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
            &[&[PDAResolver::Gravity.bump_seeds()]],
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
        // if multisig_owner_keys.len() == 0 {
        //     return Ok(());
        // }

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

        let ibport_contract_account = next_account_info(account_info_iter)?;

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

        ibport_contract_info.validate_token_mint(mint.key)?;

        msg!("Creating mint instruction");

        let mut amount: u64 = 0;
        
        let operation = ibport_contract_info.attach_data(byte_data, recipient_account.key, &mut amount)?;

        if operation == PortOperationIdentifier::MINT.to_string() {
            msg!("unpacked ibport_contract_account");
    
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
                &[&[PDAResolver::Gravity.bump_seeds()]]
            )?;
        }

        IBPortContract::pack(
            ibport_contract_info,
            &mut ibport_contract_account.try_borrow_mut_data()?[0..IBPortContract::LEN],
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

        let ibport_contract_account = next_account_info(account_info_iter)?;

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_account.data.borrow()[0..IBPortContract::LEN])?;

        msg!("validating initializer");
        Self::validate_data_provider(
            &ibport_contract_info.oracles,
            initializer.key,
        )?;

        msg!("dropping processed request");
        ibport_contract_info.drop_processed_request(byte_data)?;
        
        IBPortContract::pack(
            ibport_contract_info,
            &mut ibport_contract_account.try_borrow_mut_data()?[0..IBPortContract::LEN],
        )?;


        Ok(())
    }

    fn process_transfer_ownership(
        accounts: &[AccountInfo],
        new_authority: &Pubkey,
        new_token_address: &Pubkey,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ibport_contract_account = next_account_info(account_info_iter)?;

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_account.data.borrow()[0..IBPortContract::LEN])?;

        msg!("validating initializer");
        Self::validate_data_provider(
            &ibport_contract_info.oracles,
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
            &[&[PDAResolver::Gravity.bump_seeds()]]
        )?;
        
        let empty_addr: [u8; 32] = [0; 32];
        if new_token_address.to_bytes() != empty_addr {
            ibport_contract_info.token_address = *new_token_address;

            IBPortContract::pack(
                ibport_contract_info,
                &mut ibport_contract_account.try_borrow_mut_data()?[0..IBPortContract::LEN],
            )?;
        }

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
                byte_data
            } => {
                msg!("Instruction: ConfirmDestinationChainRequest");

                Self::process_confirm_destination_chain_request(
                    accounts,
                    &byte_data,
                    program_id,
                )
            }
            IBPortContractInstruction::TransferTokenOwnership {
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
            // _ => Err(GravityError::InvalidInstruction.into()),
        }
    }    
}
