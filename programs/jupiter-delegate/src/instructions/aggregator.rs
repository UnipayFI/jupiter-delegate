use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};
use anchor_spl::{
    associated_token::get_associated_token_address,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{constants::VAULT_SEED, error::ErrorCode, state::Config};

pub fn validate_and_transfer_input<'info>(
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
    let delegate_account_data = delegate_input_token_account.try_borrow_data()?;
    let delegate_token_account = TokenAccount::try_deserialize(&mut &delegate_account_data[..])?;
    require!(
        delegate_token_account.delegate.contains(&vault.key()),
        ErrorCode::DelegateNotApproved
    );
    require!(
        delegate_token_account.delegated_amount >= in_amount,
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

    let accounts_infos: Vec<AccountInfo> = remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    (accounts, accounts_infos)
}

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

pub fn transfer_output_tokens<'info>(
    vault_output_token_account: &InterfaceAccount<'info, TokenAccount>,
    receiver_output_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
    output_mint: &InterfaceAccount<'info, Mint>,
    output_mint_program: &Interface<'info, TokenInterface>,
    vault: &SystemAccount<'info>,
    vault_bump: u8,
    initial_output_balance: u64,
) -> Result<()> {
    if let Some(receiver_token_account) = receiver_output_token_account {
        let output_token_balance_delta = vault_output_token_account.amount - initial_output_balance;

        if output_token_balance_delta > 0 {
            let signed_seeds = &[VAULT_SEED.as_bytes(), &[vault_bump]];
            transfer_checked(
                CpiContext::new_with_signer(
                    output_mint_program.to_account_info(),
                    TransferChecked {
                        from: vault_output_token_account.to_account_info(),
                        to: receiver_token_account.to_account_info(),
                        authority: vault.to_account_info(),
                        mint: output_mint.to_account_info(),
                    },
                    &[signed_seeds],
                ),
                output_token_balance_delta,
                output_mint.decimals,
            )?;
        }
    }
    Ok(())
}

pub fn execute_cross_program_invocation<'info>(
    target_program_id: &Pubkey,
    expected_program_id: &Pubkey,
    remaining_accounts: &[AccountInfo<'info>],
    vault_key: &Pubkey,
    vault_bump: u8,
    instruction_data: Vec<u8>,
    vault_output_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
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
    let initial_output_balance = if let Some(vault_output_account) = vault_output_token_account {
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
