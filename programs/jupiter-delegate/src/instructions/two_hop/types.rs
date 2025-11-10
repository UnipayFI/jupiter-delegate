use anchor_lang::prelude::*;

use crate::{
    instructions::perpetuals, DISCRIMINATOR_DFLOW_SWAP, DISCRIMINATOR_DFLOW_SWAP2,
    DISCRIMINATOR_JUPITER_AGGREGATOR_ROUTE, DISCRIMINATOR_JUPITER_AGGREGATOR_SHARED_ACCOUNTS_ROUTE,
    DISCRIMINATOR_JUPITER_AGGREGATOR_SHARED_ACCOUNTS_ROUTE_V2,
    DISCRIMINATOR_JUPITER_ORDER_ENGINE_FILL, DISCRIMINATOR_OKX_SWAP, DISCRIMINATOR_OKX_SWAP_TOB_V3,
    DISCRIMINATOR_OKX_SWAP_TOB_V3_WITH_RECEIVER, DISCRIMINATOR_OKX_SWAP_V3,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum StepAction {
    JupiterSharedAccountsRoute,
    JupiterSharedAccountsRouteV2,
    JupiterRoute,
    JupiterRouteV2,
    JupiterOrderEngineFill,
    OkxSwapTobV3,
    OkxSwapV3,
    OkxSwapTobV3WithReceiver,
    OkxSwap,
    DFlowSwap,
    DFlowSwap2,
    JlpPerpetualsAddLiquidity2,
    JlpPerpetualsRemoveLiquidity2,
}

impl StepAction {
    pub fn to_program_instruction_data(&self, data: &[u8]) -> Vec<u8> {
        let mut instruction_data = vec![];
        let discriminator = match self {
            StepAction::JupiterSharedAccountsRoute => {
                DISCRIMINATOR_JUPITER_AGGREGATOR_SHARED_ACCOUNTS_ROUTE
            }
            StepAction::JupiterSharedAccountsRouteV2 => {
                DISCRIMINATOR_JUPITER_AGGREGATOR_SHARED_ACCOUNTS_ROUTE_V2
            }
            StepAction::JupiterRoute => DISCRIMINATOR_JUPITER_AGGREGATOR_ROUTE,
            StepAction::JupiterRouteV2 => DISCRIMINATOR_JUPITER_AGGREGATOR_SHARED_ACCOUNTS_ROUTE_V2,
            StepAction::JupiterOrderEngineFill => DISCRIMINATOR_JUPITER_ORDER_ENGINE_FILL,
            StepAction::OkxSwapTobV3 => DISCRIMINATOR_OKX_SWAP_TOB_V3,
            StepAction::OkxSwapV3 => DISCRIMINATOR_OKX_SWAP_V3,
            StepAction::OkxSwapTobV3WithReceiver => DISCRIMINATOR_OKX_SWAP_TOB_V3_WITH_RECEIVER,
            StepAction::OkxSwap => DISCRIMINATOR_OKX_SWAP,
            StepAction::DFlowSwap => DISCRIMINATOR_DFLOW_SWAP,
            StepAction::DFlowSwap2 => DISCRIMINATOR_DFLOW_SWAP2,
            StepAction::JlpPerpetualsAddLiquidity2 => {
                perpetuals::discriminator::DISCRIMINATOR_ADD_LIQUIDITY
            }
            StepAction::JlpPerpetualsRemoveLiquidity2 => {
                perpetuals::discriminator::DISCRIMINATOR_REMOVE_LIQUIDITY
            }
        };
        instruction_data.extend_from_slice(discriminator);
        instruction_data.extend_from_slice(data);
        instruction_data
    }
}

impl ToString for StepAction {
    fn to_string(&self) -> String {
        match self {
            StepAction::JupiterSharedAccountsRoute => "jupiter_shared_accounts_route",
            StepAction::JupiterSharedAccountsRouteV2 => "jupiter_shared_accounts_route_v2",
            StepAction::JupiterRoute => "jupiter_route",
            StepAction::JupiterRouteV2 => "jupiter_route_v2",
            StepAction::JupiterOrderEngineFill => "jupiter_order_engine_fill",
            StepAction::OkxSwapTobV3 => "okx_swap_tob_v3",
            StepAction::OkxSwapV3 => "okx_swap_v3",
            StepAction::OkxSwapTobV3WithReceiver => "okx_swap_tob_v3_with_receiver",
            StepAction::OkxSwap => "okx_swap",
            StepAction::DFlowSwap => "dflow_swap",
            StepAction::DFlowSwap2 => "dflow_swap2",
            StepAction::JlpPerpetualsAddLiquidity2 => "jlp_perpetuals_add_liquidity2",
            StepAction::JlpPerpetualsRemoveLiquidity2 => "jlp_perpetuals_remove_liquidity2",
        }
        .to_string()
    }
}
