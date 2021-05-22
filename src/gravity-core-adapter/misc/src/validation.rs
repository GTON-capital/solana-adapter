use std::error;
use solana_program::program_error::ProgramError;

pub fn is_contract_empty(target_contract: &[u8]) -> bool {
    for byte in target_contract.iter() {
        if *byte != 0 {
            return false;
        }
    }

    return true;
}

pub fn validate_contract_non_emptiness(target_contract: &[u8]) -> Result<(), ProgramError> {
    if is_contract_empty(target_contract) {
        return Err(ProgramError::UninitializedAccount);
    }

    Ok(())
}

pub fn validate_contract_emptiness(target_contract: &[u8]) -> Result<(), ProgramError> {
    if !is_contract_empty(target_contract) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}