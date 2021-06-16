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
        byte_data: &Vec<u8>,
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
    
    /*
    > spl-token mint 8bpdGgw47o72bhWt8Tnn33NoybmkeYgwFQ8QF88xLno7 10 DqNvoZCz6qZqhyLxhQzx7bNtTEng5HAkfD359K2C75Zq 
        Minting 10 tokens
        Token: 8bpdGgw47o72bhWt8Tnn33NoybmkeYgwFQ8QF88xLno7
        Recipient: DqNvoZCz6qZqhyLxhQzx7bNtTEng5HAkfD359K2C75Zq
        Spl TOKEN: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA 

        token: 8bpdGgw47o72bhWt8Tnn33NoybmkeYgwFQ8QF88xLno7 

        recipient: DqNvoZCz6qZqhyLxhQzx7bNtTEng5HAkfD359K2C75Zq 

        config.owner: 2t4FJfcwwtQgVBj2TU89BMJ1pbFXcj9ksZW984q2VgtH 

        amount: 10000000000 

        decimals: 9 

    */
    fn process_test_cross_mint(
        accounts: &[AccountInfo],
        _recipient: &Pubkey,
        ui_amount: f64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ibport_contract_account = next_account_info(account_info_iter)?;
        let receiver = next_account_info(account_info_iter)?;

        // let temp_token_account_info =
        //     TokenAccount::unpack(&temp_token_account.data.borrow())?;
        // let (pda, nonce) = Pubkey::find_program_address(&[b"ibportminter"], program_id);
        // let (pda, nonce) = Pubkey::find_program_address(&[b"ibportminter"], program_id);

        // let expected_allocated_key =
        //     Pubkey::create_program_address(&[b"You pass butter", &[instruction_data[0]]], program_id)?;

        // let token_data_account = next_account_info(account_info_iter)?;
        // let token_contract_data_account = next_account_info(account_info_iter)?;

        let mut ibport_contract_info =
            IBPortContract::unpack(&ibport_contract_account.data.borrow()[0..IBPortContract::LEN])?;

        // let token = ibport_contract_info.token_address;
        // let owner = ibport_contract_data_account.key;
        let decimals = 8;
        let amount = spl_token::ui_amount_to_amount(ui_amount, decimals);

        // return Ok(());
        // let token_program_id = &spl_token::id();
        // let token_program_id = ibport_contract_data_account.token_address;
        // let token = token_data_account.key;

        // let (mint_address, mint_bump_seed) =
        //     get_mint_address_with_seed(ibport_contract_data_account.key, &spl_token::id());
        // let signatures: &[&[_]] = &[];

        // let mint_signer_seeds: &[&[_]] = &[
        //     &ibport_contract_data_account.key.to_bytes(),
        //     br"mint",
        //     &[mint_bump_seed],
        // ];

        // let (pda, nonce) = Pubkey::find_program_address(&[b"ibporttheminter"], program_id);

        // let token_program_id = &ibport_contract_info.token_address;
        let token_program = next_account_info(account_info_iter)?;
        let ibport_contract_account_pda = next_account_info(account_info_iter)?;
        // let token_deployed_program_id = ibport_contract_info.token_address;
        let token_recipient_data_account = receiver.key;

        invoke_signed(
            &mint_to_checked(
                &spl_token::id(),
                token_program.key,
                token_recipient_data_account,
                ibport_contract_account_pda.key,
                &[],
                amount,
                decimals,
            )?,
            &[
                token_program.clone(),
                receiver.clone(),
                ibport_contract_account_pda.clone(),
            ],
            &[
                &[],
                &[],
                &[b"seed"],
            ]

        );
        // token_program_id: &Pubkey, 
        // mint_pubkey: &Pubkey, 
        // account_pubkey: &Pubkey, 
        // owner_pubkey: &Pubkey, 
        // signer_pubkeys: &[&Pubkey], 
        // amount: u64, 
        // decimals: u8

        // invoke_signed(
        //     &mint_to_checked(
        //         token_program.key,
        //         receiver.key,
        //         &pda,
        //         &[&pda],
        //         amount,
        //         decimals,
        //     )?,
        //     &[
        //         pdas_temp_token_account.clone(),
        //         receiver.clone(),
        //         ibport_contract_account.clone(),
        //         token_program.clone(),
        //     ],
        //     &[&[&b"ibportminter"[..], &[nonce]]],
        // )?;
        Ok(())
        // invoke_signed(
        //     &mint_to_checked(
        //         token_program_id,
        //         ibport_contract_data_account.key,
        //         &recipient,
        //         ibport_contract_data_account.key,
        //         &[],
        //         amount,
        //         decimals,
        //     )?,
        //     &[
        //         ibport_contract_data_account.clone(),
        //         receiver.clone(),
        //         ibport_contract_data_account.clone(),
        //     ],
        //     &[&mint_signer_seeds],
        // )

        // TokenProcessor::process_mint_to(
        //     &ibport_contract_info.token_address,
        //     &[
        //         ibport_contract_data_account.clone(),
        //         ibport_contract_data_account.clone(),
        //         ibport_contract_data_account.clone(),
        //     ],
        //     amount,
        //     Some(decimals)
        // )
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
