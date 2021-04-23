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
    // instruction::initialize_multisig,
    // state::Account as TokenAccount
    error::TokenError,
    instruction::is_valid_signer_index,

    // processor::Processor::process_initialize_multisig,
    processor::Processor as TokenProcessor,
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
                    program_id,
                )
            },
            _ => Err(GravityError::InvalidInstruction.into())
            // GravityContractInstruction::UpdateConsuls {
            //     new_consuls,
            //     current_round,
            // } => {
            //     msg!("Instruction: Update Consuls");
            //     Self::process_update_consuls(
            //         accounts,
            //         new_consuls.as_slice(),
            //         current_round,
            //         program_id,
            //     )
            // }
        }
    }

    fn process_init_gravity_contract(
        accounts: &[AccountInfo],
        new_consuls: &[Pubkey],
        current_round: u64,
        bft: u8,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let gravity_contract_account = next_account_info(account_info_iter)?;

        let mut gravity_contract_info = GravityContract::default();

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

        msg!("picking multisig account");
        let gravity_contract_multisig_account = next_account_info(account_info_iter)?;

        msg!("initializing multisig program");
        Self::process_init_multisig(&gravity_contract_multisig_account, new_consuls, bft)?;

        msg!("initialized multisig program!");

        Ok(())
    }

    pub fn process_init_multisig(multisig_account: &AccountInfo, signer_pubkeys: &[Pubkey], minumum_bft: u8) -> ProgramResult {
        let mut multisig = Multisig::unpack_unchecked(&multisig_account.try_borrow_data()?)?;
        let multisig_account_len = multisig_account.data_len();
        let multisig_account_rent = &Rent::from_account_info(multisig_account)?;

        if multisig.is_initialized {
            return Err(TokenError::AlreadyInUse.into());
        }
        if !multisig_account_rent.is_exempt(multisig_account.lamports(), multisig_account_len) {
            return Err(TokenError::NotRentExempt.into());
        }

        multisig.m = minumum_bft;
        multisig.n = signer_pubkeys.len() as u8;
        if !is_valid_signer_index(multisig.n as usize) {
            return Err(TokenError::InvalidNumberOfProvidedSigners.into());
        }
        if !is_valid_signer_index(multisig.m as usize) {
            return Err(TokenError::InvalidNumberOfRequiredSigners.into());
        }
        for (i, signer_pubkey) in signer_pubkeys.iter().enumerate() {
            multisig.signers[i] = *signer_pubkey;
        }
        multisig.is_initialized = true;

        Multisig::pack(multisig, &mut multisig_account.try_borrow_mut_data()?)?;

        Ok(())
    }

    pub fn process_update_consuls(
        accounts: &[AccountInfo],
        new_consuls: &[AccountInfo],
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

        msg!("picking multisig account");
        let gravity_contract_multisig_account = next_account_info(account_info_iter)?;

        TokenProcessor::validate_owner(
            program_id, 
            &gravity_contract_multisig_account.key, 
            &gravity_contract_multisig_account,
            &new_consuls.to_vec()
        )?;

        Ok(())
    }
}
