use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};


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
    const LEN: usize = 202;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, GravityContract::LEN];
        let (
            is_initialized,
            initializer_pubkey,
            bft,
            consuls,
            last_round,
        ) = array_refs![src, 1, 32, 1, 32 * 5, 8];
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
        ) = mut_array_refs![dst, 1, 32, 1, 32 * 5, 8];

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
        
        let mut consuls_copy = consuls.clone();
        consuls_dst.copy_from_slice(
            consuls_copy
                .iter()
                .fold(vec![], |acc,x| { vec![acc, x.to_bytes().to_vec()].concat() })
                .as_slice()
        );

        *last_round_dst = last_round.to_le_bytes();
    }
}


// pub struct Escrow {
//     pub is_initialized: bool,
//     pub initializer_pubkey: Pubkey,
//     pub temp_token_account_pubkey: Pubkey,
//     pub initializer_token_to_receive_account_pubkey: Pubkey,
//     pub expected_amount: u64,
// }

// impl Sealed for Escrow {}

// impl IsInitialized for Escrow {
//     fn is_initialized(&self) -> bool {
//         self.is_initialized
//     }
// }

// impl Pack for Escrow {
//     const LEN: usize = 105;
//     fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
//         let src = array_ref![src, 0, Escrow::LEN];
//         let (
//             is_initialized,
//             initializer_pubkey,
//             temp_token_account_pubkey,
//             initializer_token_to_receive_account_pubkey,
//             expected_amount,
//         ) = array_refs![src, 1, 32, 32, 32, 8];
//         let is_initialized = match is_initialized {
//             [0] => false,
//             [1] => true,
//             _ => return Err(ProgramError::InvalidAccountData),
//         };

//         Ok(Escrow {
//             is_initialized,
//             initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
//             temp_token_account_pubkey: Pubkey::new_from_array(*temp_token_account_pubkey),
//             initializer_token_to_receive_account_pubkey: Pubkey::new_from_array(
//                 *initializer_token_to_receive_account_pubkey,
//             ),
//             expected_amount: u64::from_le_bytes(*expected_amount),
//         })
//     }

//     fn pack_into_slice(&self, dst: &mut [u8]) {
//         let dst = array_mut_ref![dst, 0, Escrow::LEN];
//         let (
//             is_initialized_dst,
//             initializer_pubkey_dst,
//             temp_token_account_pubkey_dst,
//             initializer_token_to_receive_account_pubkey_dst,
//             expected_amount_dst,
//         ) = mut_array_refs![dst, 1, 32, 32, 32, 8];

//         let Escrow {
//             is_initialized,
//             initializer_pubkey,
//             temp_token_account_pubkey,
//             initializer_token_to_receive_account_pubkey,
//             expected_amount,
//         } = self;

//         is_initialized_dst[0] = *is_initialized as u8;
//         initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
//         temp_token_account_pubkey_dst.copy_from_slice(temp_token_account_pubkey.as_ref());
//         initializer_token_to_receive_account_pubkey_dst
//             .copy_from_slice(initializer_token_to_receive_account_pubkey.as_ref());
//         *expected_amount_dst = expected_amount.to_le_bytes();
//     }
// }
