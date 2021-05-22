use std::fmt;

use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

#[derive(PartialEq, PartialOrd, Default, Debug, Clone)]
pub struct GravityContract {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,

    pub bft: u8,
    pub consuls: Vec<Pubkey>,
    pub last_round: u64,
    // pub multisig_program_id: Pubkey
}

impl fmt::Display for GravityContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "is_initialized: {:};
             initializer_pubkey: {:};
             consuls: {:?};
             bft: {:};
             last_round: {:}",
            self.is_initialized, self.initializer_pubkey, self.consuls, self.bft, self.last_round
        )
    }
}

pub trait PartialStorage {
    const DATA_RANGE: std::ops::Range<usize>;

    fn store_at<'a>(raw_data: &'a [u8]) -> &'a [u8] {
        return &raw_data[Self::DATA_RANGE];
    }

    fn store_data_range() -> std::ops::Range<usize> {
        Self::DATA_RANGE
    }
}

impl PartialStorage for GravityContract {
    const DATA_RANGE: std::ops::Range<usize> = 0..74;
}

impl Sealed for GravityContract {}

impl IsInitialized for GravityContract {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for GravityContract {
    const LEN: usize = 74;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, GravityContract::LEN];
        let (is_initialized, initializer_pubkey, bft, consuls, last_round) =
            array_refs![src, 1, 32, 1, 32 * 1, 8];
        let is_initialized = is_initialized[0] != 0;

        Ok(GravityContract {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            bft: u8::from_le_bytes(*bft),
            consuls: vec![
                Pubkey::new_from_array(*array_ref![consuls[0..32], 0, 32]),
            ],
            last_round: u64::from_le_bytes(*last_round),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, GravityContract::LEN];
        let (is_initialized_dst, initializer_pubkey_dst, bft_dst, consuls_dst, last_round_dst) =
            mut_array_refs![dst, 1, 32, 1, 32 * 1, 8];

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
                .fold(vec![], |acc, x| vec![acc, x.to_bytes().to_vec()].concat())
                .as_slice(),
        );

        *last_round_dst = last_round.to_le_bytes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use solana_gravity_adapter::misc::WrappedResult;

    extern crate hex;
    extern crate rand;

    use rand::random;

    fn build_gravity_contract_mock() -> GravityContract {
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

        gravity_contract_mock
    }

    // test serialize and deserialize to prove internal algo is correct
    #[test]
    fn test_ser_deser_internal() -> WrappedResult<()> {
        let gravity_contract_mock = build_gravity_contract_mock();

        // serialize
        let mut serialized_gravity_contract_bytes = [0 as u8; GravityContract::LEN];

        // populate byte slice
        gravity_contract_mock.pack_into_slice(&mut serialized_gravity_contract_bytes);

        // deserialize
        let deserialized_gravity_contract =
            GravityContract::unpack_from_slice(&mut serialized_gravity_contract_bytes)
                .expect("deserialization failed!");

        assert!(deserialized_gravity_contract == gravity_contract_mock);

        Ok(())
    }

    // test serialize and deserialize using raw methods
    #[test]
    fn test_raw_tx_deser() -> WrappedResult<()> {
        let raw_tx_inputs = vec![
            // "01000103bfb92919a3a0f16abc73951e82c05592732e5514ffa5cdae5f77a96d04922c853b243370dff1af837da92b91fc34b6b25bc35c011fdc1061512a3a01ea324b06be8f3dc36da246f1c085fd38b1591451bde88f5681ad8418bc6098ae2852d8daac70d058d54bf86d8a417bcea4f9c98f02a27d25c4744836a7e239df600a347401020200016a0003bfb92919a3a0f16abc73951e82c05592732e5514ffa5cdae5f77a96d04922c852e01163f621519827bd0cb00cfab7f0e4bd432a1ead4e792dea13d6b6d4f6da784d4adcfec5a47849ca331117fbfb1894123239237c0ee1f53e2478cf190fbb00000000000000000",
            "01000104bfb92919a3a0f16abc73951e82c05592732e5514ffa5cdae5f77a96d04922c853b243370dff1af837da92b91fc34b6b25bc35c011fdc1061512a3a01ea324b064c9643f8e3c1418302a94791b588dfe9e50b6f31d13c605078c9a4497d0a3f7cbe8f3dc36da246f1c085fd38b1591451bde88f5681ad8418bc6098ae2852d8da46fff7293cd539558e9376ac765b5b2bc28f920eaba32f29550d22d6ee919f410103030001026a0003bfb92919a3a0f16abc73951e82c05592732e5514ffa5cdae5f77a96d04922c85a3b6d771e642ec6b7997c6013f6a822451f70064db491878fd05c27af94d49f598a4b405cd647c215e128e4bca5d736d3a09a82583e6981ed1cb4837a41f1b6c0000000000000000"
        ];

        for (i, input) in raw_tx_inputs.iter().enumerate() {
            // let decoded_string = hex::decode("48656c6c6f20776f726c6421");
            let mut serialized_gravity_contract_bytes =
                hex::decode(input).expect("hex string to bytes cast failed!");

            println!("len is: {} \n", serialized_gravity_contract_bytes.len());

            // deserialize
            let deserialized_gravity_contract = GravityContract::unpack(
                &mut serialized_gravity_contract_bytes[GravityContract::store_data_range()],
            )
            .expect("deserialization failed!");
            // let deserialized_gravity_contract = GravityContract::unpack_from_slice(&mut serialized_gravity_contract_bytes)
            //     .expect("deserialization failed!");

            println!(
                "contract #{:} from raw tx: \n {:} \n",
                i, deserialized_gravity_contract
            );
            println!(
                "deserialized_gravity_contract: {:}",
                deserialized_gravity_contract
            );
        }

        Ok(())
    }
}
