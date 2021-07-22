use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
};
// use std::mem::size_of;
// use arrayref::array_ref;

use crate::ports::error::PortError::InvalidInstructionIndex as InvalidInstruction;

pub enum SubscriberInstruction {
    AttachValue {
        byte_data: Vec<u8>,
    }
}

impl SubscriberInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            // AttachValue
            2 => {
                let byte_data = rest.to_vec();

                Self::AttachValue { byte_data }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }
}

impl SubscriberInstruction {
    pub fn pack(&self) -> Vec<u8> {
        // let buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::AttachValue {
                ref byte_data,
            } => {
                let mut buf = byte_data.clone();
                buf.insert(0, 2);
                buf
            },
        }
    }
}


pub fn attach_value(
    byte_data: &Vec<u8>,
    oracle: &Pubkey,
    subscriber_data_account: &Pubkey,
    target_program_id: &Pubkey, 
    token_program_id: &Pubkey, // actually spl_token::id()
    mint: &Pubkey, // actually the result of spl-token create-token (cli)
    recipient_account: &Pubkey,
    ibport_pda_account: &Pubkey,
    signer_pubkeys: &[&Pubkey],
) -> Result<Instruction, ProgramError> {
    let data = SubscriberInstruction::AttachValue { byte_data: byte_data.clone()  }.pack();

    let mut accounts = Vec::with_capacity(6 + signer_pubkeys.len());
    accounts.push(AccountMeta::new_readonly(*oracle, true));
    accounts.push(AccountMeta::new(*subscriber_data_account, false));
    accounts.push(AccountMeta::new_readonly(*token_program_id, false));
    accounts.push(AccountMeta::new(*mint, false));
    accounts.push(AccountMeta::new(*recipient_account, false));
    accounts.push(AccountMeta::new_readonly(*ibport_pda_account, false));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *target_program_id,
        accounts,
        data,
    })
}