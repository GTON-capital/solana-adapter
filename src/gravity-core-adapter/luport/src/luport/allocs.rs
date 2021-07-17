use gravity_misc::ports::error::PortError;
use crate::luport::instruction::LUPortContractInstruction;

pub fn allocation_by_instruction_index(
    instruction: usize,
    _oracles_bft: Option<usize>,
) -> Result<Vec<usize>, PortError> {
    Ok(match instruction {
        // InitContract
        0 => vec![
            LUPortContractInstruction::PUBKEY_ALLOC,
            LUPortContractInstruction::PUBKEY_ALLOC,
            LUPortContractInstruction::PUBKEY_ALLOC,
            1,
        ],
        // CreateTransferUnwrapRequest
        1 => vec![
            LUPortContractInstruction::DEST_AMOUNT_ALLOC,
            LUPortContractInstruction::FOREIGN_ADDRESS_ALLOC,
            16,
        ],
        // AttachValue
        2 => vec![LUPortContractInstruction::ATTACHED_DATA_ALLOC],
        // ConfirmDestinationChainRequest
        3 => vec![LUPortContractInstruction::ATTACHED_DATA_ALLOC],
        // TransferTokenOwnership
        4 => vec![
            LUPortContractInstruction::PUBKEY_ALLOC,
            LUPortContractInstruction::PUBKEY_ALLOC,
        ],
        _ => return Err(PortError::InvalidInstructionIndex.into()),
    })
}
