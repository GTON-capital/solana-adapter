use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use spl_token::{
    // instruction::initialize_multisig,
    // state::Account as TokenAccount
    error::TokenError,
    instruction::is_valid_signer_index,

    // processor::Processor::process_initialize_multisig,
    // processor::Processor as TokenProcessor,
    state::Multisig,
};

use crate::gravity::{
    error::GravityError, 
    misc::validate_contract_emptiness,
    instruction::GravityContractInstruction,
    state::GravityContract,
};

use crate::nebula::{
    instruction::NebulaContractInstruction,
    state::{
        NebulaContract,
        DataType,
        PulseID
    },
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        Self::process_nebula_contract(
            program_id,
            accounts,
            instruction_data
        )
    }

    pub fn process_nebula_contract(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = NebulaContractInstruction::unpack(instruction_data)?;

        match instruction {
            NebulaContractInstruction::InitContract {
                nebula_data_type,
                gravity_contract_program_id,
                initial_oracles,
                oracles_bft,
            } => {
                msg!("Instruction: Init Nebula Contract");

                Self::process_init_nebula_contract(
                    accounts, 
                    nebula_data_type,
                    &gravity_contract_program_id,
                    initial_oracles,
                    oracles_bft,
                    program_id,
                )
            }
            NebulaContractInstruction::UpdateOracles {
                new_oracles,
                new_round,
            } => {
                msg!("Instruction: Update Nebula Oracles");

                Self::process_update_nebula_contract_oracles(
                    accounts, 
                    new_oracles,
                    new_round,
                    program_id,
                )
            },
            _ => Err(GravityError::InvalidInstruction.into())
        }
    }

    pub fn process_gravity_contract(
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
                msg!("Instruction: Init Gravity Contract");

                Self::process_init_gravity_contract(
                    accounts,
                    new_consuls.as_slice(),
                    current_round,
                    bft,
                    program_id,
                )
            }
            GravityContractInstruction::UpdateConsuls {
                current_round,
                new_consuls,
            } => {
                msg!("Instruction: Update Gravity Consuls");
                Self::process_update_consuls(
                    accounts,
                    current_round,
                    new_consuls.as_slice(),
                    program_id,
                )
            } // _ => Err(GravityError::InvalidInstruction.into())
        }
    }


    fn process_init_gravity_contract(
        accounts: &[AccountInfo],
        new_consuls: &[Pubkey],
        current_round: PulseID,
        bft: u8,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let gravity_contract_account = next_account_info(account_info_iter)?;

        validate_contract_emptiness(&gravity_contract_account.try_borrow_data()?[..])?;

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

        GravityContract::pack(
            gravity_contract_info,
            &mut gravity_contract_account.try_borrow_mut_data()?[0..138],
        )?;

        msg!("picking multisig account");
        let gravity_contract_multisig_account = next_account_info(account_info_iter)?;

        msg!("initializing multisig program");
        Self::process_init_multisig(&gravity_contract_multisig_account, new_consuls, bft)?;

        msg!("initialized multisig program!");

        Ok(())
    }

    pub fn process_init_multisig(
        multisig_account: &AccountInfo,
        signer_pubkeys: &[Pubkey],
        minumum_bft: u8,
    ) -> ProgramResult {
        let mut multisig = Multisig::unpack_unchecked(&multisig_account.try_borrow_data()?)?;
        // let multisig_account_len = multisig_account.try_borrow_data()?.len();
        // let multisig_account_rent = &Rent::from_account_info(multisig_account)?;

        if multisig.is_initialized {
            return Err(TokenError::AlreadyInUse.into());
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
        current_round: u64,
        new_consuls: &[Pubkey],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let gravity_contract_account = next_account_info(account_info_iter)?;

        let mut gravity_contract_info =
            GravityContract::unpack(&gravity_contract_account.try_borrow_data()?[0..138])?;
        if !gravity_contract_info.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }

        msg!("picking multisig account");
        let gravity_contract_multisig_account = next_account_info(account_info_iter)?;

        let current_multisig_owners = &accounts[3..];

        match Self::validate_owner(
            program_id,
            &gravity_contract_multisig_account.key,
            &gravity_contract_multisig_account,
            &current_multisig_owners.to_vec(),
        ) {
            Err(_) => return Err(GravityError::InvalidBFTCount.into()),
            _ => {}
        };

        if current_round <= gravity_contract_info.last_round {
            return Err(GravityError::InputRoundMismatch.into());
        }

        gravity_contract_info.last_round = current_round;
        gravity_contract_info.consuls = new_consuls.to_vec();

        GravityContract::pack(
            gravity_contract_info,
            &mut gravity_contract_account.try_borrow_mut_data()?[0..138],
        )?;

        Ok(())
    }

    const MAX_SIGNERS: usize = 11;
    fn validate_owner(
        program_id: &Pubkey,
        expected_owner: &Pubkey,
        owner_account_info: &AccountInfo,
        signers: &[AccountInfo],
    ) -> ProgramResult {
        if expected_owner != owner_account_info.key {
            return Err(TokenError::OwnerMismatch.into());
        }
        if program_id == owner_account_info.owner
            && owner_account_info.data_len() == Multisig::get_packed_len()
        {
            let multisig = Multisig::unpack(&owner_account_info.try_borrow_data()?)?;
            let mut num_signers = 0;
            let mut matched = [false; Self::MAX_SIGNERS];
            for signer in signers.iter() {
                for (position, key) in multisig.signers[0..multisig.n as usize].iter().enumerate() {
                    if key == signer.key && !matched[position] {
                        if !signer.is_signer {
                            return Err(ProgramError::MissingRequiredSignature);
                        }
                        matched[position] = true;
                        num_signers += 1;
                    }
                }
            }
            if num_signers < multisig.m {
                return Err(ProgramError::MissingRequiredSignature);
            }
            return Ok(());
        } else if !owner_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }

    fn process_init_nebula_contract(
        accounts: &[AccountInfo],
        nebula_data_type: DataType,
        gravity_contract_program_id: &Pubkey,
        initial_oracles: Vec<Pubkey>,
        oracles_bft: u8,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        validate_contract_emptiness(&nebula_contract_account.try_borrow_data()?[..])?;

        msg!("instantiating nebula contract");

        let mut nebula_contract_info = NebulaContract::default();


        nebula_contract_info.is_initialized = true;
        nebula_contract_info.initializer_pubkey = *initializer.key;
        nebula_contract_info.bft = oracles_bft;

        nebula_contract_info.oracles = initial_oracles.clone();
        nebula_contract_info.gravity_contract = *gravity_contract_program_id;

        msg!("instantiated nebula contract");

        msg!("nebula contract len: {:} \n", NebulaContract::LEN);
        msg!("get packet len: {:} \n", NebulaContract::get_packed_len());

        msg!("picking multisig account");
        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;

        msg!("initializing multisig program");
        let multisig_result = Self::process_init_multisig(&nebula_contract_multisig_account, &initial_oracles, oracles_bft)?;
        msg!("initialized multisig program!");

        nebula_contract_info.multisig_account = *nebula_contract_multisig_account.key;
        // msg!("actual nebula contract len")
        msg!("packing nebula contract");

        NebulaContract::pack(
            nebula_contract_info,
            &mut nebula_contract_account.try_borrow_mut_data()?[0..NebulaContract::LEN],
        )?;

        Ok(())
    }

    fn process_update_nebula_contract_oracles(
        accounts: &[AccountInfo],
        new_oracles: Vec<Pubkey>,
        new_round: PulseID,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        let mut nebula_contract_info =
            NebulaContract::unpack(&nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN])?;
        if !nebula_contract_info.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }

        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;
        let nebula_contract_multisig_account_pubkey = nebula_contract_info.multisig_account;

        let current_multisig_owners = &accounts[3..];

        msg!("checking multisig bft count");
        match Self::validate_owner(
            program_id,
            &nebula_contract_multisig_account_pubkey,
            &nebula_contract_multisig_account,
            &current_multisig_owners.to_vec(),
        ) {
            Err(_) => return Err(GravityError::InvalidBFTCount.into()),
            _ => {}
        };

        msg!("checking new round validness");
        if new_round <= nebula_contract_info.last_round {
            return Err(GravityError::InputRoundMismatch.into());
        }

        nebula_contract_info.last_round = new_round;
        nebula_contract_info.oracles = new_oracles.to_vec();
        // nebula_contract_info.rounds_dict[&new_round] = true;

        NebulaContract::pack(
            nebula_contract_info,
            &mut nebula_contract_account.try_borrow_mut_data()?[0..NebulaContract::LEN],
        )?;

        Ok(())
    }
}
