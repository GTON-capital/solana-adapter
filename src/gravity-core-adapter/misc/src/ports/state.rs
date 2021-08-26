use borsh::{BorshDeserialize, BorshSerialize};
use arrayref::array_ref;


#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum RequestStatus {
    None,
    New,
    Rejected,
    Success,
}

impl Default for RequestStatus {
    fn default() -> Self {
        RequestStatus::None
    }
}

impl RequestStatus {
    pub fn from_u8(input: u8) -> Option<RequestStatus> {
        Some(match input {
            0 => RequestStatus::None,
            1 => RequestStatus::New,
            2 => RequestStatus::Rejected,
            3 => RequestStatus::Success,
            _ => return None
        })
    }
}

pub trait PortQueue<T> {
    fn drop_selected(&mut self, inp: T) -> Option<T>;
}

pub type RequestsQueue<T> = Vec<T>;

impl<T: PartialEq> PortQueue<T> for RequestsQueue<T> {
    fn drop_selected(&mut self, input: T) -> Option<T> {
        for (i, x) in self.iter().enumerate() {
            if *x == input {
                return Some(self.remove(i));
            }
        }
        None
    }
}

pub trait RequestCountConstrained {
    const MAX_IDLE_REQUESTS_COUNT: usize;

    fn unprocessed_requests_limit() -> usize {
        Self::MAX_IDLE_REQUESTS_COUNT
    }                                                                                                                                                                                                             

    fn count_constrained_entities(&self) -> Vec<usize>;

    fn count_is_below_limit(&self) -> bool {
        let entities = self.count_constrained_entities();

        for entity_len in entities {
            if entity_len >= Self::unprocessed_requests_limit() {
                return false
            }
        }
        return true
    }
}

pub type ForeignAddress = [u8; 32];

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Default, Copy)]
pub struct GenericRequest<O, D> {
    pub destination_address: D,
    pub origin_address: O,
    pub amount: u64
}


pub struct GenericPortOperation<'a, R> {
    pub action: u8,
    pub swap_id: &'a [u8; 16],
    pub amount: &'a [u8; 8],
    pub receiver: &'a R,
}

impl<'a, R> GenericPortOperation<'a, R> {
    // pub fn decimals() -> u8 {
    //     8
    // }

    pub fn amount_to_f64(&self) -> f64 {
        let raw_amount = array_ref![self.amount, 0, 8];
        f64::from_le_bytes(*raw_amount)
    }

    pub fn amount_to_u64(&self, decimals: u8) -> u64 {
        // let decimals = Self::decimals();
        spl_token::ui_amount_to_amount(self.amount_to_f64(), decimals)
    }
}


pub struct PortOperationIdentifier;

impl<'a> PortOperationIdentifier {
    pub const MINT: &'a str = "m";
    pub const UNLOCK: &'a str = "u";
    pub const CONFIRM: &'a str = "c";
}

// unsafe variant
// impl PortOperationIdentifier {
//     pub const MINT: *const str = "m";
//     pub const UNLOCK: *const str = "u";
// }