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
