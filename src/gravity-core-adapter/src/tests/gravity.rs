// Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
#![cfg(feature = "test-bpf")]

use {
    crate::entrypoint::process_instruction,
    crate::gravity::profcgessor::GravityProcessor as Processor,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        rent::Rent,
        system_program,
    },
    solana_program_test::*,
    solana_sdk::{account::Account, signature::Signer, transaction::Transaction},
    // gravity_core_adapter::gravity::state::SIZE,
    std::str::FromStr,
};

const SIZE: usize = 2000;

#[tokio::test]
async fn test_gravity_contract_instantiation() {
    let program_id = Pubkey::from_str(&"invoker111111111111111111111111111111111111").unwrap();

    // let process_instruction = &Processor::process_gravity_contract;

    let mocked_seed = "just a seed";
    let (allocated_pubkey, bump_seed) =
        Pubkey::find_program_address(&[mocked_seed.as_bytes()], &program_id);

    let mut program_test = ProgramTest::new(
        "gravity_core_adapter",
        program_id,
        processor!(process_instruction),
    );
    program_test.add_account(
        allocated_pubkey,
        Account {
            lamports: Rent::default().minimum_balance(SIZE),
            ..Account::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[bump_seed],
            vec![
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(allocated_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Associated account now exists
    let allocated_account = banks_client
        .get_account(allocated_pubkey)
        .await
        .expect("get_account")
        .expect("associated_account not none");
    assert_eq!(allocated_account.data.len(), SIZE);
}
