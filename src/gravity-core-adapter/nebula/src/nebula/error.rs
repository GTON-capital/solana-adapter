use thiserror::Error;

use solana_program::program_error::{ProgramError, PrintProgramError};
// use solana_program::decode_error::DecodeError;
// use num_traits::FromPrimitive;


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
    InvalidSubscriptionProgramID,

    #[error("Pulse id has not been persisted")]
    PulseIDHasNotBeenPersisted,

    #[error("Unsubscribe is not available")]
    UnsubscribeIsNotAvailable,

    #[error("Pulse validation order mismatch")]
    PulseValidationOrderMismatch,
}

impl From<NebulaError> for ProgramError {
    fn from(e: NebulaError) -> Self {
        ProgramError::Custom(e as u32)
    }
}


// impl PrintProgramError for NebulaError {
//     fn print<E>(&self)
//     where
//         E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
//     {
//         match self {
//             NebulaError::AlreadyInUse => msg!("Error: The account cannot be initialized because it is already being used"),
//         }
//     }
// }