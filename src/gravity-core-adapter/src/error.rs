use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum GravityError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Input Round <= Last Round
    #[error("Input round is less or equal than last round")]
    InputRoundMismatch,
    /// Input Bft < Target Bft
    #[error("Invalid bft count")]
    InvalidBFTCount,
}

#[derive(Error, Debug, Copy, Clone)]
pub enum NebulaError {
    /// Failed to send value to subs
    #[error("Failed to send value to subs")]
    SendValueToSubsFailed
}

impl From<GravityError> for ProgramError {
    fn from(e: GravityError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
