use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::{
    associated_token::get_associated_token_address,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use jupiter_order_engine::program::OrderEngine;

declare_program!(jupiter_order_engine);

use crate::constants::{ACCESS_SEED, VAULT_SEED};
use crate::error::ErrorCode;
use crate::jupiter_order_engine_program_id;
use crate::state::Config;
use crate::Access;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct OrderEngineParams {
    pub in_amount: u64,
    pub out_amount: u64,
    pub data: Vec<u8>,
    pub delegate: Pubkey,
}

#[derive(Accounts)]
pub struct FillOrderEngine<'info> {
    pub input_mint: InterfaceAccount<'info, Mint>,
    pub input_mint_program: Interface<'info, TokenInterface>,
    pub output_mint: InterfaceAccount<'info, Mint>,
    pub output_mint_program: Interface<'info, TokenInterface>,

    #[account(mut)]
    pub operator: Signer<'info>,

    // Vault作为taker
    #[account(
        mut,
        seeds=[VAULT_SEED.as_bytes()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    // 用户授权给vault的代币账户
    #[account(mut)]
    pub delegate_input_token_account: InterfaceAccount<'info, TokenAccount>,

    // vault的输入代币账户，对应taker_input_mint_token_account
    #[account(
        mut,
        associated_token::mint = input_mint,
        associated_token::authority = vault,
        associated_token::token_program = input_mint_program,
    )]
    pub vault_input_token_account: InterfaceAccount<'info, TokenAccount>,

    // vault的输出代币账户，对应taker_output_mint_token_account
    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = vault,
        associated_token::token_program = output_mint_program,
    )]
    pub vault_output_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub config: Box<Account<'info, Config>>,

    #[account(
        seeds = [ACCESS_SEED.as_bytes(), user.key().as_ref()],
        bump,
        constraint = access.is_granted @ ErrorCode::AccessNotGranted
    )]
    pub access: Account<'info, Access>,

    /// CHECK: This is the user's account
    pub user: UncheckedAccount<'info>,

    /// CHECK: Receiver output token account
    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = user,
        associated_token::token_program = output_mint_program,
    )]
    pub receiver_output_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Validated in CPI
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    /// CHECK: Validated in CPI
    #[account(mut)]
    pub maker_input_token_account: UncheckedAccount<'info>,

    /// CHECK: Validated in CPI
    #[account(mut)]
    pub maker_output_token_account: UncheckedAccount<'info>,

    pub order_engine_program: Program<'info, OrderEngine>,

    /// CHECK: Validated in CPI
    pub system_program: UncheckedAccount<'info>,
}

pub fn process_fill_order_engine(
    ctx: Context<FillOrderEngine>,
    params: OrderEngineParams,
) -> Result<()> {
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
    // 2. CPI to Order Engine
    require_keys_eq!(
        *ctx.accounts.order_engine_program.key,
        jupiter_order_engine_program_id()
    );

    let ix = Instruction {
        program_id: ctx.accounts.order_engine_program.key(),
        accounts: vec![
            // taker (vault) and maker
            AccountMeta::new(ctx.accounts.vault.key(), true),
            AccountMeta::new(ctx.accounts.maker.key(), true),
            // token accounts
            AccountMeta::new(ctx.accounts.vault_input_token_account.key(), false),
            AccountMeta::new(ctx.accounts.maker_input_token_account.key(), false),
            AccountMeta::new(ctx.accounts.vault_output_token_account.key(), false),
            AccountMeta::new(ctx.accounts.maker_output_token_account.key(), false),
            // mints and programs
            AccountMeta::new_readonly(ctx.accounts.input_mint.key(), false),
            AccountMeta::new_readonly(ctx.accounts.input_mint_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.output_mint.key(), false),
            AccountMeta::new_readonly(ctx.accounts.output_mint_program.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        ],
        data: params.data.clone(),
    };

    let vault_output_token_balance_before = ctx.accounts.vault_output_token_account.amount;
    invoke_signed(
        &ix,
        &[
            // taker (vault) and maker
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.maker.to_account_info(),
            // token accounts
            ctx.accounts.vault_input_token_account.to_account_info(),
            ctx.accounts.maker_input_token_account.to_account_info(),
            ctx.accounts.vault_output_token_account.to_account_info(),
            ctx.accounts.maker_output_token_account.to_account_info(),
            // mints and programs
            ctx.accounts.input_mint.to_account_info(),
            ctx.accounts.input_mint_program.to_account_info(),
            ctx.accounts.output_mint.to_account_info(),
            ctx.accounts.output_mint_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[signed_seeds],
    )?;
    let vault_output_token_balance_after = ctx.accounts.vault_output_token_account.amount;
    require!(
        vault_output_token_balance_after > vault_output_token_balance_before
            && vault_output_token_balance_after - vault_output_token_balance_before
                >= params.out_amount,
        ErrorCode::OrderEngineFailed
    );
    let output_amount = vault_output_token_balance_after - vault_output_token_balance_before;
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.output_mint_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_output_token_account.to_account_info(),
                to: ctx.accounts.receiver_output_token_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.output_mint.to_account_info(),
            },
            &[signed_seeds],
        ),
        output_amount,
        ctx.accounts.output_mint.decimals,
    )?;

    Ok(())
}
