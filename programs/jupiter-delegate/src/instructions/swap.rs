use super::declare::jupiter_aggregator::program::Jupiter;
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::{
    associated_token::get_associated_token_address,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use super::{check_and_transfer, check_receiver_token_account, prepare_cpi_accounts};
use crate::{
    constants::{ACCESS_SEED, VAULT_SEED},
    error::ErrorCode,
    jupiter_program_id,
    state::Config,
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
    // 1. 通用检查和转账
    check_and_transfer(
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

    // 2. 检查 Jupiter 程序 ID
    require_keys_eq!(*ctx.accounts.jupiter_program.key, jupiter_program_id());

    // 3. 检查接收者的代币账户
    let receiver_output_token_account =
        get_associated_token_address(&ctx.accounts.user.key(), &ctx.accounts.output_mint.key());
    check_receiver_token_account(ctx.remaining_accounts, &receiver_output_token_account)?;

    // 4. 准备 CPI 账户
    let (accounts, accounts_infos) =
        prepare_cpi_accounts(ctx.remaining_accounts, &ctx.accounts.vault.key());

    // 5. 调用 Jupiter
    let signed_seeds = &[VAULT_SEED.as_bytes(), &[ctx.bumps.vault]];
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
