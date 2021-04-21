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

use spl_token::state::Account as TokenAccount;

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

        msg!("about to persist data to contract\n");
        // msg!("byte array: \n");
        msg!("gravity contract: {:} \n", gravity_contract_info);
        let mut data = [u8::from(0); 138];
        msg!("data len: {:} \n", data.len());
        msg!("gravity contract len: {:} \n", GravityContract::LEN);
        msg!("get packet len: {:} \n", GravityContract::get_packed_len());

        GravityContract::pack(gravity_contract_info, &mut data)?;
        let mut target_data = &mut gravity_contract_account.try_borrow_mut_data()?;
        for (i, c) in data.iter().enumerate() {
            target_data[i] = *c;
        }

        // msg!(format!("{:x?}", gravity_contract_account.data.borrow()).as_ref());

        Ok(())
    }

    fn process_update_consuls(
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
        // let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        // if !rent.is_exempt(gravity_contract_account.lamports(), gravity_contract_account.data_len()) {
        //     return Err(GravityError::NotRentExempt.into());
        // }

        let mut gravity_contract_info =
            GravityContract::unpack(&gravity_contract_account.data.borrow()[0..138])?;
        if !gravity_contract_info.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }

        msg!("current round: {:}\n", gravity_contract_info.last_round);

        msg!("iterating current consuls \n");
        for (i, consul) in gravity_contract_info.consuls.iter().enumerate() {
            msg!("current consul #{:} is \n", i);
            consul.log();
        }

        msg!("input round: {:}\n", current_round);

        msg!("iterating input consuls \n");
        for (i, consul) in new_consuls.iter().enumerate() {
            msg!("input consul #{:} is \n", i);
            consul.log();
        }

        Ok(())
    }
}
