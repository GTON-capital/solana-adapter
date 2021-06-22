//! Bridge transition types

use std::mem::size_of;

// use primitive_types::U256;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    instruction::Instruction,
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
};
// use zerocopy::AsBytes;
// use primitive_types::U256;


use crate::{
    ibport::error::PortError as Error,
};
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;

pub struct Bridge {
    ibport_program_id: Pubkey
}

/// Implementation of actions
impl Bridge {

    /// Calculates derived seeds for a bridge
    pub fn derive_bridge_seeds() -> Vec<Vec<u8>> {
        vec!["ibport".as_bytes().to_vec()]
    }

    pub fn derive_bridge_id(program_id: &Pubkey) -> Result<Pubkey, Error> {
        Ok(Self::derive_key(program_id, &Self::derive_bridge_seeds())?.0)
    }

    pub fn find_program_address(
        seeds: &Vec<Vec<u8>>,
        program_id: &Pubkey,
    ) -> (Pubkey, Vec<Vec<u8>>) {
        let mut nonce = [255u8];
        for _ in 0..std::u8::MAX {
            {
                let mut seeds_with_nonce = seeds.to_vec();
                seeds_with_nonce.push(nonce.to_vec());
                let s: Vec<_> = seeds_with_nonce
                    .iter()
                    .map(|item| item.as_slice())
                    .collect();
                if let Ok(address) = Pubkey::create_program_address(&s, program_id) {
                    return (address, seeds_with_nonce);
                }
            }
            nonce[0] -= 1;
        }
        panic!("Unable to find a viable program address nonce");
    }

    pub fn derive_key(
        program_id: &Pubkey,
        seeds: &Vec<Vec<u8>>,
    ) -> Result<(Pubkey, Vec<Vec<u8>>), Error> {
        Ok(Self::find_program_address(seeds, program_id))
    }

    /// Burn a wrapped asset from account
    pub fn wrapped_burn(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        token_program_id: &Pubkey,
        token_account: &Pubkey,
        mint_account: &Pubkey,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let ix = spl_token::instruction::burn(
            token_program_id,
            token_account,
            mint_account,
            &Self::derive_bridge_id(program_id)?,
            &[],
            amount,
        )?;
        Self::invoke_as_bridge(program_id, &ix, accounts)
    }

    /// Mint a wrapped asset to account
    pub fn wrapped_mint_to(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        token_program_id: &Pubkey,
        mint: &Pubkey,
        destination: &Pubkey,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let ix = spl_token::instruction::mint_to(
            token_program_id,
            mint,
            destination,
            &Self::derive_bridge_id(program_id)?,
            &[],
            amount,
        )?;
        Self::invoke_as_bridge(program_id, &ix, accounts)
    }

    pub fn invoke_as_bridge<'a>(
        program_id: &Pubkey,
        instruction: &Instruction,
        account_infos: &[AccountInfo<'a>],
    ) -> ProgramResult {
        let (_, seeds) = Self::find_program_address(&vec!["ibport".as_bytes().to_vec()], program_id);
        Self::invoke_vec_seed(program_id, instruction, account_infos, &seeds)
    }

    pub fn invoke_vec_seed<'a>(
        program_id: &Pubkey,
        instruction: &Instruction,
        account_infos: &[AccountInfo<'a>],
        seeds: &Vec<Vec<u8>>,
    ) -> ProgramResult {
        let s: Vec<_> = seeds.iter().map(|item| item.as_slice()).collect();
        invoke_signed(instruction, account_infos, &[s.as_slice()])
    }
}
