use super::declare::jupiter_perpetuals::program::Perpetuals;
use crate::{
    declare::jupiter_perpetuals, error::ErrorCode, execute_cross_program_invocation,
    validate_and_transfer_input, Access, Config, JupiterPerpetualsEvent, ACCESS_SEED,
    PERPETUALS_PROGRAM_ID, VAULT_SEED,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use borsh::BorshDeserialize;

const DISCRIMINATOR_ADD_LIQUIDITY: &[u8; 8] = &[228, 162, 78, 28, 70, 219, 116, 115];
const DISCRIMINATOR_REMOVE_LIQUIDITY: &[u8; 8] = &[230, 215, 82, 127, 241, 101, 227, 146];

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct JupiterPerpetualsParams {
    pub delegate: Pubkey,
    pub data: Vec<u8>,
}

impl JupiterPerpetualsParams {
    fn get_action(&self) -> JupiterPerpetualsAction {
        let (discriminator, rest) = self.data.split_at(8);
        let discriminator = arrayref::array_ref![discriminator, 0, 8];
        if discriminator.eq(DISCRIMINATOR_ADD_LIQUIDITY) {
            let p = jupiter_perpetuals::types::AddLiquidity2Params::try_from_slice(&rest).unwrap();
            return JupiterPerpetualsAction::AddLiquidity(AddLiquidity2Params {
                token_amount_in: p.token_amount_in,
                min_lp_amount_out: p.min_lp_amount_out,
                token_amount_pre_swap: p.token_amount_pre_swap,
            });
        } else if discriminator.eq(DISCRIMINATOR_REMOVE_LIQUIDITY) {
            let p =
                jupiter_perpetuals::types::RemoveLiquidity2Params::try_from_slice(&rest).unwrap();
            return JupiterPerpetualsAction::RemoveLiquidity(RemoveLiquidity2Params {
                lp_amount_in: p.lp_amount_in,
                min_amount_out: p.min_amount_out,
            });
        }
        panic!("invalidata data")
    }
}

#[derive(Debug, PartialEq)]
enum JupiterPerpetualsAction {
    AddLiquidity(AddLiquidity2Params),
    RemoveLiquidity(RemoveLiquidity2Params),
}

#[derive(Debug, PartialEq)]
struct AddLiquidity2Params {
    token_amount_in: u64,
    min_lp_amount_out: u64,
    token_amount_pre_swap: Option<u64>,
}

#[derive(Debug, PartialEq)]
struct RemoveLiquidity2Params {
    lp_amount_in: u64,
    min_amount_out: u64,
}

impl JupiterPerpetualsAction {
    fn get_input_amount(&self) -> u64 {
        match self {
            JupiterPerpetualsAction::AddLiquidity(p) => p.token_amount_in,
            JupiterPerpetualsAction::RemoveLiquidity(p) => p.lp_amount_in,
        }
    }
}

impl ToString for JupiterPerpetualsAction {
    fn to_string(&self) -> String {
        match self {
            JupiterPerpetualsAction::AddLiquidity(_p) => "add_liquidity2".to_string(),
            JupiterPerpetualsAction::RemoveLiquidity(_p) => "remove_liquidity2".to_string(),
        }
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
    args: JupiterPerpetualsParams,
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
        &PERPETUALS_PROGRAM_ID,
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
        let args = JupiterPerpetualsParams {
            delegate: Pubkey::new_unique(),
            data: hex::decode("e4a24e1c46db7473809698000000000087991b0000000000010000000000000000")
                .unwrap(),
        };
        let action = args.get_action();
        assert_eq!(action.get_input_amount(), 10000000);
        assert_eq!(
            action,
            JupiterPerpetualsAction::AddLiquidity(AddLiquidity2Params {
                token_amount_in: 10000000,
                min_lp_amount_out: 1808775,
                token_amount_pre_swap: Some(0),
            })
        );

        let args = JupiterPerpetualsParams {
            delegate: Pubkey::new_unique(),
            data: hex::decode("e6d7527ff165e392ba061200000000000000000000000000").unwrap(),
        };
        let action = args.get_action();
        assert_eq!(action.get_input_amount(), 1181370);
        assert_eq!(
            action,
            JupiterPerpetualsAction::RemoveLiquidity(RemoveLiquidity2Params {
                lp_amount_in: 1181370,
                min_amount_out: 0,
            })
        );
    }
}
