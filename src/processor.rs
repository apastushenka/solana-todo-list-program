use crate::state::TodoCounter;
use crate::{instruction::TodoInstruction, state::TodoState};

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
        TodoInstruction::AddTodo { message } => add_todo(program_id, accounts, message),
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

fn add_todo(program_id: &Pubkey, accounts: &[AccountInfo], message: String) -> ProgramResult {
    msg!("Add Todo Item...");

    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;
    let pda_counter = next_account_info(account_info_iter)?;
    let pda_todo = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if pda_counter.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    if system_program.key != &solana_program::system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (pda, _) = Pubkey::find_program_address(&[initializer.key.as_ref()], program_id);
    if pda_counter.key != &pda {
        return Err(ProgramError::InvalidSeeds);
    }

    msg!("Deserialize TodoCounter...");
    let mut counter = try_from_slice_unchecked::<TodoCounter>(&pda_counter.data.borrow())?;
    if counter.discriminator != TodoCounter::DISCRIMINATOR {
        return Err(ProgramError::InvalidAccountData);
    }
    if !counter.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }
    msg!("TodoCounter.count = {}", counter.count);

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[
            initializer.key.as_ref(),
            counter.count.to_be_bytes().as_ref(),
        ],
        program_id,
    );
    if pda_todo.key != &pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let account_len = 4 + TodoState::DISCRIMINATOR.len() + 1 + 8 + 4 + message.len() + 1;
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    msg!("Create PDA todo");
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_todo.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_todo.clone(),
            system_program.clone(),
        ],
        &[&[
            initializer.key.as_ref(),
            counter.count.to_be_bytes().as_ref(),
            &[bump_seed],
        ]],
    )?;

    let mut todo = try_from_slice_unchecked::<TodoState>(&pda_todo.data.borrow())?;
    todo.discriminator = TodoState::DISCRIMINATOR.to_owned();
    todo.is_initialized = true;
    todo.index = counter.count;
    todo.message = message;
    todo.is_completed = false;
    todo.serialize(&mut &mut pda_todo.data.borrow_mut()[..])?;
    msg!(
        "Save todo: index = {}, message = {}",
        todo.index,
        todo.message
    );

    counter.count += 1;
    counter.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;
    msg!("Update counter: count = {}", counter.count);

    Ok(())
}
