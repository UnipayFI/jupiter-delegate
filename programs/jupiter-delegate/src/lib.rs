pub mod constants;
pub mod error;
pub mod event;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use event::*;
pub use instructions::*;
pub use state::*;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::pubkey;
use std::str::FromStr;

declare_id!("JPDGXJky3iRkPmJx3cixg5cxJGGwP9kXBJzMpT5GLir");

pub fn jupiter_program_id() -> Pubkey {
    Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap()
}

pub fn jupiter_order_engine_program_id() -> Pubkey {
    Pubkey::from_str("61DFfeTKM7trxYcPQCM78bJ794ddZprZpAwAnLiwTpYH").unwrap()
}

pub fn dflow_program_id() -> Pubkey {
    Pubkey::from_str("DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH").unwrap()
}

pub fn okx_program_id() -> Pubkey {
    Pubkey::from_str("6m2CDdhRgxpH4WjvdzxAYbGxwdGUz5MziiL5jek2kBma").unwrap()
}

pub const PERPETUALS_PROGRAM_ID: Pubkey = pubkey!("PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu");

#[program]
pub mod jupiter_delegate {
    use super::*;

    pub fn init_config(
        ctx: Context<InitConfig>,
        operator: Pubkey,
        cooldown_duration: i64,
    ) -> Result<()> {
        process_init_config(ctx, operator, cooldown_duration)
    }

    pub fn grant_access(ctx: Context<GrantAccess>, user: Pubkey) -> Result<()> {
        process_grant_access(ctx, user)
    }

    pub fn revoke_access(ctx: Context<RevokeAccess>, user: Pubkey) -> Result<()> {
        process_revoke_access(ctx, user)
    }

    pub fn swap(ctx: Context<Swap>, params: SwapParams) -> Result<()> {
        process_swap(ctx, params)
    }

    pub fn fill_order_engine<'a>(
        ctx: Context<'_, '_, '_, 'a, FillOrderEngine<'a>>,
        params: FillOrderEngineParams,
    ) -> Result<()> {
        process_fill_order_engine(ctx, params)
    }

    pub fn jupiter_aggregator<'a>(
        ctx: Context<'_, '_, '_, 'a, JupiterAggregator<'a>>,
        params: JupiterAggregatorParams,
    ) -> Result<()> {
        process_jupiter_aggregator(ctx, params)
    }

    pub fn jupiter_perpetuals<'a>(
        ctx: Context<'_, '_, '_, 'a, JupiterPerpetuals<'a>>,
        params: JupiterPerpetualsParams,
    ) -> Result<()> {
        process_jupiter_perpetuals(ctx, params)
    }

    pub fn dflow_aggregator<'a>(
        ctx: Context<'_, '_, '_, 'a, DflowAggregator<'a>>,
        params: DflowAggregatorParams,
    ) -> Result<()> {
        process_dflow_aggregator(ctx, params)
    }

    pub fn okx_aggregator<'a>(
        ctx: Context<'_, '_, '_, 'a, OkxAggregator<'a>>,
        params: OkxAggregatorParams,
    ) -> Result<()> {
        process_okx_aggregator(ctx, params)
    }

    pub fn propose_new_admin(ctx: Context<ProposeNewAdmin>) -> Result<()> {
        process_propose_new_admin(ctx)
    }

    pub fn accept_admin_transfer(ctx: Context<AcceptAdminTransfer>) -> Result<()> {
        process_accept_admin_transfer(ctx)
    }

    pub fn modify_cooldown_duration(
        ctx: Context<ModifyCooldownDuration>,
        cooldown_duration: i64,
    ) -> Result<()> {
        process_modify_cooldown_duration(ctx, cooldown_duration)
    }

    pub fn pause(ctx: Context<Pause>, toggle: bool) -> Result<()> {
        process_pause(ctx, toggle)
    }
}
