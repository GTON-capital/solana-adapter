use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum NebulaError {
    /// Failed to send value to subs
    #[error("Failed to send value to subs")]
    SendValueToSubsFailed,
    #[error("Sub id exists")]
    SubscriberExists,
    #[error("Subscribe failed")]
    SubscribeFailed,

    #[error("Data provider for subscribers is invalid")]
    DataProviderForSendValueToSubsIsInvalid,

    #[error("Value has been already sent to subscriber")]
    SubscriberValueBeenSent,
    #[error("Invalid subscription id")]
    InvalidSubscriptionID,

    #[error("No such instruction index")]
    InvalidInstructionIndex,

    #[error("Invalid subscription target program id")]
    InvalidSubscriptionProgramID
}

impl From<NebulaError> for ProgramError {
    fn from(e: NebulaError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
