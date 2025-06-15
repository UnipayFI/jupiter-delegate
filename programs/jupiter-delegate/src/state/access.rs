use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Access {
    pub user: Pubkey,
    pub is_granted: bool,
    pub bump: u8,
}

impl Access {
    pub const LEN: usize = 8 + Self::INIT_SPACE;
}
