use crate::nebula::error::NebulaError;
use crate::nebula::instruction::NebulaContractInstruction;

pub fn allocation_by_instruction_index(
    instruction: usize,
    oracles_bft: Option<usize>,
) -> Result<Vec<usize>, NebulaError> {
    Ok(match instruction {
        // InitContract
        0 => vec![
            NebulaContractInstruction::BFT_ALLOC,
            NebulaContractInstruction::DATA_TYPE_ALLOC_RANGE,
            NebulaContractInstruction::PUBKEY_ALLOC,
            NebulaContractInstruction::PUBKEY_ALLOC * oracles_bft.unwrap(),
        ],
        // UpdateOracles
        1 => vec![
            NebulaContractInstruction::BFT_ALLOC,
            NebulaContractInstruction::PUBKEY_ALLOC * oracles_bft.unwrap(),
            NebulaContractInstruction::PULSE_ID_ALLOC,
        ],
        // SendHashValue
        2 => vec![NebulaContractInstruction::DATA_HASH_ALLOC],
        // SendValueToSubs
        3 => vec![
            NebulaContractInstruction::DATA_HASH_ALLOC,
            NebulaContractInstruction::DATA_TYPE_ALLOC_RANGE,
            NebulaContractInstruction::PULSE_ID_ALLOC,
            NebulaContractInstruction::SUB_ID_ALLOC,
        ],
        // Subscribe
        4 => vec![
            NebulaContractInstruction::PUBKEY_ALLOC, 
            1,
            8,
            NebulaContractInstruction::SUB_ID_ALLOC,
        ],
        5 => vec![
            NebulaContractInstruction::SUB_ID_ALLOC,
        ],
        _ => return Err(NebulaError::InvalidInstructionIndex.into()),
    })
}
