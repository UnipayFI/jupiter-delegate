use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::{
    error::ErrorCode,
    state::Access,
    constants::ACCESS_SEED,
};

#[derive(Accounts)]
pub struct TokenReceive<'info> {
    pub output_mint: InterfaceAccount<'info, Mint>,
    pub output_mint_program: Interface<'info, TokenInterface>,

    #[account(mut)]
    pub executor: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = executor,
        associated_token::token_program = output_mint_program,
    )]
    pub executor_output_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = receiver,
        associated_token::token_program = output_mint_program,
    )]
    pub receiver_output_token_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: This is the fund vault account
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,
    #[account(
        seeds = [ACCESS_SEED.as_bytes(), receiver.key().as_ref()],
        bump,
        constraint = access.is_granted @ ErrorCode::AccessNotGranted
    )]
    pub access: Account<'info, Access>,
}

pub fn process_token_receive(ctx: Context<TokenReceive>) -> Result<()> {
    // 1. 验证接收者代币账户存在
    require!(
        ctx.accounts.executor_output_token_account.amount > 0,
        ErrorCode::ExecutorOutputTokenAccountIsInsufficient
    );
    let receiver_output_token_account = get_associated_token_address_with_program_id(
        &ctx.accounts.receiver.key(),
        &ctx.accounts.output_mint.key(),
        &ctx.accounts.output_mint_program.key(),
    );
    require!(
        receiver_output_token_account == ctx.accounts.receiver_output_token_account.key(),
        ErrorCode::FundVaultOutputTokenAccountNotFound
    );

    // 2. 转移代币
    transfer_checked(
        CpiContext::new(
            ctx.accounts.output_mint_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.executor_output_token_account.to_account_info(),
                to: ctx.accounts.receiver_output_token_account.to_account_info(),
                authority: ctx.accounts.executor.to_account_info(),
                mint: ctx.accounts.output_mint.to_account_info(),
            },
        ),
        ctx.accounts.executor_output_token_account.amount,
        ctx.accounts.output_mint.decimals,
    )?;

    Ok(())
}
