use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use super::aggregator::{
    execute_cross_program_invocation, validate_and_transfer_input, validate_receiver_token_account,
};
use super::declare::jupiter_aggregator::program::Jupiter;
use crate::{
    constants::{ACCESS_SEED, VAULT_SEED},
    error::ErrorCode,
    jupiter_program_id,
    state::Config,
    Access, SwapEvent,
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
    // 1. 验证并转移输入代币
    validate_and_transfer_input(
        &ctx.accounts.operator.to_account_info(),
        &mut ctx.accounts.config,
        &ctx.accounts.vault.to_account_info(),
        ctx.bumps.vault,
        &ctx.accounts.delegate_input_token_account.to_account_info(),
        &ctx.accounts.input_mint.to_account_info(),
        &ctx.accounts.input_mint_program.to_account_info(),
        &ctx.accounts.vault_input_token_account.to_account_info(),
        params.in_amount,
        ctx.accounts.input_mint.decimals,
    )?;

    // 2. 验证接收者代币账户存在
    let receiver_output_token_account =
        get_associated_token_address(&ctx.accounts.user.key(), &ctx.accounts.output_mint.key());
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
    emit!(SwapEvent {
        user: ctx.accounts.user.key(),
        input_mint: ctx.accounts.input_mint.key(),
        output_mint: ctx.accounts.output_mint.key(),
        input_amount: params.in_amount,
        operator: ctx.accounts.operator.key(),
    });

    Ok(())
}
