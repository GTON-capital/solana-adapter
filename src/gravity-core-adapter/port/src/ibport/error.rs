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
}

impl From<PortError> for ProgramError {
    fn from(e: PortError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
