use std::fmt;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[derive(
    BorshDeserialize, BorshSchema, BorshSerialize, PartialEq, PartialOrd, Default, Debug, Clone,
)]
pub struct GravityContract {
    // pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,

    pub bft: u8,
    pub consuls: Vec<Pubkey>,
    pub last_round: u64,
    pub multisig_account: Pubkey,
}

impl fmt::Display for GravityContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "initializer_pubkey: {:};
             consuls: {:?};
             bft: {:};
             last_round: {:}",
            self.initializer_pubkey, self.consuls, self.bft, self.last_round
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
    const DATA_RANGE: std::ops::Range<usize> = 0..299;
}

impl Sealed for GravityContract {}

impl IsInitialized for GravityContract {
    fn is_initialized(&self) -> bool {
        true
    }
}

impl Pack for GravityContract {
    const LEN: usize = 299;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut mut_src: &[u8] = src;
        Self::deserialize(&mut mut_src).map_err(|err| {
            msg!(
                "Error: failed to deserialize GravityContract instruction: {}",
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
