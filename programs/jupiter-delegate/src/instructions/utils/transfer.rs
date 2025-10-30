use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::constants::VAULT_SEED;

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

        msg!("Debug: initial_output_balance: {}", initial_output_balance);
        msg!(
            "Debug: vault_output_token_account.amount: {}",
            vault_output_token_account.amount
        );
        msg!(
            "Debug: output_token_balance_delta: {}",
            output_token_balance_delta
        );

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
