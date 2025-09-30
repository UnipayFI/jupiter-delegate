use anchor_lang::prelude::*;
use anchor_spl::token_interface::{transfer_checked, TransferChecked};

use crate::{constants::VAULT_SEED, error::ErrorCode, state::Config};

pub fn check_and_transfer<'info>(
    operator: &AccountInfo<'info>,
    config: &mut Account<'info, Config>,
    vault: &AccountInfo<'info>,
    vault_bump: u8,
    delegate_input_token_account: &AccountInfo<'info>,
    input_mint: &AccountInfo<'info>,
    input_mint_program: &AccountInfo<'info>,
    vault_input_token_account: &AccountInfo<'info>,
    in_amount: u64,
    decimal: u8,
) -> Result<()> {
    // 1. 基本检查
    require!(
        operator.key() == config.operator || operator.key() == config.admin,
        ErrorCode::InvalidOperator
    );
    require!(config.is_initialized, ErrorCode::ConfigNotInitialized);
    require!(!config.is_paused, ErrorCode::ConfigPaused);

    // 2. 检查冷却时间
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

    // 3. 从 delegate 转账到 vault
    let signed_seeds = &[VAULT_SEED.as_bytes(), &[vault_bump]];
    transfer_checked(
        CpiContext::new_with_signer(
            input_mint_program.to_account_info(),
            TransferChecked {
                from: delegate_input_token_account.to_account_info(),
                to: vault_input_token_account.to_account_info(),
                authority: vault.to_account_info(),
                mint: input_mint.to_account_info(),
            },
            &[signed_seeds],
        ),
        in_amount,
        decimal,
    )?;

    Ok(())
}

pub fn prepare_cpi_accounts<'info>(
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

    let accounts_infos: Vec<AccountInfo> = remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    (accounts, accounts_infos)
}

pub fn check_receiver_token_account(
    remaining_accounts: &[AccountInfo],
    receiver_token_account: &Pubkey,
) -> Result<()> {
    let is_receiver_ata_found = remaining_accounts
        .iter()
        .any(|acc| acc.key == receiver_token_account);
    require!(
        is_receiver_ata_found,
        ErrorCode::ReceiverTokenAccountNotFound
    );
    Ok(())
}
