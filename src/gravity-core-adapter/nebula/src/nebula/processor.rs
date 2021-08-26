use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    program::{invoke_signed},
    pubkey::Pubkey,
};

use spl_token::{
    state::Multisig,
};

use gravity_misc::validation::validate_contract_emptiness;
use solana_gravity_contract::gravity::{
    error::GravityError, processor::MiscProcessor,
};

use crate::nebula::instruction::NebulaContractInstruction;
use crate::nebula::state::{NebulaContract};
use crate::nebula::error::NebulaError;

// use solana_port_contract::ibport::instruction::attach_value;
use gravity_misc::ports::instruction::attach_value;

use gravity_misc::model::{DataType, PulseID, SubscriptionID};
use gravity_misc::validation::PDAResolver;

pub struct NebulaProcessor;

impl NebulaProcessor {
    fn process_init_nebula_contract(
        accounts: &[AccountInfo],
        nebula_data_type: DataType,
        gravity_contract_data_account: &Pubkey,
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

        validate_contract_emptiness(&nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN])?;

        let mut nebula_contract_info = NebulaContract::default();

        nebula_contract_info.is_state_initialized = true;
        nebula_contract_info.initializer_pubkey = *initializer.key;
        nebula_contract_info.bft = oracles_bft;

        nebula_contract_info.data_type = nebula_data_type;

        nebula_contract_info.oracles = initial_oracles.clone();
        nebula_contract_info.gravity_contract = *gravity_contract_data_account;

        msg!("instantiated nebula contract");

        msg!("picking multisig account");
        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;

        msg!("initializing multisig program");
        MiscProcessor::process_init_multisig(
            &nebula_contract_multisig_account,
            &initial_oracles,
            oracles_bft,
        )?;
        msg!("initialized multisig program!");

        nebula_contract_info.multisig_account = *nebula_contract_multisig_account.key;
        msg!("packing nebula contract");

        NebulaContract::pack(
            nebula_contract_info,
            &mut nebula_contract_account.try_borrow_mut_data()?[0..NebulaContract::LEN],
        )?;

        Ok(())
    }

    fn process_update_nebula_contract_oracles(
        accounts: &[AccountInfo],
        new_round: PulseID,
        new_oracles: Vec<Pubkey>,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        let mut nebula_contract_info =
            NebulaContract::unpack(&nebula_contract_account.data.borrow()[0..NebulaContract::LEN])?;

        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;
        let nebula_contract_multisig_account_pubkey = nebula_contract_info.multisig_account;

        msg!("checking multisig bft count");
        match MiscProcessor::validate_owner(
            program_id,
            &nebula_contract_multisig_account_pubkey,
            &nebula_contract_multisig_account,
            &accounts[3..3 + nebula_contract_info.bft as usize].to_vec(),
        ) {
            Err(err) => return Err(err),
            _ => {}
        };

        msg!("checking new round validness");
        if new_round <= nebula_contract_info.last_round {
            return Err(GravityError::InputRoundMismatch.into());
        }

        nebula_contract_info.last_round = new_round;
        nebula_contract_info.oracles = new_oracles;

        NebulaContract::pack(
            nebula_contract_info,
            &mut nebula_contract_account.data.borrow_mut()[0..NebulaContract::LEN],
        )?;

        Ok(())
    }

    pub fn process_nebula_send_hash_value(
        accounts: &[AccountInfo],
        data_hash: Vec<u8>,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        let mut nebula_contract_info = NebulaContract::unpack(
            &nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN],
        )?;

        if !nebula_contract_info.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }

        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;
        let nebula_contract_multisig_account_pubkey = nebula_contract_info.multisig_account;

        msg!("checking multisig bft count");

        let multisig_owner_keys = &accounts[3..3 + nebula_contract_info.bft as usize].to_vec();

        match MiscProcessor::validate_owner(
            program_id,
            &nebula_contract_multisig_account_pubkey,
            &nebula_contract_multisig_account,
            &multisig_owner_keys,
        ) {
            Err(err) => return Err(err),
            _ => {}
        };

        msg!("incrementing pulse id");

        // TODO: find out where to catch timestamp
        // let current_block = 1;

        msg!("data_hash(len): {:} \n", &data_hash.len());
        msg!("data_hash: {:?} \n", &data_hash);

        nebula_contract_info.add_pulse(data_hash, nebula_contract_info.last_pulse_id)?;

        NebulaContract::pack(
            nebula_contract_info,
            &mut nebula_contract_account.data.borrow_mut()[0..NebulaContract::LEN],
        )?;

        Ok(())
    }

    pub fn process_nebula_send_value_to_subs(
        accounts: &[AccountInfo],
        data_value: &Vec<u8>,
        _data_type: &DataType,
        pulse_id: &PulseID,
        subscription_id: &SubscriptionID,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        // let _accounts_copy = accounts.clone();
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        let mut nebula_contract_info = NebulaContract::unpack(
            &nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN],
        )?;

        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;
        // let _nebula_contract_multisig_account_pubkey = nebula_contract_info.multisig_account;

        msg!("checking multisig bft count");

        let nebula_multisig_info =
            Multisig::unpack(&nebula_contract_multisig_account.try_borrow_data()?)?;

        NebulaContract::validate_data_provider(
            &nebula_multisig_info.signers.to_vec(),
            initializer.key,
        )?;

        match nebula_contract_info.send_value_to_subs(pulse_id, subscription_id) {
            Ok(subscription) => {

                let destination_program_id = subscription.contract_address;

                // TOKEN - spl_token::id()
                let target_program_id = next_account_info(account_info_iter)?;

                // IB Port Binary
                let subscriber_contract_program_id = next_account_info(account_info_iter)?;

                // IB Port Data Account
                let ibport_data_account = next_account_info(account_info_iter)?;

                let mint = next_account_info(account_info_iter)?;
                let recipient_account = next_account_info(account_info_iter)?;
                let pda_account = next_account_info(account_info_iter)?;

                let additional_data_accounts = &accounts[9..].to_vec().clone();
                // let mut additional_data_account_pubkeys = vec![];

                // for additional_data_account in additional_data_accounts {
                //     additional_data_account_pubkeys.push(additional_data_account.key);
                // }

                if *pda_account.key != destination_program_id {
                    return Err(NebulaError::InvalidSubscriptionProgramID.into());
                }

                let instruction = attach_value(
                    &data_value,
                    &initializer.key,
                    &ibport_data_account.key,
                    &subscriber_contract_program_id.key,
                    target_program_id.key, // &spl_token::id(),
                    &mint.key,
                    &recipient_account.key,
                    &pda_account.key,
                    &[],
                    &additional_data_accounts,
                )?;

                let mut cross_program_accounts = vec![
                    initializer.clone(),
                    ibport_data_account.clone(),
                    subscriber_contract_program_id.clone(),
                    mint.clone(),
                    recipient_account.clone(),
                    pda_account.clone(),
                ];

                for additional_account_info in additional_data_accounts {
                    msg!("left acc: {:?} \n", additional_account_info.key);
                    cross_program_accounts.push(additional_account_info.clone());
                }

                invoke_signed(
                    &instruction,
                    cross_program_accounts.as_slice(),
                    &[&[
                        PDAResolver::Gravity.bump_seeds(),
                    ]]
                )?;

                nebula_contract_info.drop_processed_pulse(data_value)?;

                NebulaContract::pack(
                    nebula_contract_info,
                    &mut nebula_contract_account.try_borrow_mut_data()?[0..NebulaContract::LEN],
                )?;

                Ok(())
            },
            Err(err) => Err(err.into())
        }
    }

    pub fn process_nebula_subscription(
        accounts: &[AccountInfo],
        subscriber_address: Pubkey,
        min_confirmations: u8,
        reward: u64,
        subscription_id: SubscriptionID,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        let mut nebula_contract_info = NebulaContract::unpack(
            &nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN],
        )?;

        msg!("generating subscription id");
        msg!("subscribing");

        nebula_contract_info.subscribe(
            *initializer.key,
            subscriber_address,
            min_confirmations,
            reward,
            &subscription_id,
        )?;

        msg!("successfully subscribed!");

        NebulaContract::pack(
            nebula_contract_info,
            &mut nebula_contract_account.try_borrow_mut_data()?[0..NebulaContract::LEN],
        )?;

        Ok(())
    }

    pub fn process(
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
                new_round,
                new_oracles,
            } => {
                msg!("Instruction: Update Nebula Oracles");

                Self::process_update_nebula_contract_oracles(
                    accounts,
                    new_round,
                    new_oracles,
                    program_id,
                )
            }
            NebulaContractInstruction::SendHashValue { data_hash } => {
                msg!("Instruction: Send Hash Value");

                Self::process_nebula_send_hash_value(accounts, data_hash, program_id)
            }
            NebulaContractInstruction::SendValueToSubs {
                data_value,
                data_type,
                pulse_id,
                subscription_id,
            } => {
                msg!("Instruction: Send Value To Subs");

                Self::process_nebula_send_value_to_subs(
                    accounts,
                    &data_value,
                    &data_type,
                    &pulse_id,
                    &subscription_id,
                    program_id,
                )
            }
            NebulaContractInstruction::Subscribe {
                address,
                min_confirmations,
                reward,
                subscription_id,
            } => {
                msg!("Instruction: Subscribe To Nebula");

                Self::process_nebula_subscription(
                    accounts,
                    address,
                    min_confirmations,
                    reward,
                    subscription_id,
                    program_id,
                )
            }
            _ => Err(GravityError::InvalidInstruction.into()),
        }
    }
}
