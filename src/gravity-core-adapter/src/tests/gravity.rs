// use std::mem;

// use byteorder::{ByteOrder, LittleEndian};
// use solana_bpf_helloworld::process_instruction;
// use solana_program_test::*;
// use solana_sdk::{
//     account::Account,
//     instruction::{AccountMeta, Instruction},
//     pubkey::Pubkey,
//     signature::Signer,
//     transaction::Transaction,
// };

// use crate::state;


// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_gravity_contract() {
//         let program_id = Pubkey::new_unique();
//         let gravity_contract_pubkey = Pubkey::new_unique();

//         let mut program_test = ProgramTest::new(
//             "solana-gravity-adaptor", // Run the BPF version with `cargo test-bpf`
//             program_id,
//             processor!(process_instruction), // Run the native version with `cargo test`
//         );

//         program_test.add_account(
//             gravity_contract_pubkey,
//             Account {
//                 lamports: 5,
//                 data: vec![0_u8; mem::size_of::<u32>()],
//                 owner: program_id,
//                 ..Account::default()
//             },
//         );
//         let (mut banks_client, payer, recent_blockhash) = program_test.start().await;


//         // fetch gravity contract state
//         let gravity_contract_account = banks_client
//             .get_account(gravity_contract_pubkey)
//             .await
//             .expect("get_account")
//             .expect("greeted_account not found");
        
//         // verify that gravity contract is empty
//         assert!(&gravity_contract_account.data.iter().fold(true, |acc, x| { acc && *x == 0 }));

//         // verify ser/deserialization

//         let mock_gravity_consuls = vec![
//             Pubkey::new_unique(),
//             Pubkey::new_unique(),
//             Pubkey::new_unique(),
//         ];
//         let mock_bft: u8 = 3;

//         let gravity_contract_mock = state::GravityContract {
//             consuls: mock_gravity_consuls.clone(),
//             bft: mock_bft,
//             ..state::GravityContract::default()
//         };
//         // let serialize

//         let mut transaction = Transaction::new_with_payer(
//             &[Instruction::new_with_bincode(
//                 program_id,
//                 &[0], // ignored but makes the instruction unique in the slot
//                 vec![
//                     AccountMeta::new(greeted_pubkey, false)
//                 ],
//             )],
//             Some(&payer.pubkey()),
//         );
//         transaction.sign(&[&payer], recent_blockhash);
//         banks_client.process_transaction(transaction).await.unwrap();

//         // Verify account has one greeting
//         let greeted_account = banks_client
//             .get_account(greeted_pubkey)
//             .await
//             .expect("get_account")
//             .expect("greeted_account not found");
//         assert_eq!(LittleEndian::read_u32(&greeted_account.data), 1);

//         // Greet again
//         let mut transaction = Transaction::new_with_payer(
//             &[Instruction::new_with_bincode(
//                 program_id,
//                 &[1], // ignored but makes the instruction unique in the slot
//                 vec![AccountMeta::new(greeted_pubkey, false)],
//             )],
//             Some(&payer.pubkey()),
//         );
//         transaction.sign(&[&payer], recent_blockhash);
//         banks_client.process_transaction(transaction).await.unwrap();

//         // Verify account has two greetings
//         let greeted_account = banks_client
//             .get_account(greeted_pubkey)
//             .await
//             .expect("get_account")
//             .expect("greeted_account not found");
//         assert_eq!(LittleEndian::read_u32(&greeted_account.data), 2);
//     }

// }