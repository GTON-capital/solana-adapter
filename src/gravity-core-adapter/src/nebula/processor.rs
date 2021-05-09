use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

// use solana_client::rpc_client::RpcClient;

use spl_token::{
    // instruction::initialize_multisig,
    // state::Account as TokenAccount
    error::TokenError,
    instruction::is_valid_signer_index,

    // processor::Processor::process_initialize_multisig,
    // processor::Processor as TokenProcessor,
    state::Multisig,
};

use uuid::Uuid;

use crate::gravity::{
    error::GravityError,
    instruction::GravityContractInstruction,
    misc::{validate_contract_emptiness, validate_contract_non_emptiness},
    state::GravityContract,
};

use crate::nebula::{
    instruction::NebulaContractInstruction,
    state::{DataType, NebulaContract, PulseID, SubscriptionID},
};

use crate::gravity::{misc::ContractStateValidator, processor::MiscProcessor};

struct NebulaStateValidator;

impl ContractStateValidator for NebulaStateValidator {
    fn extract_account_data(accounts: Vec<AccountInfo>) -> Result<AccountInfo, ProgramError> {
        let account_info_iter = &mut accounts.iter();

        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        Ok(nebula_contract_account.clone())
    }

    fn validate_initialized(accounts: &[AccountInfo]) -> ProgramResult {
        let nebula_contract_account = Self::extract_account_data(accounts.to_vec())?;
        let borrowed_data = nebula_contract_account.try_borrow_data()?;
        validate_contract_non_emptiness(&borrowed_data[..])
    }

    fn validate_non_initialized(accounts: &[AccountInfo]) -> ProgramResult {
        let nebula_contract_account = Self::extract_account_data(accounts.to_vec())?;
        let borrowed_data = nebula_contract_account.try_borrow_data()?;
        validate_contract_emptiness(&borrowed_data[..])
    }
}

pub struct NebulaProcessor;

impl NebulaProcessor {
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

        // if !initializer.is_signer {
        //     return Err(ProgramError::MissingRequiredSignature);
        // }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        // validate_contract_emptiness(&nebula_contract_account.try_borrow_data()?[..])?;
        NebulaStateValidator::validate_non_initialized(accounts)?;

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
        let multisig_result = MiscProcessor::process_init_multisig(
            &nebula_contract_multisig_account,
            &initial_oracles,
            oracles_bft,
        )?;
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

        // if !initializer.is_signer {
        //     return Err(ProgramError::MissingRequiredSignature);
        // }

        let nebula_contract_account = next_account_info(account_info_iter)?;

        // validate_contract_non_emptiness(&nebula_contract_account.try_borrow_data()?[..])?;
        NebulaStateValidator::validate_initialized(accounts)?;

        let mut nebula_contract_info = NebulaContract::unpack(
            &nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN],
        )?;

        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;
        let nebula_contract_multisig_account_pubkey = nebula_contract_info.multisig_account;

        let current_multisig_owners = &accounts[3..];

        msg!("checking multisig bft count");
        match MiscProcessor::validate_owner(
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

    pub fn process_nebula_send_hash_value(
        accounts: &[AccountInfo],
        data_hash: Vec<u8>,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        let nebula_contract_account = next_account_info(account_info_iter)?;

        NebulaStateValidator::validate_initialized(accounts)?;

        let mut nebula_contract_info = NebulaContract::unpack(
            &nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN],
        )?;

        if !nebula_contract_info.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }

        let nebula_contract_multisig_account = next_account_info(account_info_iter)?;
        let nebula_contract_multisig_account_pubkey = nebula_contract_info.multisig_account;

        let current_multisig_owners = &accounts[3..];

        msg!("checking multisig bft count");
        match MiscProcessor::validate_owner(
            program_id,
            &nebula_contract_multisig_account_pubkey,
            &nebula_contract_multisig_account,
            &current_multisig_owners.to_vec(),
        ) {
            Err(_) => return Err(GravityError::InvalidBFTCount.into()),
            _ => {}
        };

        msg!("incrementing pulse id");

        let new_pulse_id = nebula_contract_info.last_pulse_id + 1;

        let multisig_owner_keys = &current_multisig_owners.to_vec();
        let data_hash = multisig_owner_keys.iter().fold(Vec::new(), |a, x| {
            vec![a, x.key.to_bytes().to_vec()].concat()
        });

        let current_block = 0;

        nebula_contract_info.add_pulse(new_pulse_id, data_hash, current_block)?;

        Ok(())
    }

    pub fn process_nebula_send_value_to_subs(
        accounts: &[AccountInfo],
        data_type: DataType,
        pulse_id: PulseID,
        subscription_id: SubscriptionID,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        let nebula_contract_account = next_account_info(account_info_iter)?;

        NebulaStateValidator::validate_initialized(accounts)?;

        let mut nebula_contract_info = NebulaContract::unpack(
            &nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN],
        )?;

        let nebula_contract_multisig_account_pubkey = nebula_contract_info.multisig_account;

        panic!("failed to send values");

        // let rpc_client = RpcClient::new(String::from("https://testnet.solana.com"));
        // let nebula_contract_multisig_info = rpc_client
        //     .get_account(&nebula_contract_multisig_account_pubkey)
        //     .unwrap();

        // let nebula_multisig_info =
        //     Multisig::unpack(&nebula_contract_multisig_info.data[0..NebulaContract::LEN])?;

        // NebulaContract::validate_data_provider(
        //     nebula_multisig_info.signers.to_vec(),
        //     initializer.key,
        // )?;

        // nebula_contract_info.send_value_to_subs(data_type, pulse_id, subscription_id)?;

        // rpc_client.send_and_confirm

        Ok(())
    }

    pub fn process_nebula_subscription(
        accounts: &[AccountInfo],
        subscriber_address: Pubkey,
        min_confirmations: u8,
        reward: u64,
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        let nebula_contract_account = next_account_info(account_info_iter)?;

        NebulaStateValidator::validate_initialized(accounts)?;

        let mut nebula_contract_info = NebulaContract::unpack(
            &nebula_contract_account.try_borrow_data()?[0..NebulaContract::LEN],
        )?;

        // nebula_contract_info.
        // let mut subscription_id = nebula_contract_info.new_subscription_id();

        msg!("generating subscription id");
        msg!("subscribing");

        nebula_contract_info.subscribe(
            *nebula_contract_account.key,
            subscriber_address,
            min_confirmations,
            reward,
        )?;

        msg!("successfully subscribed!");

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
            }
            NebulaContractInstruction::SendHashValue { data_hash } => {
                msg!("Instruction: Send Hash Value");

                Self::process_nebula_send_hash_value(accounts, data_hash, program_id)
            }
            NebulaContractInstruction::SendValueToSubs {
                data_type,
                pulse_id,
                subscription_id,
            } => {
                msg!("Instruction: Send Value To Subs");

                Self::process_nebula_send_value_to_subs(
                    accounts,
                    data_type,
                    pulse_id,
                    subscription_id,
                    program_id,
                )
            }
            NebulaContractInstruction::Subscribe {
                address,
                min_confirmations,
                reward,
            } => {
                msg!("Instruction: Subscribe To Nebula");

                Self::process_nebula_subscription(
                    accounts,
                    address,
                    min_confirmations,
                    reward,
                    program_id,
                )
            }
            _ => Err(GravityError::InvalidInstruction.into()),
        }
    }
}
