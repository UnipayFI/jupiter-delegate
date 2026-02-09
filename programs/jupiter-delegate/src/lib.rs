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

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

declare_id!("JPDGXJky3iRkPmJx3cixg5cxJGGwP9kXBJzMpT5GLir");

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "jupiter-delegate",
    project_url: "https://github.com/UnipayFI/jupiter-delegate",
    contacts: "link:https://github.com/UnipayFI/jupiter-delegate/issues",
    policy: "https://github.com/UnipayFI/jupiter-delegate/issues",
    preferred_languages: "en,zh",
    source_code: "https://github.com/UnipayFI/jupiter-delegate"
}

pub fn jupiter_program_id() -> Pubkey {
    pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")
}

pub fn jupiter_order_engine_program_id() -> Pubkey {
    pubkey!("61DFfeTKM7trxYcPQCM78bJ794ddZprZpAwAnLiwTpYH")
}

pub fn dflow_program_id() -> Pubkey {
    pubkey!("DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH")
}

pub fn okx_program_id() -> Pubkey {
    pubkey!("6m2CDdhRgxpH4WjvdzxAYbGxwdGUz5MziiL5jek2kBma")
}

pub fn jupiter_perpetuals_program_id() -> Pubkey {
    pubkey!("PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu")
}

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

    pub fn swap(_ctx: Context<JupiterSwap>, _params: JupiterSwapParams) -> Result<()> {
        // process_jupiter_swap(ctx, params)
        Ok(())
    }

    pub fn fill_order_engine<'a>(
        _ctx: Context<'_, '_, '_, 'a, FillOrderEngine<'a>>,
        _params: FillOrderEngineParams,
    ) -> Result<()> {
        // process_fill_order_engine(ctx, params)
        Ok(())
    }

    pub fn jupiter_aggregator<'a>(
        _ctx: Context<'_, '_, '_, 'a, JupiterAggregator<'a>>,
        _params: JupiterAggregatorParams,
    ) -> Result<()> {
        // process_jupiter_aggregator(ctx, params)
        Ok(())
    }

    pub fn jupiter_perpetuals<'a>(
        _ctx: Context<'_, '_, '_, 'a, JupiterPerpetuals<'a>>,
        _params: JupiterLiquidityParams,
    ) -> Result<()> {
        // process_jupiter_perpetuals(ctx, params)
        Ok(())
    }

    pub fn dflow_aggregator<'a>(
        _ctx: Context<'_, '_, '_, 'a, DflowAggregator<'a>>,
        _params: DflowAggregatorParams,
    ) -> Result<()> {
        //process_dflow_aggregator(ctx, params)
        Ok(())
    }

    pub fn okx_aggregator<'a>(
        ctx: Context<'_, '_, '_, 'a, OkxAggregator<'a>>,
        params: OkxAggregatorParams,
    ) -> Result<()> {
        process_okx_aggregator(ctx, params)
    }

    pub fn two_hop<'a>(
        _ctx: Context<'_, '_, '_, 'a, TwoHop<'a>>,
        _params: TwoHopParams,
    ) -> Result<()> {
        // process_two_hop(ctx, params)
        Ok(())
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

    pub fn token_receive(ctx: Context<TokenReceive>) -> Result<()> {
        process_token_receive(ctx)
    }

    pub fn transfer_in(ctx: Context<TransferIn>, amounts: u64) -> Result<()> {
        prorcess_transfer_in(ctx, amounts)
    }

    pub fn transfer_out(ctx: Context<TransferOut>, amounts: u64) -> Result<()> {
        prorcess_transfer_out(ctx, amounts)
    }
}
