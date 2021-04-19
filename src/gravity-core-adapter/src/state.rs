use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use rand::random;

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};


#[derive(PartialEq, PartialOrd, Default, Clone)]
pub struct GravityContract {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,

    pub bft: u8,
    pub consuls: Vec<Pubkey>,
    pub last_round: u64
}


impl Sealed for GravityContract {}

impl IsInitialized for GravityContract {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for GravityContract {
    const LEN: usize = 138;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, GravityContract::LEN];
        let (
            is_initialized,
            initializer_pubkey,
            bft,
            consuls,
            last_round,
        ) = array_refs![src, 1, 32, 1, 32 * 3, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(GravityContract {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            bft: u8::from_le_bytes(*bft),
            consuls: vec![
                Pubkey::new_from_array(*array_ref![consuls[0..32], 0, 32]),
                Pubkey::new_from_array(*array_ref![consuls[32..64], 0, 32]),
                Pubkey::new_from_array(*array_ref![consuls[64..96], 0, 32]),
            ],
            last_round: u64::from_le_bytes(*last_round),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, GravityContract::LEN];
        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            bft_dst,
            consuls_dst,
            last_round_dst,
        ) = mut_array_refs![dst, 1, 32, 1, 32 * 3, 8];

        let GravityContract {
            is_initialized,
            initializer_pubkey,
            bft,
            consuls,
            last_round,
        } = self;
        
        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        bft_dst[0] = *bft as u8;
        
        let consuls_copy = consuls.clone();
        consuls_dst.copy_from_slice(
            consuls_copy
                .iter()
                .fold(vec![], |acc,x| { vec![acc, x.to_bytes().to_vec()].concat() })
                .as_slice()
        );

        *last_round_dst = last_round.to_le_bytes();
    }
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    type WrappedResult<T> = Result<T, Box<dyn error::Error>>;

    #[test]
    fn test_ser_deser() -> WrappedResult<()> {
        let mock_gravity_consuls = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let mock_bft: u8 = random();
        let mock_last_round: u64 = random();

        let gravity_contract_mock = GravityContract {
            consuls: mock_gravity_consuls.clone(),
            bft: mock_bft,
            last_round: mock_last_round,
            ..GravityContract::default()
        };

        // serialize
        // let mut serialized_gravity_contract: Vec<u8> = Vec::new();
        let mut serialized_gravity_contract_bytes = [0 as u8; GravityContract::LEN];
        gravity_contract_mock.pack_into_slice(&mut serialized_gravity_contract_bytes);

        // deserialize
        let deserialized_gravity_contract = GravityContract::unpack_from_slice(&mut serialized_gravity_contract_bytes)
            .expect("deserialization failed!");

        assert!(deserialized_gravity_contract == gravity_contract_mock);

        Ok(())
    }
}