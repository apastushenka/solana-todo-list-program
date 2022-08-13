use crate::instruction::TodoInstruction;
use crate::state::TodoCounter;

use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match TodoInstruction::unpack(instruction_data)? {
        TodoInstruction::InitTodoList => init_todo_list(program_id, accounts),
    }
}

fn init_todo_list(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    msg!("Initialize Todo List...");

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let pda_counter = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if system_program.key != &solana_program::system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref()], program_id);
    if pda_counter.key != &pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Calculate rent
    let account_len = 4 + TodoCounter::DISCRIMINATOR.len() + 1 + 8;
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    msg!("Create PDA account...");
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_counter.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_counter.clone(),
            system_program.clone(),
        ],
        &[&[initializer.key.as_ref(), &[bump_seed]]],
    )?;

    let mut account_data = try_from_slice_unchecked::<TodoCounter>(&pda_counter.data.borrow())?;
    account_data.discriminator = TodoCounter::DISCRIMINATOR.to_owned();
    account_data.is_initialized = true;
    account_data.count = 0;

    account_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;

    Ok(())
}
