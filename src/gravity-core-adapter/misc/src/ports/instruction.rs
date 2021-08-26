use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
};
use solana_program::account_info::AccountInfo;


use crate::ports::error::PortError::InvalidInstructionIndex as InvalidInstruction;

pub enum SubscriberInstruction {
    AttachValue {
        byte_data: Vec<u8>,
    }
}

pub const ATTACH_VALUE_INSTRUCTION_INDEX: &u8 = &2;

impl SubscriberInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            // AttachValue
            ATTACH_VALUE_INSTRUCTION_INDEX => {
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
                buf.insert(0, *ATTACH_VALUE_INSTRUCTION_INDEX);
                buf
            },
        }
    }
}


pub fn attach_value<'a>(
    byte_data: &Vec<u8>,
    oracle: &Pubkey,
    subscriber_data_account: &Pubkey,
    target_program_id: &Pubkey, 
    token_program_id: &Pubkey, // actually spl_token::id()
    mint: &Pubkey, // actually the result of spl-token create-token (cli)
    recipient_account: &Pubkey,
    ibport_pda_account: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    // additional_data: &[&Pubkey],
    additional_data: &[AccountInfo<'a>],
) -> Result<Instruction, ProgramError> {
    let data = SubscriberInstruction::AttachValue { byte_data: byte_data.clone()  }.pack();

    let mut accounts = Vec::with_capacity(6 + signer_pubkeys.len() + additional_data.len());
    accounts.push(AccountMeta::new_readonly(*oracle, true));
    accounts.push(AccountMeta::new(*subscriber_data_account, false));
    accounts.push(AccountMeta::new_readonly(*token_program_id, false));
    accounts.push(AccountMeta::new(*mint, false));
    accounts.push(AccountMeta::new(*recipient_account, false));
    accounts.push(AccountMeta::new_readonly(*ibport_pda_account, false));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    for additional_data_account in additional_data.iter() {
        // accounts.push(AccountMeta::new(**additional_data_account, false));
        let acc = if additional_data_account.is_writable {
            AccountMeta::new(*additional_data_account.key, additional_data_account.is_signer)
        } else {
            AccountMeta::new_readonly(*additional_data_account.key, additional_data_account.is_signer)
        };
        accounts.push(acc);
    }

    Ok(Instruction {
        program_id: *target_program_id,
        accounts,
        data,
    })
}