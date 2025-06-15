use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use jupiter_override::program::Jupiter;

declare_program!(jupiter_override);

use crate::constants::{ACCESS_SEED, VAULT_SEED};
use crate::error::ErrorCode;
use crate::event::SwapEvent;
use crate::state::{Access, Config};
use crate::jupiter_program_id;

#[derive(Accounts)]
#[instruction(in_amount: u64)]
pub struct Swap<'info> {
    pub input_mint: InterfaceAccount<'info, Mint>,
    pub input_mint_program: Interface<'info, TokenInterface>,
    pub output_mint: InterfaceAccount<'info, Mint>,
    pub output_mint_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        seeds=[VAULT_SEED.as_bytes()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        associated_token::mint = input_mint,
        associated_token::authority = vault,
        associated_token::token_program = input_mint_program,
    )]
    pub vault_input_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = vault,
        associated_token::token_program = output_mint_program,
    )]
    pub vault_output_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub config: Account<'info, Config>,

    /// CHECK: The authority that has delegated its tokens to the vault. Not a signer.
    pub delegate: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = input_mint,
        associated_token::authority = delegate,
        constraint = delegate_input_token_account.delegate.contains(&vault.key()) @ ErrorCode::DelegateNotApproved,
        constraint = delegate_input_token_account.delegated_amount >= in_amount @ ErrorCode::InsufficientDelegatedAmount
    )]
    pub delegate_input_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: The beneficiary of the swap. Not a signer.
    pub user: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = user,
        associated_token::token_program = output_mint_program,
    )]
    pub user_output_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [ACCESS_SEED.as_bytes(), user.key().as_ref()],
        has_one = user,
        bump,
    )]
    pub access: Account<'info, Access>,

    pub jupiter_program: Program<'info, Jupiter>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_swap(ctx: Context<Swap>, in_amount: u64, data: Vec<u8>) -> Result<()> {
    require!(ctx.accounts.access.is_granted, ErrorCode::AccessNotGranted);
    require!(
        ctx.accounts.config.is_initialized,
        ErrorCode::ConfigNotInitialized
    );

    let config = &mut ctx.accounts.config;
    let now = Clock::get()?.unix_timestamp;
    require!(
        config
            .last_trade_timestamp
            .checked_add(config.cooldown_duration)
            .expect("overflow")
            < now,
        ErrorCode::SwapTooFrequent
    );
    config.last_trade_timestamp = now;

    let vault_seeds = &[VAULT_SEED.as_bytes(), &[ctx.bumps.vault]];

    // 1. Transfer from delegate to vault, using the vault's delegated authority
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.input_mint_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.delegate_input_token_account.to_account_info(),
                to: ctx.accounts.vault_input_token_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.input_mint.to_account_info(),
            },
            &[vault_seeds],
        ),
        in_amount,
        ctx.accounts.input_mint.decimals,
    )?;

    // 2. CPI to Jupiter
    let before_balance = ctx.accounts.vault_output_token_account.amount;
    require_keys_eq!(*ctx.accounts.jupiter_program.key, jupiter_program_id());
    let accounts: Vec<AccountMeta> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| {
            let is_signer = acc.key == &ctx.accounts.vault.key();
            AccountMeta {
                pubkey: *acc.key,
                is_signer,
                is_writable: acc.is_writable,
            }
        })
        .collect();

    let accounts_infos: Vec<AccountInfo> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    invoke_signed(
        &Instruction {
            program_id: ctx.accounts.jupiter_program.key(),
            accounts,
            data,
        },
        &accounts_infos,
        &[vault_seeds],
    )?;

    // 3. Transfer from vault back to user
    ctx.accounts.vault_output_token_account.reload()?;
    let after_balance = ctx.accounts.vault_output_token_account.amount;
    require!(after_balance > before_balance, ErrorCode::SwapFailed);

    let diff_amount = after_balance - before_balance;

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.output_mint_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_output_token_account.to_account_info(),
                to: ctx.accounts.user_output_token_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.output_mint.to_account_info(),
            },
            &[vault_seeds],
        ),
        diff_amount,
        ctx.accounts.output_mint.decimals,
    )?;

    emit!(SwapEvent {
        user: ctx.accounts.user.key(),
        user_output_token_account: ctx.accounts.user_output_token_account.key(),
        amount: diff_amount,
    });

    Ok(())
}
