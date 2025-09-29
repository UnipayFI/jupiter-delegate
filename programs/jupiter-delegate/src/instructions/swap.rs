use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::{
    associated_token::get_associated_token_address,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use jupiter_aggregator::program::Jupiter;

declare_program!(jupiter_aggregator);

use crate::error::ErrorCode;
use crate::jupiter_program_id;
use crate::state::Config;
use crate::{
    constants::{ACCESS_SEED, VAULT_SEED},
    Access,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapParams {
    pub data: Vec<u8>,
    pub in_amount: u64,
    pub delegate: Pubkey,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    pub input_mint: InterfaceAccount<'info, Mint>,
    pub input_mint_program: Interface<'info, TokenInterface>,
    pub output_mint: InterfaceAccount<'info, Mint>,
    pub output_mint_program: Interface<'info, TokenInterface>,

    #[account(mut)]
    pub operator: Signer<'info>,
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
    #[account(mut)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub delegate_input_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [ACCESS_SEED.as_bytes(), user.key().as_ref()],
        bump,
        constraint = access.is_granted @ ErrorCode::AccessNotGranted
    )]
    pub access: Account<'info, Access>,
    /// CHECK: This is the user's account
    pub user: UncheckedAccount<'info>,
    pub jupiter_program: Program<'info, Jupiter>,
}

pub fn process_swap(ctx: Context<Swap>, params: SwapParams) -> Result<()> {
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
        ctx.accounts
            .delegate_input_token_account
            .delegate
            .contains(&ctx.accounts.vault.key()),
        ErrorCode::DelegateNotApproved
    );
    require!(
        ctx.accounts.delegate_input_token_account.delegated_amount >= params.in_amount,
        ErrorCode::InsufficientDelegatedAmount
    );
    require_keys_eq!(
        get_associated_token_address(&params.delegate, &ctx.accounts.input_mint.key()),
        ctx.accounts.delegate_input_token_account.key(),
        ErrorCode::InvalidDelegateTokenAccount
    );

    let receiver_output_token_account =
        get_associated_token_address(&ctx.accounts.user.key(), &ctx.accounts.output_mint.key());
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

    let signed_seeds = &[VAULT_SEED.as_bytes(), &[ctx.bumps.vault]];
    // 1. Transfer from delegate to vault, using the vault's delegated authority
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.input_mint_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.delegate_input_token_account.to_account_info(),
                to: ctx.accounts.vault_input_token_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.input_mint.to_account_info(),
            },
            &[signed_seeds],
        ),
        params.in_amount,
        ctx.accounts.input_mint.decimals,
    )?;

    // 2. CPI to Jupiter
    require_keys_eq!(*ctx.accounts.jupiter_program.key, jupiter_program_id());

    // Security check: Ensure the intended receiver's token account is present in the remaining_accounts
    // that will be passed to Jupiter. This prevents a malicious client from omitting or changing
    // the destination account.
    let is_receiver_ata_found = ctx
        .remaining_accounts
        .iter()
        .any(|acc| acc.key == &receiver_output_token_account);
    require!(
        is_receiver_ata_found,
        ErrorCode::ReceiverTokenAccountNotFound
    );

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
            data: params.data,
        },
        &accounts_infos,
        &[signed_seeds],
    )?;

    Ok(())
}
