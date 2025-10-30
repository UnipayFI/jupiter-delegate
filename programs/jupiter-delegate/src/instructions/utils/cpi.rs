use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use std::collections::HashSet;

use super::transfer::transfer_output_tokens;
use crate::constants::VAULT_SEED;

pub fn prepare_cross_program_accounts<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    vault: &Pubkey,
) -> (Vec<AccountMeta>, Vec<AccountInfo<'info>>) {
    let accounts: Vec<AccountMeta> = remaining_accounts
        .iter()
        .map(|acc| {
            let is_signer = acc.key == vault;
            AccountMeta {
                pubkey: *acc.key,
                is_signer,
                is_writable: acc.is_writable,
            }
        })
        .collect();

    let mut seen = HashSet::new();
    let accounts_infos: Vec<AccountInfo> = remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .filter(|acc| seen.insert(*acc.key))
        .collect();

    (accounts, accounts_infos)
}

pub fn execute_cross_program_invocation<'info>(
    target_program_id: &Pubkey,
    expected_program_id: &Pubkey,
    remaining_accounts: &[AccountInfo<'info>],
    vault_key: &Pubkey,
    vault_bump: u8,
    instruction_data: Vec<u8>,
    vault_output_token_account: Option<&mut InterfaceAccount<'info, TokenAccount>>,
    receiver_output_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
    output_mint: Option<&InterfaceAccount<'info, Mint>>,
    output_mint_program: Option<&Interface<'info, TokenInterface>>,
    vault: Option<&SystemAccount<'info>>,
) -> Result<()> {
    // 1. 检查目标程序 ID
    require_keys_eq!(*target_program_id, *expected_program_id);

    // 2. 准备 CPI 账户
    let (account_metas, account_infos) =
        prepare_cross_program_accounts(remaining_accounts, vault_key);

    // 3. 记录输出代币余额
    let initial_output_balance = if let Some(ref vault_output_account) = vault_output_token_account
    {
        vault_output_account.amount
    } else {
        0
    };

    // 4. 调用目标聚合器
    let signed_seeds = &[VAULT_SEED.as_bytes(), &[vault_bump]];
    invoke_signed(
        &Instruction {
            program_id: *target_program_id,
            accounts: account_metas,
            data: instruction_data,
        },
        &account_infos,
        &[signed_seeds],
    )?;

    // 5. 转移输出代币
    if let (
        Some(vault_output_account),
        Some(receiver),
        Some(mint),
        Some(mint_program),
        Some(vault_account),
    ) = (
        vault_output_token_account,
        receiver_output_token_account,
        output_mint,
        output_mint_program,
        vault,
    ) {
        vault_output_account.reload()?;

        transfer_output_tokens(
            vault_output_account,
            Some(receiver),
            mint,
            mint_program,
            vault_account,
            vault_bump,
            initial_output_balance,
        )?;
    }

    Ok(())
}
