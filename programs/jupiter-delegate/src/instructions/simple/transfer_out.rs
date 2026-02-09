use crate::{constants::VAULT_SEED, error::ErrorCode, state::Config};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct TransferOut<'info> {
    #[account(mut)]
    pub operator: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        seeds=[VAULT_SEED.as_bytes()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        associated_token::mint = token_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub from_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = token_mint,
        token::token_program = token_program,
    )]
    pub to_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn prorcess_transfer_out(ctx: Context<TransferOut>, amounts: u64) -> Result<()> {
    require!(
        ctx.accounts.operator.key() == ctx.accounts.config.operator
            || ctx.accounts.operator.key() == ctx.accounts.config.admin,
        ErrorCode::InvalidOperator
    );
    require!(
        ctx.accounts.config.is_initialized,
        ErrorCode::ConfigNotInitialized
    );
    require!(!ctx.accounts.config.is_paused, ErrorCode::ConfigPaused);

    require!(
        ctx.accounts.from_token_account.amount >= amounts,
        ErrorCode::InsufficientFunds
    );

    // 检查冷却时间
    let now = Clock::get()?.unix_timestamp;
    require!(
        ctx.accounts
            .config
            .last_trade_timestamp
            .checked_add(ctx.accounts.config.cooldown_duration)
            .expect("overflow")
            < now,
        ErrorCode::SwapTooFrequent
    );
    ctx.accounts.config.last_trade_timestamp = now;

    // 4. 从 delegate 转账到 vault
    let signed_seeds = &[VAULT_SEED.as_bytes(), &[ctx.bumps.vault]];
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.from_token_account.to_account_info(),
                to: ctx.accounts.to_token_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
            },
            &[signed_seeds],
        ),
        amounts,
        ctx.accounts.token_mint.decimals,
    )?;

    Ok(())
}
