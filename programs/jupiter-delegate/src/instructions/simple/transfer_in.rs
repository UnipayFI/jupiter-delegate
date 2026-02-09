use crate::{constants::VAULT_SEED, error::ErrorCode};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct TransferIn<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account()]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = token_mint,
        token::token_program = token_program,
    )]
    pub from_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds=[VAULT_SEED.as_bytes()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
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
    if ctx
        .accounts
        .from_token_account
        .owner
        .eq(ctx.accounts.authority.key)
    {
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

    Ok(())
}
