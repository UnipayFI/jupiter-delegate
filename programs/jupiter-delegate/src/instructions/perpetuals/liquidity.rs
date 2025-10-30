use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use super::{
    AddLiquidity2Params, JupiterPerpetualsAction, RemoveLiquidity2Params,
    DISCRIMINATOR_ADD_LIQUIDITY, DISCRIMINATOR_REMOVE_LIQUIDITY,
};
use crate::{
    error::ErrorCode, execute_cross_program_invocation, jupiter_perpetuals,
    jupiter_perpetuals::program::Perpetuals, jupiter_perpetuals_program_id,
    validate_and_transfer_input, Access, Config, JupiterPerpetualsEvent, ACCESS_SEED, VAULT_SEED,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct JupiterLiquidityParams {
    pub delegate: Pubkey,
    pub data: Vec<u8>,
}

impl JupiterLiquidityParams {
    fn get_action(&self) -> JupiterPerpetualsAction {
        let (discriminator, rest) = self.data.split_at(8);
        let discriminator = arrayref::array_ref![discriminator, 0, 8];
        if discriminator.eq(DISCRIMINATOR_ADD_LIQUIDITY) {
            let p = jupiter_perpetuals::types::AddLiquidity2Params::try_from_slice(&rest).unwrap();
            return JupiterPerpetualsAction::AddLiquidity(AddLiquidity2Params::new(
                p.token_amount_in,
                p.min_lp_amount_out,
                p.token_amount_pre_swap,
            ));
        } else if discriminator.eq(DISCRIMINATOR_REMOVE_LIQUIDITY) {
            let p =
                jupiter_perpetuals::types::RemoveLiquidity2Params::try_from_slice(&rest).unwrap();
            return JupiterPerpetualsAction::RemoveLiquidity(RemoveLiquidity2Params::new(
                p.lp_amount_in,
                p.min_amount_out,
            ));
        }
        panic!("invalidata data")
    }
}

#[derive(Accounts)]
pub struct JupiterPerpetuals<'info> {
    pub input_mint: Box<InterfaceAccount<'info, Mint>>,
    pub input_mint_program: Interface<'info, TokenInterface>,
    pub output_mint: Box<InterfaceAccount<'info, Mint>>,
    pub output_mint_program: Interface<'info, TokenInterface>,

    #[account(mut)]
    pub operator: Signer<'info>,

    #[account(mut, seeds=[VAULT_SEED.as_bytes()], bump)]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub delegate_input_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = input_mint,
        associated_token::authority = vault,
        associated_token::token_program = input_mint_program
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
        constraint = access.is_granted @ ErrorCode::AccessNotGranted,
    )]
    pub access: Account<'info, Access>,

    /// CHECK: this is the user's account
    pub user: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = user,
        associated_token::token_program = output_mint_program,
    )]
    pub receiver_output_token_account: InterfaceAccount<'info, TokenAccount>,

    pub perpetuals_program: Program<'info, Perpetuals>,
}

pub fn process_jupiter_perpetuals<'a>(
    ctx: Context<'_, '_, '_, 'a, JupiterPerpetuals<'a>>,
    args: JupiterLiquidityParams,
) -> Result<()> {
    let action = args.get_action();
    validate_and_transfer_input(
        &ctx.accounts.operator.to_account_info(),
        &mut ctx.accounts.config,
        &ctx.accounts.vault.to_account_info(),
        ctx.bumps.vault,
        &ctx.accounts.delegate_input_token_account,
        &ctx.accounts.input_mint.to_account_info(),
        &ctx.accounts.input_mint_program.to_account_info(),
        &ctx.accounts.vault_input_token_account.to_account_info(),
        action.get_input_amount(),
        ctx.accounts.input_mint.decimals,
        &args.delegate,
    )?;

    execute_cross_program_invocation(
        ctx.accounts.perpetuals_program.key,
        &jupiter_perpetuals_program_id(),
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

    emit!(JupiterPerpetualsEvent {
        user: ctx.accounts.user.key(),
        input_mint: ctx.accounts.input_mint.key(),
        output_mint: ctx.accounts.output_mint.key(),
        input_amount: action.get_input_amount(),
        action: action.to_string(),
        operator: ctx.accounts.operator.key(),
    });

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_jupiter_perpetuals_params() {
        let args = JupiterLiquidityParams {
            delegate: Pubkey::new_unique(),
            data: hex::decode("e4a24e1c46db7473809698000000000087991b0000000000010000000000000000")
                .unwrap(),
        };
        let action = args.get_action();
        assert_eq!(action.get_input_amount(), 10000000);
        assert_eq!(
            action,
            JupiterPerpetualsAction::AddLiquidity(AddLiquidity2Params::new(
                10000000,
                1808775,
                Some(0),
            ))
        );

        let args = JupiterLiquidityParams {
            delegate: Pubkey::new_unique(),
            data: hex::decode("e6d7527ff165e392ba061200000000000000000000000000").unwrap(),
        };
        let action = args.get_action();
        assert_eq!(action.get_input_amount(), 1181370);
        assert_eq!(
            action,
            JupiterPerpetualsAction::RemoveLiquidity(RemoveLiquidity2Params::new(1181370, 0,))
        );
    }
}
