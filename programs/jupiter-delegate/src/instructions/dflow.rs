use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use super::aggregator::{execute_cross_program_invocation, validate_and_transfer_input};
use super::declare::dflow_aggregator::program::SwapOrchestrator;
use crate::{
    constants::{ACCESS_SEED, VAULT_SEED},
    dflow_program_id,
    error::ErrorCode,
    state::Config,
    Access, DflowAggregatorEvent,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DflowAggregatorParams {
    pub data: Vec<u8>,
    pub in_amount: u64,
    pub instruction_name: String,
    pub delegate: Pubkey,
}

#[derive(Accounts)]
pub struct DflowAggregator<'info> {
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

    #[account(mut)]
    pub delegate_input_token_account: InterfaceAccount<'info, TokenAccount>,

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

    pub dflow_program: Program<'info, SwapOrchestrator>,
}

pub fn process_dflow_aggregator<'info>(
    ctx: Context<'_, '_, '_, 'info, DflowAggregator<'info>>,
    args: DflowAggregatorParams,
) -> Result<()> {
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
        args.in_amount,
        ctx.accounts.input_mint.decimals,
        &args.delegate,
    )?;

    // 2. CPI
    execute_cross_program_invocation(
        ctx.accounts.dflow_program.key,
        &dflow_program_id(),
        ctx.remaining_accounts,
        &ctx.accounts.vault.key(),
        ctx.bumps.vault,
        args.data,
        Some(&mut ctx.accounts.vault_output_token_account),
        Some(&ctx.accounts.receiver_output_token_account),
        Some(&ctx.accounts.output_mint),
        Some(&ctx.accounts.output_mint_program),
        Some(&ctx.accounts.vault),
    )?;

    // 3. emit event
    emit!(DflowAggregatorEvent {
        user: ctx.accounts.user.key(),
        input_mint: ctx.accounts.input_mint.key(),
        output_mint: ctx.accounts.output_mint.key(),
        input_amount: args.in_amount,
        instruction_name: args.instruction_name,
        operator: ctx.accounts.operator.key(),
    });

    Ok(())
}
