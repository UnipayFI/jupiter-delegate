use super::declare::jupiter_order_engine::program::OrderEngine;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use super::check_and_transfer;
use crate::{
    constants::{ACCESS_SEED, VAULT_SEED},
    error::ErrorCode,
    jupiter_order_engine_program_id,
    state::Config,
    Access,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FillOrderEngineParams {
    pub data: Vec<u8>,
    pub in_amount: u64,
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

    // Jupiter Order Engine 特定账户
    /// CHECK: Taker account (will be vault in our case)
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,
    /// CHECK: Taker input mint token account (vault's token account)
    #[account(mut)]
    pub taker_input_mint_token_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Maker input mint token account
    #[account(mut)]
    pub maker_input_mint_token_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Taker output mint token account
    pub taker_output_mint_token_account: UncheckedAccount<'info>,
    /// CHECK: Maker output mint token account
    #[account(mut)]
    pub maker_output_mint_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub jupiter_order_engine_program: Program<'info, OrderEngine>,
}

pub fn process_fill_order_engine<'a>(
    ctx: Context<'_, '_, '_, 'a, FillOrderEngine<'a>>,
    params: FillOrderEngineParams,
) -> Result<()> {
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

    // 2. 检查 Jupiter Order Engine 程序 ID
    require_keys_eq!(
        *ctx.accounts.jupiter_order_engine_program.key,
        jupiter_order_engine_program_id()
    );

    // 3. 构建 Jupiter Order Engine fill 指令的账户
    // 根据解码结果，fill 指令需要以下账户顺序：
    // 0. taker (signer) - 在我们的情况下是 vault
    // 1. maker (signer) - 来自参数
    // 2. taker_input_mint_token_account - vault 的输入代币账户
    // 3. maker_input_mint_token_account - maker 的输入代币账户
    // 4. taker_output_mint_token_account - taker 的输出代币账户
    // 5. maker_output_mint_token_account - maker 的输出代币账户
    // 6. input_mint
    // 7. input_token_program
    // 8. output_mint
    // 9. output_token_program
    // 10. system_program

    let mut accounts = vec![
        AccountMeta::new(ctx.accounts.vault.key(), true), // taker (vault as signer)
        AccountMeta::new(ctx.accounts.maker.key(), true), // maker (signer)
        AccountMeta::new(ctx.accounts.taker_input_mint_token_account.key(), false), // taker input token account
        AccountMeta::new(ctx.accounts.maker_input_mint_token_account.key(), false), // maker input token account
        AccountMeta::new_readonly(ctx.accounts.taker_output_mint_token_account.key(), false), // taker output token account
        AccountMeta::new(ctx.accounts.maker_output_mint_token_account.key(), false), // maker output token account
        AccountMeta::new_readonly(ctx.accounts.input_mint.key(), false),             // input mint
        AccountMeta::new_readonly(ctx.accounts.input_mint_program.key(), false), // input token program
        AccountMeta::new_readonly(ctx.accounts.output_mint.key(), false),        // output mint
        AccountMeta::new_readonly(ctx.accounts.output_mint_program.key(), false), // output token program
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),      // system program
    ];

    // 添加 remaining accounts
    ctx.remaining_accounts.iter().for_each(|acc| {
        accounts.push(AccountMeta {
            pubkey: acc.key(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        });
    });

    let signed_seeds = &[VAULT_SEED.as_bytes(), &[ctx.bumps.vault]];
    let mut account_infos: Vec<AccountInfo<'a>> = vec![
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.maker.to_account_info(),
        ctx.accounts.taker_input_mint_token_account.to_account_info(),
        ctx.accounts.maker_input_mint_token_account.to_account_info(),
        ctx.accounts.taker_output_mint_token_account.to_account_info(),
        ctx.accounts.maker_output_mint_token_account.to_account_info(),
    ];
    account_infos.extend(ctx.remaining_accounts.iter().map(|a| -> AccountInfo<'a> {a.to_owned()}));
    
    let account_infos = account_infos.as_slice();
    invoke_signed(
        &Instruction {
            program_id: ctx.accounts.jupiter_order_engine_program.key(),
            accounts,
            data: params.data,
        },
        account_infos,
        &[signed_seeds],
    )?;

    // // 调用 Jupiter Order Engine
    // let signed_seeds = &[VAULT_SEED.as_bytes(), &[ctx.bumps.vault]];
    // invoke_signed(
    //     &Instruction {
    //         program_id: ctx.accounts.jupiter_order_engine_program.key(),
    //         accounts,
    //         data: params.data,
    //     },
    //     &all_account_infos,
    //     &[signed_seeds],
    // )?;

    Ok(())
}
