use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub operator: Pubkey,
    pub vault: Pubkey,
    pub pending_admin: Pubkey,
    pub last_trade_timestamp: i64, // last trade timestamp
    pub is_initialized: bool,
    pub is_paused: bool,
    pub cooldown_duration: i64, // cooldown duration in seconds
    pub bump: u8,
}

impl Config {
    pub const LEN: usize = 8 + Self::INIT_SPACE;
}
