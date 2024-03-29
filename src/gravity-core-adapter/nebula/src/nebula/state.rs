


use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use gravity_misc::model::{
    AbstractRecordHandler, RecordHandler, DataType, PulseID, SubscriptionID
};
use gravity_misc::validation::validate_pubkey_match;

use solana_gravity_contract::gravity::state::PartialStorage;

use crate::nebula::error::NebulaError;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Default, Debug, Clone, Copy)]
pub struct Subscription {
    pub sender: Pubkey,
    pub contract_address: Pubkey,
    pub min_confirmations: u8,
    pub reward: u64, // should be 2^256
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Default, Debug, Clone)]
pub struct Pulse {
    pub data_hash: Vec<u8>,
    // pub height: u64,
}

pub type NebulaQueue<T> = Vec<T>;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Default, Debug, Clone)]
pub struct NebulaContract {
    pub oracles: Vec<Pubkey>,

    pub bft: u8,
    pub multisig_account: Pubkey,
    pub gravity_contract: Pubkey,
    pub data_type: DataType,
    pub last_round: PulseID,

    pub last_pulse_id: PulseID,

    subscriptions_map: RecordHandler<SubscriptionID, Subscription>,

    pulses_map: RecordHandler<Pulse, PulseID>,

    pub is_state_initialized: bool,
    pub initializer_pubkey: Pubkey,
}

impl PartialStorage for NebulaContract {
    const DATA_RANGE: std::ops::Range<usize> = 0..1500;
}

impl Sealed for NebulaContract {}

impl IsInitialized for NebulaContract {
    fn is_initialized(&self) -> bool {
        self.is_state_initialized
    }
}

impl Pack for NebulaContract {
    const LEN: usize = 1500;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut mut_src: &[u8] = src;
        Self::deserialize(&mut mut_src).map_err(|err| {
            msg!(
                "Error: failed to deserialize NebulaContract instruction: {}",
                err
            );
            ProgramError::InvalidInstructionData
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let data = self.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }
}

impl NebulaContract {
    pub fn add_pulse(
        &mut self,
        data_hash: Vec<u8>,
        last_pulse_id: u64,
    ) -> Result<(), NebulaError> {
        let new_pulse_id = last_pulse_id + 1;

        self.pulses_map.insert(
            Pulse {
                data_hash,
            },
            new_pulse_id,
        );

        self.last_pulse_id = new_pulse_id;

        Ok(())
    }

    pub fn unsubscribe(
        &mut self,
        _subscription_id: &SubscriptionID,
    ) -> Result<(), NebulaError> {
        // Ok(())
        Err(NebulaError::UnsubscribeIsNotAvailable)
    }

    pub fn subscribe(
        &mut self,
        sender: Pubkey,
        contract_address: Pubkey,
        min_confirmations: u8,
        reward: u64,
        subscription_id: &SubscriptionID,
    ) -> Result<(), NebulaError> {
        let subscription = Subscription {
            sender,
            contract_address,
            min_confirmations,
            reward,
        };

        // an approach to avoid collision
        if self.subscriptions_map.contains_key(subscription_id) {
            return Err(NebulaError::SubscribeFailed);
        }

        self.subscriptions_map.insert(*subscription_id, subscription);

        Ok(())
    }

    pub fn validate_data_provider(
        multisig_owner_keys: &Vec<Pubkey>,
        data_provider: &Pubkey,
    ) -> Result<(), NebulaError> {
        // for owner_key in multisig_owner_keys {
        //     if owner_key == data_provider {
        //         return Ok(());
        //     }
        // }

        validate_pubkey_match(
            multisig_owner_keys,
            data_provider,
            NebulaError::DataProviderForSendValueToSubsIsInvalid
        )
    }

    pub fn drop_processed_pulse(&mut self, raw_data_value: &Vec<u8>) -> Result<(), NebulaError> {
        let sha256_hashed = solana_program::hash::hash(raw_data_value.clone().as_slice());

        msg!("sha256_hashed: {:?} \n", &sha256_hashed);
        msg!("sha256_hashed(bytes): {:?} \n", & sha256_hashed.to_bytes());
        msg!("sha256_hashed(bytes.to_vec()): {:?} \n", & sha256_hashed.to_bytes().to_vec());
        msg!("raw_data_value: {:?} \n", raw_data_value);

        let pulse = &Pulse {
            data_hash: sha256_hashed.to_bytes().to_vec(),
        };

        match self.pulses_map.drop(pulse) {
            Some(_) => Ok(()),
            None => Err(NebulaError::PulseIDHasNotBeenPersisted),
        }
    }

    pub fn send_value_to_subs(
        &mut self,
        pulse_id: &PulseID,
        subscription_id: &SubscriptionID,
    ) -> Result<Subscription, ProgramError> {
        let prev_pulse_id = self.last_pulse_id - 1;
        msg!("prev_pulse_id: {:} \n", prev_pulse_id);
        msg!("pulse_id: {:} \n", pulse_id);
        
        if *pulse_id != prev_pulse_id {
            return Err(NebulaError::PulseValidationOrderMismatch.into());
        }

        match self.subscriptions_map.get(&subscription_id) {
            Some(v) => Ok(v.clone()),
            None => return Err(NebulaError::InvalidSubscriptionID.into()),
        }
    }
}
