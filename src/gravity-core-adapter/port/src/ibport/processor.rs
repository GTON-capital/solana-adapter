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

use spl_token::{
    error::TokenError,
    processor::Processor as TokenProcessor,
    instruction::{burn_checked, mint_to_checked, mint_to, set_authority, is_valid_signer_index, TokenInstruction, AuthorityType},
    state::Multisig,
};

use uuid::Uuid;

use gravity_misc::validation::{validate_contract_emptiness, validate_contract_non_emptiness, extract_from_range, retrieve_oracles};

use solana_gravity_contract::gravity::{
    error::GravityError, instruction::GravityContractInstruction, processor::MiscProcessor,
    state::GravityContract,
};

use arrayref::array_ref;

use gravity_misc::model::{U256};
use crate::ibport::state::{ForeignAddress, AttachedData};

use crate::ibport::instruction::IBPortContractInstruction;
use crate::ibport::state::IBPortContract;
use crate::ibport::error::PortError;
use gravity_misc::model::{DataType, PulseID, SubscriptionID};

pub struct IBPortProcessor;

impl IBPortProcessor {
    fn process_init_ibport_contract(
        accounts: &[AccountInfo],
        token_address: &Pubkey,
        nebula_address: &Pubkey,
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

    fn process_create_transfer_unwrap_request(
        accounts: &[AccountInfo],
        amount: &U256,
        receiver: &ForeignAddress,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        // if !initializer.is_signer {
        //     return Err(ProgramError::MissingRequiredSignature);
        // }

        let ibport_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&ibport_contract_account.try_borrow_data()?[..])?;

        // panic!("not implemented");

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_account.data.borrow()[0..IBPortContract::LEN])?;

        ibport_contract_info.create_transfer_unwrap_request(amount, initializer.key, receiver)?;

        Ok(())
    }

    fn process_attach_value(
        accounts: &[AccountInfo],
        byte_data: &AttachedData,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ibport_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&ibport_contract_account.try_borrow_data()?[..])?;

        panic!("not implemented");

        let nebula_contract_account = next_account_info(account_info_iter)?;
        if !nebula_contract_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_account.data.borrow()[0..IBPortContract::LEN])?;

        ibport_contract_info.attach_data(byte_data)?;
        // mint_to(
        //     &ibport_contract_info.token_address,

        //     // token_program_id: &Pubkey, 
        //     // mint_pubkey: &Pubkey, 
        //     // account_pubkey: &Pubkey, 
        //     // owner_pubkey: &Pubkey, 
        //     // signer_pubkeys: &[&Pubkey], 
        //     // amount: u64
        // );

        
        Ok(())
    }

    fn process_transfer_token_ownership(
        accounts: &[AccountInfo],
        new_owner: &Pubkey,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ibport_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&ibport_contract_account.try_borrow_data()?[..])?;

        // invoke(
        //     &spl_token::instruction::set_authority(
        //         &ibport_contract_account.token_address;,
        //         ibport_contract_account.initializer_pubkey,
        //         Some(new_owner),
        //         spl_token::instruction::AuthorityType::AccountOwner,
        //         ibport_contract_account.initializer_pubkey,
        //         &[],
        //     )?,
        //     &[
        //         // spl_token_program_info.clone(),
        //         // acceptance_token_info.clone(),
        //         // feature_proposal_info.clone(),
        //         AccountInfo::new(
        //             // key: &'a Pubkey,
        //             // is_signer: bool,
        //             // is_writable: bool,
        //             // lamports: &'a mut u64,
        //             // data: &'a mut [u8],
        //             // owner: &'a Pubkey,
        //             // executable: bool,
        //             // rent_epoch: Epoch
        //         )
        //     ],
        // )?;
        Ok(())
    }

    fn process_test_cross_burn(
        accounts: &[AccountInfo],
        recipient: &Pubkey,
        ui_amount: f64,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ibport_contract_data_account = next_account_info(account_info_iter)?;

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_data_account.data.borrow()[0..IBPortContract::LEN])?;

        let token = ibport_contract_info.token_address;
        let owner = ibport_contract_data_account.key;
        let decimals = 8;
        let amount = spl_token::ui_amount_to_amount(ui_amount, decimals);
        
        // let instructions = vec![?];

        invoke(
            &burn_checked(
                &spl_token::id(),
                &token,
                &recipient,
                owner,
                &[],
                amount,
                decimals,
            ).unwrap(), 
            &[
                ibport_contract_data_account.clone()
            ]
        )
        // process_test_cross_mint
    }
    
    fn process_test_cross_mint(
        accounts: &[AccountInfo],
        recipient: &Pubkey,
        ui_amount: f64,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ibport_contract_data_account = next_account_info(account_info_iter)?;
        // let receiver = next_account_info(account_info_iter)?;
        let token_data_account = next_account_info(account_info_iter)?;
        // let token_contract_data_account = next_account_info(account_info_iter)?;

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_data_account.data.borrow()[0..IBPortContract::LEN])?;

        // let token = ibport_contract_info.token_address;
        // let owner = ibport_contract_data_account.key;
        let decimals = 8;
        let amount = spl_token::ui_amount_to_amount(ui_amount, decimals);

        TokenProcessor::process_mint_to(
            &ibport_contract_info.token_address,
            &[
                ibport_contract_data_account.clone(),
                ibport_contract_data_account.clone(),
                ibport_contract_data_account.clone(),
            ],
            amount,
            Some(decimals)
        )
        // let signatures: &[&[_]] = &[
        //     &ibport_contract_data_account.key.to_bytes(),
        // ];

        // let authority_signature_seeds = [&ibport_contract_data_account.key.to_bytes(), &[ibport_contract_data_account.]];
        // let signers = &[&authority_signature_seeds[..]];

        // invoke_signed(
        //     &mint_to_checked(
        //         &ibport_contract_info.token_address,
        //         token_data_account.key,
        //         &recipient,
        //         ibport_contract_data_account.key,
        //         &[],
        //         amount,
        //         decimals,
        //     ).unwrap(), 
        //     &[
        //         ibport_contract_data_account.clone(),
        //         // token_data_account.clone(),
        //     ],
        //     signers
        // )
        // process_test_cross_mint
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
                nebula_address
            } => {
                msg!("Instruction: Init IB Port Contract");

                Self::process_init_ibport_contract(
                    accounts,
                    &token_address,
                    &nebula_address,
                    program_id,
                )
            }
            IBPortContractInstruction::CreateTransferUnwrapRequest {
                amount,
                receiver
            } => {
                msg!("Instruction: Init IB Port Contract");

                Self::process_create_transfer_unwrap_request(
                    accounts,
                    &amount,
                    &receiver,
                    program_id,
                )
            }
            IBPortContractInstruction::AttachValue {
                byte_data
            } => {
                Self::process_attach_value(
                    accounts,
                    &byte_data,
                    program_id,
                )
            }
            IBPortContractInstruction::TransferTokenOwnership {
                new_owner
            } => {
                Self::process_transfer_token_ownership(
                    accounts,
                    &new_owner,
                    program_id,
                )
            },
            IBPortContractInstruction::TestCrossMint {
                receiver,
                amount
            } => {
                Self::process_test_cross_mint(
                    accounts,
                    &receiver,
                    amount,
                    program_id,
                )
            },
            IBPortContractInstruction::TestCrossBurn {
                receiver,
                amount
            } => {
                Self::process_test_cross_burn(
                    accounts,
                    &receiver,
                    amount,
                    program_id,
                )
            },
            _ => Err(GravityError::InvalidInstruction.into()),
        }
    }    
}
