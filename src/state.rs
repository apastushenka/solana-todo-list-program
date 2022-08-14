use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TodoCounter {
    pub discriminator: String,
    pub is_initialized: bool,
    pub count: u64,
}

impl TodoCounter {
    pub const DISCRIMINATOR: &'static str = "counter";
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TodoState {
    pub discriminator: String,
    pub is_initialized: bool,
    pub index: u64,
    pub message: String,
    pub is_completed: bool,
}

impl TodoState {
    pub const DISCRIMINATOR: &'static str = "todo";
}
