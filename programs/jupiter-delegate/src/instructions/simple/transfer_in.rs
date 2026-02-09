use crate::{
    constants::{CONFIG_SEED, VAULT_SEED},
    error::ErrorCode,
    state::Config,
};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct TransferIn<'info> {
    #[account(mut)]
    pub operator: Signer<'info>,

    /// CHECK: authority of the token account
    #[account(mut)]
    pub authority: UncheckedAccount<'info>,

    #[account()]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = token_mint,
        token::token_program = token_program,
    )]
    pub from_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds=[VAULT_SEED.as_bytes()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = operator,
        associated_token::mint = token_mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub to_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

pub fn prorcess_transfer_in(ctx: Context<TransferIn>, amounts: u64) -> Result<()> {
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

    if ctx
        .accounts
        .from_token_account
        .owner
        .eq(ctx.accounts.authority.key)
    {
        require!(
            ctx.accounts.authority.to_account_info().is_signer,
            ErrorCode::InvalidTokenAccount
        );
        require!(
            ctx.accounts.from_token_account.amount >= amounts,
            ErrorCode::InsufficientFunds
        )
    } else {
        require!(
            ctx.accounts
                .from_token_account
                .delegate
                .contains(ctx.accounts.vault.key),
            ErrorCode::InvalidDelegateTokenAccount
        );
        require!(
            ctx.accounts.from_token_account.delegated_amount >= amounts,
            ErrorCode::InsufficientFunds
        );
    }

    if ctx
        .accounts
        .from_token_account
        .owner
        .eq(ctx.accounts.authority.key)
    {
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.from_token_account.to_account_info(),
                    to: ctx.accounts.to_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                    mint: ctx.accounts.token_mint.to_account_info(),
                },
            ),
            amounts,
            ctx.accounts.token_mint.decimals,
        )?;
    } else {
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
    }

    Ok(())
}
