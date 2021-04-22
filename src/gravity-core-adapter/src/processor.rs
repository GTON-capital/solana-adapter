use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::{
    instruction::initialize_multisig,
    // state::Account as TokenAccount
    error::TokenError,
    instruction::is_valid_signer_index,

    processor::Processor::process_initialize_multisig,
    processor::Processor::validate_owner,
    state::Multisig,
};

use crate::{
    error::GravityError, gravity::instruction::GravityContractInstruction,
    gravity::state::GravityContract,
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        mutlisig_program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = GravityContractInstruction::unpack(instruction_data)?;

        match instruction {
            GravityContractInstruction::InitContract {
                new_consuls,
                current_round,
                bft,
            } => {
                msg!("Instruction: Init Consuls");

                Self::process_init_gravity_contract(
                    accounts,
                    new_consuls.as_slice(),
                    current_round,
                    bft,
                    mutlisig_program_id,
                    program_id,
                )
            }
            GravityContractInstruction::UpdateConsuls {
                new_consuls,
                current_round,
            } => {
                msg!("Instruction: Update Consuls");
                Self::process_update_consuls(
                    accounts,
                    new_consuls.as_slice(),
                    current_round,
                    program_id,
                )
            }
        }
    }

    fn process_init_gravity_contract(
        accounts: &[AccountInfo],
        new_consuls: &[Pubkey],
        current_round: u64,
        bft: u8,
        multisig_program_id: &Pubkey,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let gravity_contract_account = next_account_info(account_info_iter)?;
        // let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        // if !rent.is_exempt(gravity_contract_account.lamports(), gravity_contract_account.data_len()) {
        //     return Err(GravityError::NotRentExempt.into());
        // }

        let mut gravity_contract_info =
            GravityContract::unpack(&gravity_contract_account.try_borrow_data()?[0..138])?;
        // if gravity_contract_info.is_initialized() {
        //     return Err(ProgramError::AccountAlreadyInitialized);
        // }

        gravity_contract_info.is_initialized = true;
        gravity_contract_info.initializer_pubkey = *initializer.key;
        gravity_contract_info.bft = bft;

        gravity_contract_info.consuls = new_consuls.to_vec();
        gravity_contract_info.last_round = current_round;


        msg!("checking bft multisignature");

        // msg!("byte array: \n");
        msg!("gravity contract: {:} \n", gravity_contract_info);

        msg!("gravity contract len: {:} \n", GravityContract::LEN);
        msg!("get packet len: {:} \n", GravityContract::get_packed_len());

        GravityContract::pack(gravity_contract_info, &mut gravity_contract_account.try_borrow_mut_data()?[0..138])?;

        let multisig_signers: Vec<&Pubkey> = new_consuls
            .to_vec()
            .iter()
            .collect();

        msg!("building multisig instruction");
        let instruction = initialize_multisig(&multisig_program_id, multisig_program_id, &multisig_signers, bft)?;
        msg!("built multisig instruction");

        msg!("initializing multisig program");
        process_initialize_multisig(accounts, bft);
        msg!("initialized multisig program!");

        Ok(())
    }

    // pub fn process_initialize_multisig(accounts: &[AccountInfo], m: u8) -> ProgramResult {
    //     let account_info_iter = &mut accounts.iter();
    //     let multisig_info = next_account_info(account_info_iter)?;
    //     let multisig_info_data_len = multisig_info.data_len();
    //     // let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    //     let mut multisig = Multisig::unpack_unchecked(&multisig_info.try_borrow_data()?)?;
    //     if multisig.is_initialized {
    //         return Err(TokenError::AlreadyInUse.into());
    //     }

    //     // if !rent.is_exempt(multisig_info.lamports(), multisig_info_data_len) {
    //     //     return Err(TokenError::NotRentExempt.into());
    //     // }

    //     let signer_infos = account_info_iter.as_slice();
    //     multisig.m = m;
    //     multisig.n = signer_infos.len() as u8;

    //     if !is_valid_signer_index(multisig.n as usize) {
    //         return Err(TokenError::InvalidNumberOfProvidedSigners.into());
    //     }
    //     if !is_valid_signer_index(multisig.m as usize) {
    //         return Err(TokenError::InvalidNumberOfRequiredSigners.into());
    //     }
    //     for (i, signer_info) in signer_infos.iter().enumerate() {
    //         multisig.signers[i] = *signer_info.key;
    //     }

    //     // multisig.is_initialized = true;
    //     // Multisig::pack(multisig, &mut multisig_info.data.borrow_mut())?;

    //     Ok(())
    // }

    pub fn process_update_consuls(
        accounts: &[AccountInfo],
        new_consuls: &[Pubkey],
        current_round: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let gravity_contract_account = next_account_info(account_info_iter)?;

        let mut gravity_contract_info =
            GravityContract::unpack(&gravity_contract_account.data.borrow()[0..138])?;
        if !gravity_contract_info.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }


        Ok(())
    }
}
