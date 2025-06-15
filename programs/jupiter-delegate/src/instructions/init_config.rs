use anchor_lang::prelude::*;

use crate::constants::{CONFIG_SEED, MINIMUM_TRADE_INTERVAL, VAULT_SEED};
use crate::error::ErrorCode;
use crate::event::InitConfigEvent;
use crate::state::Config;

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init_if_needed,
        payer = admin,
        space = Config::LEN,
        seeds = [CONFIG_SEED.as_bytes()],
        bump,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [VAULT_SEED.as_bytes()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn process_init_config(ctx: Context<InitConfig>, cooldown_duration: i64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require!(!config.is_initialized, ErrorCode::ConfigAlreadyInitialized);
    require!(
        cooldown_duration >= MINIMUM_TRADE_INTERVAL,
        ErrorCode::InvalidCooldownDuration
    );
    config.admin = ctx.accounts.admin.key();
    config.vault = ctx.accounts.vault.key();
    config.pending_admin = Pubkey::default();
    config.last_trade_timestamp = 0;
    config.cooldown_duration = cooldown_duration;
    config.bump = ctx.bumps.config;
    config.is_initialized = true;

    emit!(InitConfigEvent {
        admin: ctx.accounts.admin.key(),
        vault: ctx.accounts.vault.key(),
        cooldown_duration,
    });
    Ok(())
}
