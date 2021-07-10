use std::str::FromStr;
use solana_program::{declare_id, pubkey::Pubkey};

// declare_id!("nVZnRKdr3pmcgnJvYDE8iafgiMiBqxiffQMcyv5ETdA");

pub fn susy_wrapped_gton_mint() -> Pubkey {
    Pubkey::from_str("nVZnRKdr3pmcgnJvYDE8iafgiMiBqxiffQMcyv5ETdA").unwrap()
}
