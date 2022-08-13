use solana_program::program_error::ProgramError;

pub enum TodoInstruction {
    InitTodoList,
}

impl TodoInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, _rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        match tag {
            0 => Ok(TodoInstruction::InitTodoList),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
