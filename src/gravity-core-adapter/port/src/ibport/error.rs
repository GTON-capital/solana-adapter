use thiserror::Error;
use solana_program::program_error::ProgramError;


#[derive(Error, Debug, Copy, Clone)]
pub enum PortError {
    #[error("Invalid data on attach")]
    InvalidDataOnAttach,
    
    #[error("Invalid status")]
    InvalidRequestStatus,
    
    #[error("No such instruction index")]
    InvalidInstructionIndex,

    #[error("Access denied")]
    AccessDenied,

    #[error("Processing requests count hit limit")]
    TransferRequestsCountLimit,

    #[error("Invalid token on request create")]
    InvalidInputToken,

    #[error("Error on receiver unpack (mint)")]
    ErrorOnReceiverUnpack,

    #[error("Request id is already being processed")]
    RequestIDIsAlreadyBeingProcessed,

    #[error("Destination chain request confirmation failed: no such request ID")]
    RequestIDForConfirmationIsInvalid,

    #[error("Request amount mismatch")]
    RequestAmountMismatch,

    #[error("Request receiver mismatch")]
    RequestReceiverMismatch,

    #[error("Request status mismatch")]
    RequestStatusMismatch,

    #[error("Byte array unpack failed")]
    ByteArrayUnpackFailed,
}

impl From<PortError> for ProgramError {
    fn from(e: PortError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
