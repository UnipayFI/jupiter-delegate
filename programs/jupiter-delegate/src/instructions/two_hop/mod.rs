pub mod types;

use super::types::StepAction;
use crate::{
    error::ErrorCode, execute_cross_program_invocation, transfer_output_tokens,
    validate_and_transfer_input, Access, Config, ACCESS_SEED, VAULT_SEED,
};
use anchor_lang::{prelude::*, solana_program::account_info::next_account_infos};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use std::slice::Iter;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StepParams {
    pub action: StepAction,
    pub account_counts: u8,
    pub data: Vec<u8>,
    pub amount_in: u64,
    pub expect_amount_out: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TwoHopParams {
    pub delegate: Pubkey,
    pub step1: StepParams,
    pub step2: StepParams,
}

#[derive(Accounts)]
pub struct TwoHop<'info> {
    pub input_mint_one: Box<InterfaceAccount<'info, Mint>>,
    pub input_mint_program_one: Interface<'info, TokenInterface>,
    pub output_mint_one: Box<InterfaceAccount<'info, Mint>>,
    pub output_mint_program_one: Interface<'info, TokenInterface>,
    pub input_mint_two: Box<InterfaceAccount<'info, Mint>>,
    pub input_mint_program_two: Interface<'info, TokenInterface>,
    pub output_mint_two: Box<InterfaceAccount<'info, Mint>>,
    pub output_mint_program_two: Interface<'info, TokenInterface>,

    #[account(mut)]
    pub operator: Signer<'info>,

    #[account(
        mut,
        seeds=[VAULT_SEED.as_bytes()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub delegate_input_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = input_mint_one,
        associated_token::authority = vault,
        associated_token::token_program = input_mint_program_one,
    )]
    pub vault_input_token_account_one: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = output_mint_one,
        associated_token::authority = vault,
        associated_token::token_program = output_mint_program_one,
    )]
    pub vault_output_token_account_one: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = input_mint_two,
        associated_token::authority = vault,
        associated_token::token_program = input_mint_program_two,
    )]
    pub vault_input_token_account_two: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = output_mint_two,
        associated_token::authority = vault,
        associated_token::token_program = output_mint_program_two,
    )]
    pub vault_output_token_account_two: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub config: Box<Account<'info, Config>>,

    #[account(
        seeds = [ACCESS_SEED.as_bytes(), user.key().as_ref()],
        bump,
        constraint = access.is_granted @ ErrorCode::AccessNotGranted,
    )]
    pub access: Account<'info, Access>,

    /// CHECK: this is the user's account
    pub user: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = output_mint_one,
        associated_token::authority = user,
        associated_token::token_program = output_mint_program_one,
    )]
    pub receiver_output_token_account_one: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = output_mint_two,
        associated_token::authority = user,
        associated_token::token_program = output_mint_program_two,
    )]
    pub receiver_output_token_account_two: InterfaceAccount<'info, TokenAccount>,
}

pub fn process_two_hop<'a>(
    ctx: Context<'_, '_, '_, 'a, TwoHop<'a>>,
    args: TwoHopParams,
) -> Result<()> {
    validate_and_transfer_input(
        &ctx.accounts.operator.to_account_info(),
        &mut ctx.accounts.config,
        &ctx.accounts.vault.to_account_info(),
        ctx.bumps.vault,
        &ctx.accounts.delegate_input_token_account,
        &ctx.accounts.input_mint_one.to_account_info(),
        &ctx.accounts.input_mint_program_one.to_account_info(),
        &ctx.accounts.vault_input_token_account_one.to_account_info(),
        args.step1.amount_in,
        ctx.accounts.input_mint_one.decimals,
        &args.delegate,
    )?;

    let mut remain_accounts = ctx.remaining_accounts.iter();

    let (_, _) = process_step(
        &args.step1,
        &mut remain_accounts,
        &ctx.accounts.vault,
        ctx.bumps.vault,
        &mut ctx.accounts.vault_input_token_account_one,
        &mut ctx.accounts.vault_output_token_account_one,
    )?;

    let (_, _) = process_step(
        &args.step2,
        &mut remain_accounts,
        &ctx.accounts.vault,
        ctx.bumps.vault,
        &mut ctx.accounts.vault_input_token_account_two,
        &mut ctx.accounts.vault_output_token_account_two,
    )?;

    ctx.accounts.vault_output_token_account_two.reload()?;
    if ctx.accounts.vault_output_token_account_two.amount > 0 {
        transfer_output_tokens(
            &ctx.accounts.vault_output_token_account_two,
            Some(&ctx.accounts.receiver_output_token_account_two),
            &ctx.accounts.output_mint_two,
            &ctx.accounts.output_mint_program_two,
            &ctx.accounts.vault,
            ctx.bumps.vault,
            ctx.accounts.vault_output_token_account_two.amount,
        )?;
    }
    ctx.accounts.vault_output_token_account_one.reload()?;
    if ctx.accounts.vault_output_token_account_one.amount > 0 {
        transfer_output_tokens(
            &ctx.accounts.vault_output_token_account_one,
            Some(&ctx.accounts.receiver_output_token_account_one),
            &ctx.accounts.output_mint_one,
            &ctx.accounts.output_mint_program_one,
            &ctx.accounts.vault,
            ctx.bumps.vault,
            ctx.accounts.vault_output_token_account_two.amount,
        )?;
    }
    ctx.accounts.vault_input_token_account_one.reload()?;
    if ctx.accounts.vault_input_token_account_one.amount > 0 {
        transfer_output_tokens(
            &ctx.accounts.vault_input_token_account_one,
            Some(&ctx.accounts.delegate_input_token_account),
            &ctx.accounts.input_mint_one,
            &ctx.accounts.input_mint_program_one,
            &ctx.accounts.vault,
            ctx.bumps.vault,
            ctx.accounts.vault_input_token_account_one.amount,
        )?;
    }

    Ok(())
}

fn process_step<'info>(
    args: &StepParams,
    remain_accounts: &mut Iter<AccountInfo<'info>>,
    vault: &SystemAccount<'info>,
    bump: u8,
    vault_input_token_account: &mut Box<InterfaceAccount<'info, TokenAccount>>,
    vault_output_token_account: &mut Box<InterfaceAccount<'info, TokenAccount>>,
) -> Result<(u64, u64)> {
    let step_out_token_account_one_amount = vault_output_token_account.amount;
    let step_in_token_account_one_amount = vault_input_token_account.amount;

    let program_account = next_account_info(remain_accounts)?;
    let accounts = next_account_infos(remain_accounts, args.account_counts as usize)?;

    execute_cross_program_invocation(
        &program_account.key(),
        &program_account.key(),
        accounts,
        &vault.key(),
        bump,
        args.action.to_program_instruction_data(&args.data),
        None,
        None,
        None,
        None,
        None,
    )?;

    vault_input_token_account.reload()?;
    vault_output_token_account.reload()?;

    let step_in_token_account_one_amount_after = vault_input_token_account.amount;
    let step_out_token_account_one_amount_after = vault_output_token_account.amount;

    let amount_in_diff =
        if step_in_token_account_one_amount_after < step_in_token_account_one_amount {
            let amount_in_diff =
                step_in_token_account_one_amount - step_out_token_account_one_amount_after;
            if amount_in_diff > args.amount_in {
                return Err(ErrorCode::InvalidOperator.into());
            }
            amount_in_diff
        } else {
            0
        };
    let amount_out_diff =
        if step_out_token_account_one_amount_after < step_out_token_account_one_amount {
            return Err(ErrorCode::InvalidOperator.into());
        } else {
            let diff = step_out_token_account_one_amount_after - step_out_token_account_one_amount;
            if diff < args.expect_amount_out {
                return Err(ErrorCode::InvalidOperator.into());
            }
            diff
        };
    Ok((amount_in_diff, amount_out_diff))
}
