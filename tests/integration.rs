#![cfg(feature = "test-bpf")]

use solana_program::borsh::try_from_slice_unchecked;
use todo_list_program::state::TodoCounter;

use {
    assert_matches::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    solana_sdk::{signature::Signer, system_program, transaction::Transaction},
    solana_validator::test_validator::*,
};

#[test]
fn test_program() {
    solana_logger::setup_with_default("solana_runtime::message=debug");

    let program_id = Pubkey::new_unique();

    let (test_validator, payer) = TestValidatorGenesis::default()
        .add_program("todo_list_program", program_id)
        .start();

    let rpc_client = test_validator.get_rpc_client();

    let (pda_counter, _) = Pubkey::find_program_address(&[payer.pubkey().as_ref()], &program_id);

    let mut transaction = Transaction::new_with_payer(
        &[Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(pda_counter, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![0],
        }],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], rpc_client.get_latest_blockhash().unwrap());

    assert_matches!(rpc_client.send_and_confirm_transaction(&transaction), Ok(_));

    let counter_data = rpc_client.get_account_data(&pda_counter).unwrap();
    let counter = try_from_slice_unchecked::<TodoCounter>(&counter_data).unwrap();

    assert_eq!(counter.discriminator, TodoCounter::DISCRIMINATOR);
    assert_eq!(counter.is_initialized, true);
    assert_eq!(counter.count, 0);
}
