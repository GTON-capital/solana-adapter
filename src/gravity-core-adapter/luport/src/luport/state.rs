use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use solana_gravity_contract::gravity::state::PartialStorage;
use spl_token::state::Mint;

use gravity_misc::model::{AbstractRecordHandler, RecordHandler};
use gravity_misc::validation::TokenMintConstrained;
use gravity_misc::ports::state::{
    GenericRequest,
    GenericPortOperation,
    RequestsQueue, 
    RequestCountConstrained,
    RequestStatus,
    ForeignAddress,
    PortOperationIdentifier
};

use arrayref::array_ref;
use borsh::{BorshDeserialize, BorshSerialize};

use gravity_misc::ports::error::PortError;


pub type WrapRequest = GenericRequest<Pubkey, ForeignAddress>;


#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Default, Debug, Clone)]
pub struct LUPortContract {
    pub nebula_address: Pubkey, // distinct nebula address (not nebula data account)
    pub token_address: Pubkey,
    pub token_mint: Pubkey, // common token info, (result of spl-token create-token or as it so called - 'the mint')
    pub initializer_pubkey: Pubkey,
    pub oracles: Vec<Pubkey>,

    pub swap_status: RecordHandler<[u8; 16], RequestStatus>,
    pub requests: RecordHandler<[u8; 16], WrapRequest>,

    pub is_state_initialized: bool,

    pub requests_queue: RequestsQueue<[u8; 16]>,
}

impl RequestCountConstrained for LUPortContract {
    const MAX_IDLE_REQUESTS_COUNT: usize = 100;

    fn count_constrained_entities(&self) -> Vec<usize> {
        vec![
            self.unprocessed_burn_requests()
        ]
    }
}

impl TokenMintConstrained<PortError> for LUPortContract {

    fn bound_token_mint(&self) -> (Pubkey, PortError) {
        return (
            self.token_mint,
            PortError::InvalidTokenMint
        )
    }
}

impl PartialStorage for LUPortContract {
    const DATA_RANGE: std::ops::Range<usize> = 0..20000;
}

impl Sealed for LUPortContract {} 

impl IsInitialized for LUPortContract {
    fn is_initialized(&self) -> bool {
        self.is_state_initialized
    }
}


impl Pack for LUPortContract {
    const LEN: usize = 20000;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut mut_src: &[u8] = src;
        Self::deserialize(&mut mut_src).map_err(|err| {
            msg!(
                "Error: failed to deserialize LUPortContract instruction: {}",
                err
            );
            ProgramError::InvalidInstructionData
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let data = self.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }
}


pub type PortOperation<'a> = GenericPortOperation<'a, ForeignAddress>;

impl LUPortContract {

    fn unprocessed_burn_requests(&self) -> usize {
        self.requests.len()
    }

    fn validate_requests_count(&self) -> Result<(), PortError> {
        if !self.count_is_below_limit() {
            return Err(PortError::TransferRequestsCountLimit);
        }
        Ok(())
    }

    pub fn unpack_byte_array(byte_data: &Vec<u8>) -> Result<PortOperation, ProgramError> {
        if byte_data.len() < 57 {
            return Err(PortError::ByteArrayUnpackFailed.into());
        }

        let mut pos = 0;
        let action = byte_data[pos];
        pos += 1;

        let swap_id = array_ref![byte_data, pos, 16];
        pos += 16;
        
        let raw_amount = array_ref![byte_data, pos, 8];
        pos += 8;

        let receiver = array_ref![byte_data, pos, 32];

        return Ok(PortOperation {
            action,
            swap_id,
            amount: raw_amount,
            receiver
        });
    }

    pub fn attach_data<'a>(&mut self, byte_data: &'a Vec<u8>, input_pubkey: &'a Pubkey, input_amount: &'a mut u64, token_mint_info: &Mint) -> Result<String, ProgramError> {
        let action = &[byte_data[0]];

        let command_char = std::str::from_utf8(action).unwrap();

        match command_char {
            PortOperationIdentifier::UNLOCK => {
                let port_operation = Self::unpack_byte_array(byte_data)?;
                let swap_status = self.swap_status.get(port_operation.swap_id);

                if swap_status.is_some() {
                    return Err(PortError::InvalidRequestStatus.into());
                }

                msg!("input_pubkey.to_bytes(): {:?}", input_pubkey.to_bytes());
                msg!("port_operation.receiver: {:?}", *port_operation.receiver);

                if input_pubkey.to_bytes() != *port_operation.receiver {
                    return Err(PortError::ErrorOnReceiverUnpack.into());
                }

                *input_amount = port_operation.amount_to_u64(token_mint_info.decimals);

                self.swap_status.insert(*port_operation.swap_id, RequestStatus::Success);
            },
            _ => return Err(PortError::InvalidDataOnAttach.into())
        }
        
        Ok(String::from(command_char))
    }

    pub fn create_transfer_wrap_request(&mut self, record_id: &[u8; 16], amount: u64, sender_data_account: &Pubkey, receiver: &ForeignAddress) -> Result<(), PortError>  {
        self.validate_requests_count()?;

        if self.requests.contains_key(record_id) {
            return Err(PortError::RequestIDIsAlreadyBeingProcessed.into());
        }

        self.requests.insert(*record_id, WrapRequest {
            destination_address: *receiver,
            origin_address: *sender_data_account,
            amount
        });
        self.swap_status.insert(*record_id, RequestStatus::New);
        self.requests_queue.push(*record_id);

        Ok(())
    }
}