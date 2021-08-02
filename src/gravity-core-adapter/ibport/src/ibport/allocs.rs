use gravity_misc::ports::error::PortError;
use crate::ibport::instruction::IBPortContractInstruction;

pub fn allocation_by_instruction_index(
    instruction: usize,
    _oracles_bft: Option<usize>,
) -> Result<Vec<usize>, PortError> {
    Ok(match instruction {
        // InitContract
        0 => vec![
            IBPortContractInstruction::PUBKEY_ALLOC,
            IBPortContractInstruction::PUBKEY_ALLOC,
            IBPortContractInstruction::PUBKEY_ALLOC,
            1,
        ],
        // CreateTransferUnwrapRequest
        1 => vec![
            IBPortContractInstruction::DEST_AMOUNT_ALLOC,
            IBPortContractInstruction::FOREIGN_ADDRESS_ALLOC,
            16,
        ],
        // AttachValue
        2 => vec![IBPortContractInstruction::ATTACHED_DATA_ALLOC],
        // ConfirmDestinationChainRequest
        3 => vec![IBPortContractInstruction::ATTACHED_DATA_ALLOC],
        // TransferTokenOwnership
        4 => vec![
            IBPortContractInstruction::PUBKEY_ALLOC,
            IBPortContractInstruction::PUBKEY_ALLOC,
        ],
        _ => return Err(PortError::InvalidInstructionIndex.into()),
    })
}
