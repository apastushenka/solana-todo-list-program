#![cfg(feature = "test-bpf")]

use todo_list_program::state::{TodoCounter, TodoState};

use {
    assert_matches::*,
    solana_client::rpc_client::RpcClient,
    solana_program::{
        borsh::try_from_slice_unchecked,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    solana_sdk::{signature::Keypair, signature::Signer, system_program, transaction::Transaction},
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

    init_todo_list(&rpc_client, &program_id, &payer);

    add_todo(&rpc_client, &program_id, &payer, 0, "ToDo");
}

fn init_todo_list(rpc_client: &RpcClient, program_id: &Pubkey, payer: &Keypair) {
    let (pda_counter, _) = Pubkey::find_program_address(&[payer.pubkey().as_ref()], program_id);

    let mut transaction = Transaction::new_with_payer(
        &[Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(pda_counter, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![0],
        }],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer], rpc_client.get_latest_blockhash().unwrap());

    assert_matches!(rpc_client.send_and_confirm_transaction(&transaction), Ok(_));

    let counter_data = rpc_client.get_account_data(&pda_counter).unwrap();
    let counter = try_from_slice_unchecked::<TodoCounter>(&counter_data).unwrap();

    assert_eq!(counter.discriminator, TodoCounter::DISCRIMINATOR);
    assert_eq!(counter.is_initialized, true);
    assert_eq!(counter.count, 0);
}

fn add_todo(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
    payer: &Keypair,
    todo_index: u64,
    todo_message: &str,
) {
    let (pda_counter, _) = Pubkey::find_program_address(&[payer.pubkey().as_ref()], program_id);

    let (pda_todo, _) = Pubkey::find_program_address(
        &[payer.pubkey().as_ref(), todo_index.to_be_bytes().as_ref()],
        program_id,
    );

    let mut transaction = Transaction::new_with_payer(
        &[Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(pda_counter, false),
                AccountMeta::new(pda_todo, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: {
                let mut vec = vec![1];
                vec.append(&mut borsh::to_vec(todo_message).unwrap());
                vec
            },
        }],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[payer], rpc_client.get_latest_blockhash().unwrap());

    assert_matches!(rpc_client.send_and_confirm_transaction(&transaction), Ok(_));

    let todo_data = rpc_client.get_account_data(&pda_todo).unwrap();
    let todo = try_from_slice_unchecked::<TodoState>(&todo_data).unwrap();

    assert_eq!(todo.discriminator, TodoState::DISCRIMINATOR);
    assert_eq!(todo.is_initialized, true);
    assert_eq!(todo.index, todo_index);
    assert_eq!(todo.message, todo_message);
    assert_eq!(todo.is_completed, false);

    let counter_data = rpc_client.get_account_data(&pda_counter).unwrap();
    let counter = try_from_slice_unchecked::<TodoCounter>(&counter_data).unwrap();

    assert_eq!(counter.discriminator, TodoCounter::DISCRIMINATOR);
    assert_eq!(counter.is_initialized, true);
    assert_eq!(counter.count, 1);
}
