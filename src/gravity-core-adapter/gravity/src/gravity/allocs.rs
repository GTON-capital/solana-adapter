use crate::gravity::error::GravityError;
use crate::gravity::instruction::GravityContractInstruction;

pub fn allocation_by_instruction_index(
    instruction: usize,
    oracles_bft: Option<usize>,
) -> Result<Vec<usize>, GravityError> {
    Ok(match instruction {
        // InitContract
        0 => vec![
            GravityContractInstruction::BFT_ALLOC,
            GravityContractInstruction::LAST_ROUND_ALLOC,
            GravityContractInstruction::PUBKEY_ALLOC * oracles_bft.unwrap(),
        ],
        // UpdateConsuls
        1 => vec![
            GravityContractInstruction::BFT_ALLOC,
            GravityContractInstruction::LAST_ROUND_ALLOC,
            GravityContractInstruction::PUBKEY_ALLOC * oracles_bft.unwrap(),
        ],
        _ => return Err(GravityError::InvalidInstructionIndex.into()),
    })
}
