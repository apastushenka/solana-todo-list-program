use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum TodoInstruction {
    InitTodoList,
    AddTodo { message: String },
    MarkCompleted { index: u64 },
}

#[derive(BorshDeserialize)]
struct AddTodoPayload {
    message: String,
}

#[derive(BorshDeserialize)]
struct MarkCompletedPayload {
    index: u64,
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
            2 => {
                let payload = MarkCompletedPayload::try_from_slice(rest).unwrap();
                Ok(TodoInstruction::MarkCompleted {
                    index: payload.index,
                })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
