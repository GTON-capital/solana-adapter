use std::str::FromStr;
use solana_program::{declare_id, pubkey::Pubkey};

// declare_id!("nVZnRKdr3pmcgnJvYDE8iafgiMiBqxiffQMcyv5ETdA");

// let my_id = Pubkey::from_str("nVZnRKdr3pmcgnJvYDE8iafgiMiBqxiffQMcyv5ETdA").unwrap();
// assert_eq!(id(), my_id);


// declare_id()
pub fn susy_wrapped_gton_mint() -> Pubkey {
    Pubkey::from_str("nVZnRKdr3pmcgnJvYDE8iafgiMiBqxiffQMcyv5ETdA").unwrap()
}
