use anchor_lang::prelude::*;

use crate::constants::{CONFIG_SEED, MINIMUM_TRADE_INTERVAL};
use crate::error::ErrorCode;
use crate::event::ModifyCooldownDurationEvent;
use crate::state::Config;

#[derive(Accounts)]
pub struct ModifyCooldownDuration<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()],
        bump = config.bump,
        constraint = config.admin == admin.key() @ ErrorCode::OnlyAdminCanModifyCooldownDuration,
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn process_modify_cooldown_duration(ctx: Context<ModifyCooldownDuration>, cooldown_duration: i64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(cooldown_duration >= MINIMUM_TRADE_INTERVAL, ErrorCode::InvalidCooldownDuration);
    config.cooldown_duration = cooldown_duration;
    emit!(ModifyCooldownDurationEvent {
        config: config.key(),
        cooldown_duration,
    });
    Ok(())
}