use anchor_lang::prelude::*;

use crate::constants::ACCESS_SEED;
use crate::error::ErrorCode;
use crate::event::{GrantAccessEvent, RevokeAccessEvent};
use crate::state::{Access, Config};

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct GrantAccess<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        has_one = admin,
    )]
    pub config: Account<'info, Config>,
    #[account(
        init_if_needed,
        payer = admin,
        space = Access::LEN,
        seeds = [ACCESS_SEED.as_bytes(), user.as_ref()],
        bump,
    )]
    pub access: Account<'info, Access>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn process_grant_access(ctx: Context<GrantAccess>, user: Pubkey) -> Result<()> {
    let access = &mut ctx.accounts.access;
    require!(!access.is_granted, ErrorCode::AccessAlreadyGranted);
    access.user = user;
    access.is_granted = true;
    access.bump = ctx.bumps.access;

    emit!(GrantAccessEvent {
        user,
        access: ctx.accounts.access.key(),
    });
    Ok(())
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct RevokeAccess<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        has_one = admin,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        close = admin,
        seeds = [ACCESS_SEED.as_bytes(), user.as_ref()],
        bump,
    )]
    pub access: Account<'info, Access>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn process_revoke_access(ctx: Context<RevokeAccess>, user: Pubkey) -> Result<()> {
    let access = &mut ctx.accounts.access;
    require!(access.is_granted, ErrorCode::AccessNotGranted);
    access.is_granted = false;

    emit!(RevokeAccessEvent {
        user,
        access: ctx.accounts.access.key(),
    });
    Ok(())
}
