use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token_interface::{transfer_checked, TokenAccount, TransferChecked},
};

use crate::{constants::VAULT_SEED, error::ErrorCode, state::Config};

pub fn validate_receiver_token_account(
    remaining_accounts: &[AccountInfo],
    receiver_token_account: &Pubkey,
) -> Result<()> {
    let receiver_account_exists = remaining_accounts
        .iter()
        .any(|acc| acc.key == receiver_token_account);
    require!(
        receiver_account_exists,
        ErrorCode::ReceiverTokenAccountNotFound
    );
    Ok(())
}

pub fn validate_and_transfer_input<'info>(
    operator: &AccountInfo<'info>,
    config: &mut Account<'info, Config>,
    vault: &AccountInfo<'info>,
    vault_bump: u8,
    delegate_input_token_account: &InterfaceAccount<'info, TokenAccount>,
    input_mint: &AccountInfo<'info>,
    input_mint_program: &AccountInfo<'info>,
    vault_input_token_account: &AccountInfo<'info>,
    in_amount: u64,
    decimal: u8,
    delegate_pubkey: &Pubkey,
) -> Result<()> {
    // 1. 基本检查
    require!(
        operator.key() == config.operator || operator.key() == config.admin,
        ErrorCode::InvalidOperator
    );
    require!(config.is_initialized, ErrorCode::ConfigNotInitialized);
    require!(!config.is_paused, ErrorCode::ConfigPaused);

    // 2. 验证委托账户
    require!(
        delegate_input_token_account.delegate.contains(&vault.key()),
        ErrorCode::DelegateNotApproved
    );
    require!(
        delegate_input_token_account.delegated_amount >= in_amount,
        ErrorCode::InsufficientDelegatedAmount
    );
    require_keys_eq!(
        get_associated_token_address(delegate_pubkey, input_mint.key),
        delegate_input_token_account.key(),
        ErrorCode::InvalidDelegateTokenAccount
    );

    // 3. 检查冷却时间
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

    // 4. 从 delegate 转账到 vault
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
