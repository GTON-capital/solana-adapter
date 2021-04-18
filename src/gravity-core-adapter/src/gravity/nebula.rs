
#[warn(dead_code)]
pub const ORACLE_COUNT_IN_EPOCH: i8 = 5;

#[warn(dead_code)]
pub type Bytes32<'a> = &'a[u8];

pub enum DataType {
    Int64,
    String,
    Bytes
}

pub struct Subscription<A> {
    pub address: A,
    pub contract_address: A,
    pub min_confirmations: i8,
    pub reward: i64 // should be 2^256
}

pub struct Pulse<'a> {
    pub data_hash: Bytes32<'a>,
    pub height: i64
}

pub struct Oracle<'a, A> {
    pub address: A,
    pub is_online: bool,
    pub id_in_queue: Bytes32<'a>
}
