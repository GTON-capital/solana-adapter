use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::{Clock, Slot},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use spl_token::{
    error::TokenError,
    instruction::is_valid_signer_index,
    state::Multisig,
};

use gravity_misc::validation::{validate_contract_emptiness, validate_contract_non_emptiness};
use solana_gravity_contract::gravity::{
    error::GravityError, instruction::GravityContractInstruction, processor::MiscProcessor,
    state::GravityContract,
};

use crate::nebula::instruction::NebulaContractInstruction;
use crate::nebula::state::NebulaContract;
use crate::nebula::error::NebulaError;
use solana_port_contract::ibport::instruction::attach_value;
use gravity_misc::model::{DataType, PulseID, SubscriptionID};

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

        let nebula_contract_account = next_account_info(account_info_iter)?;

        validate_contract_emptiness(&nebula_contract_account.try_borrow_data()?[..])?;

        let mut nebula_contract_info = NebulaContract::default();

        nebula_contract_info.is_initialized = true;
        nebula_contract_info.initializer_pubkey = *initializer.key;
        nebula_contract_info.bft = oracles_bft;

        nebula_contract_info.oracles = initial_oracles.clone();
        nebula_contract_info.gravity_contract = *gravity_contract_data_account;

        msg!("instantiated nebula contract");

        msg!("nebula contract len: {:} \n", NebulaContract::LEN);
        msg!("get packet len: {:} \n", NebulaContract::get_packed_len());

        msg!("picking multisig account");
        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;

        msg!("initializing multisig program");
        let multisig_result = MiscProcessor::process_init_multisig(
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

        let nebula_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&nebula_contract_account.try_borrow_data()?[..])?;

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

        let nebula_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&nebula_contract_account.data.borrow()[..])?;

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

        // match MiscProcessor::validate_owner(
        //     program_id,
        //     &nebula_contract_multisig_account_pubkey,
        //     &nebula_contract_multisig_account,
        //     &multisig_owner_keys,
        // ) {
        //     Err(err) => return Err(err),
        //     _ => {}
        // };

        msg!("incrementing pulse id");

        // let new_pulse_id = nebula_contract_info.last_pulse_id + 1;

        // let data_hash = multisig_owner_keys.iter().fold(Vec::new(), |a, x| {
        //     vec![a, x.key.to_bytes().to_vec()].concat()
        // });

        let clock_info = &accounts[3 + nebula_contract_info.bft as usize];
        msg!(format!("clock_info: {:}", *clock_info.key).as_str());
        let clock = &Clock::from_account_info(clock_info)?;

        let current_block = clock.slot;

        nebula_contract_info.add_pulse(data_hash, nebula_contract_info.last_pulse_id, current_block)?;

        NebulaContract::pack(
            nebula_contract_info,
            &mut nebula_contract_account.data.borrow_mut()[0..NebulaContract::LEN],
        )?;

        Ok(())
    }

    pub fn process_nebula_send_value_to_subs(
        accounts: &[AccountInfo],
        data_value: &Vec<u8>,
        data_type: &DataType,
        pulse_id: &PulseID,
        subscription_id: &SubscriptionID,
        program_id: &Pubkey,
    ) -> ProgramResult {
        msg!("data_type: {:?} \n", data_type);
        msg!("pulse_id: {:?} \n", pulse_id);
        msg!("subscription_id: {:?} \n", subscription_id);

        let accounts_copy = accounts.clone();
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        let nebula_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&nebula_contract_account.data.borrow()[..])?;

        let mut nebula_contract_info = NebulaContract::unpack(
            &nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN],
        )?;

        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;
        let nebula_contract_multisig_account_pubkey = nebula_contract_info.multisig_account;

        msg!("checking multisig bft count");

        // match MiscProcessor::validate_owner(
        //     program_id,
        //     &nebula_contract_multisig_account_pubkey,
        //     &nebula_contract_multisig_account,
        //     &accounts[3..3 + nebula_contract_info.bft as usize].to_vec(),
        // ) {
        //     Err(err) => return Err(err),
        //     _ => {}
        // };

        let nebula_multisig_info =
            Multisig::unpack(&nebula_contract_multisig_account.try_borrow_data()?)?;

        NebulaContract::validate_data_provider(
            nebula_multisig_info.signers.to_vec(),
            initializer.key,
        )?;

        match nebula_contract_info.send_value_to_subs(data_type, pulse_id, subscription_id) {
            Ok(subscription) => {
                let destination_program_id = subscription.contract_address;

                // TOKEN - spl_token::id()
                let target_program_id = next_account_info(account_info_iter)?;

                // IB Port Binary
                let subscriber_contract_program_id = next_account_info(account_info_iter)?;

                msg!("target_program_id(got) {:} \n", target_program_id.key);
                msg!("destination_program_id(expected): {:} \n", destination_program_id);
                msg!("subscriber_contract_program_id(expected): {:} \n", subscriber_contract_program_id.key);

                // return Ok(());

                // IB Port Data Account
                let ibport_data_account = next_account_info(account_info_iter)?;

                let mint = next_account_info(account_info_iter)?;
                let recipient_account = next_account_info(account_info_iter)?;
                let pda_account = next_account_info(account_info_iter)?;

                if *pda_account.key != destination_program_id {
                    return Err(NebulaError::InvalidSubscriptionProgramID.into());
                }

                msg!("ibport_data_account {:?} \n", ibport_data_account.key);
                msg!("data_value {:?} \n", data_value);
                msg!("destination_program_id {:?} \n", destination_program_id);
                msg!("initializer {:?} \n", initializer.key);
                msg!("subscriber_contract_program_id {:?} \n", subscriber_contract_program_id.key);
                msg!("mint {:?} \n", mint.key);
                msg!("recipient_account {:?} \n", recipient_account.key);
                msg!("pda_account {:?} \n", pda_account.key);

                let instruction = attach_value(
                    &data_value,
                    &subscriber_contract_program_id.key,
                    target_program_id.key, // &spl_token::id(),
                    &mint.key,
                    &recipient_account.key,
                    &pda_account.key,
                    &[],
                )?;


                // byte_data: &Vec<u8>,
                // target_program_id: &Pubkey,  // IB Port binary
                // initializer: &Pubkey,
                // token_program_id: &Pubkey, // actually spl_token::id()
                // mint: &Pubkey, // actually the result of spl-token create-token (cli)
                // recipient_account: &Pubkey,
                // ibport_pda_account: &Pubkey,
                // signer_pubkeys: &[&Pubkey],

                invoke_signed(
                    &instruction,
                    &[
                        subscriber_contract_program_id.clone(),
                        mint.clone(),
                        recipient_account.clone(),
                        pda_account.clone(),
                        // target_program_id.clone(),
                    ],
                    &[&[b"ibport"]]
                )?;

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

        let nebula_contract_account = next_account_info(account_info_iter)?;

        validate_contract_non_emptiness(&nebula_contract_account.data.borrow()[..])?;

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
