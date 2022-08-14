use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum TodoInstruction {
    InitTodoList,
    AddTodo { message: String },
}

#[derive(BorshDeserialize)]
struct AddTodoPayload {
    message: String,
}

impl TodoInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        match tag {
            0 => Ok(TodoInstruction::InitTodoList),
            1 => {
                let payload = AddTodoPayload::try_from_slice(rest).unwrap();
                Ok(TodoInstruction::AddTodo {
                    message: payload.message,
                })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
