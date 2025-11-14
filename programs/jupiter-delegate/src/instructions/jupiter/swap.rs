use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::{ACCESS_SEED, VAULT_SEED},
    error::ErrorCode,
    execute_cross_program_invocation,
    jupiter_aggregator::program::Jupiter,
    jupiter_program_id,
    state::Config,
    validate_and_transfer_input, validate_receiver_token_account, Access, JupiterSwapEvent,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct JupiterSwapParams {
    pub data: Vec<u8>,
    pub in_amount: u64,
    pub delegate: Pubkey,
}

#[derive(Accounts)]
pub struct JupiterSwap<'info> {
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

pub fn process_jupiter_swap(ctx: Context<JupiterSwap>, params: JupiterSwapParams) -> Result<()> {
    // 1. 验证并转移输入代币
    validate_and_transfer_input(
        &ctx.accounts.operator.to_account_info(),
        &mut ctx.accounts.config,
        &ctx.accounts.vault.to_account_info(),
        ctx.bumps.vault,
        &ctx.accounts.delegate_input_token_account,
        &ctx.accounts.input_mint.to_account_info(),
        &ctx.accounts.input_mint_program.to_account_info(),
        &ctx.accounts.vault_input_token_account.to_account_info(),
        params.in_amount,
        ctx.accounts.input_mint.decimals,
        &params.delegate,
    )?;

    // 2. 验证接收者代币账户存在
    let receiver_output_token_account = get_associated_token_address_with_program_id(
        &ctx.accounts.user.key(),
        &ctx.accounts.output_mint.key(),
        &ctx.accounts.output_mint_program.key(),
    );
    validate_receiver_token_account(ctx.remaining_accounts, &receiver_output_token_account)?;

    // 3. CPI
    execute_cross_program_invocation(
        ctx.accounts.jupiter_program.key,
        &jupiter_program_id(),
        ctx.remaining_accounts,
        &ctx.accounts.vault.key(),
        ctx.bumps.vault,
        params.data,
        None,
        None,
        None,
        None,
        None,
    )?;

    // 4. emit event
    emit!(JupiterSwapEvent {
        user: ctx.accounts.user.key(),
        input_mint: ctx.accounts.input_mint.key(),
        output_mint: ctx.accounts.output_mint.key(),
        input_amount: params.in_amount,
        operator: ctx.accounts.operator.key(),
    });

    Ok(())
}
