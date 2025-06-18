pub mod constants;
pub mod error;
pub mod event;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use event::*;
pub use instructions::*;
pub use state::*;

use anchor_lang::prelude::*;
use std::str::FromStr;

declare_id!("66LUuou5irvF3zgbJSMZbpPaCTiTuB5RCnBHHa1LihB2");

pub fn jupiter_program_id() -> Pubkey {
    Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap()
}

#[program]
pub mod jupiter_delegate {
    use super::*;

    pub fn init_config(
        ctx: Context<InitConfig>,
        operator: Pubkey,
        cooldown_duration: i64,
    ) -> Result<()> {
        process_init_config(ctx, operator, cooldown_duration)
    }

    pub fn grant_access(ctx: Context<GrantAccess>, user: Pubkey) -> Result<()> {
        process_grant_access(ctx, user)
    }

    pub fn revoke_access(ctx: Context<RevokeAccess>, user: Pubkey) -> Result<()> {
        process_revoke_access(ctx, user)
    }

    pub fn swap(ctx: Context<Swap>, params: SwapParams) -> Result<()> {
        process_swap(ctx, params)
    }

    pub fn propose_new_admin(ctx: Context<ProposeNewAdmin>) -> Result<()> {
        process_propose_new_admin(ctx)
    }

    pub fn accept_admin_transfer(ctx: Context<AcceptAdminTransfer>) -> Result<()> {
        process_accept_admin_transfer(ctx)
    }

    pub fn modify_cooldown_duration(ctx: Context<ModifyCooldownDuration>, cooldown_duration: i64) -> Result<()> {
        process_modify_cooldown_duration(ctx, cooldown_duration)
    }

    pub fn pause(ctx: Context<Pause>, toggle: bool) -> Result<()> {
        process_pause(ctx, toggle)
    }
}
