use anchor_lang::prelude::*;

#[constant]
pub const VAULT_SEED: &str = "jupiter-delegate-vault";

#[constant]
pub const CONFIG_SEED: &str = "jupiter-delegate-config";

#[constant]
pub const ACCESS_SEED: &str = "jupiter-delegate-access";

#[constant]
pub const MINIMUM_TRADE_INTERVAL: i64 = 0; // 0 seconds

pub const DISCRIMINATOR_JUPITER_AGGREGATOR_SHARED_ACCOUNTS_ROUTE: &[u8] =
    &[193, 32, 155, 51, 65, 214, 156, 129];

pub const DISCRIMINATOR_JUPITER_AGGREGATOR_SHARED_ACCOUNTS_ROUTE_V2: &[u8] =
    &[53, 96, 229, 202, 216, 187, 250, 24];

pub const DISCRIMINATOR_JUPITER_AGGREGATOR_ROUTE: &[u8] = &[229, 23, 203, 151, 122, 227, 173, 42];

pub const DISCRIMINATOR_JUPITER_AGGREGATOR_ROUTE_V2: &[u8] =
    &[187, 100, 250, 204, 49, 196, 175, 20];

pub const DISCRIMINATOR_JUPITER_ORDER_ENGINE_FILL: &[u8] = &[168, 96, 183, 163, 92, 10, 40, 160];

pub const DISCRIMINATOR_OKX_SWAP_TOB_V3: &[u8] = &[63, 114, 246, 131, 51, 2, 247, 29];

pub const DISCRIMINATOR_OKX_SWAP_V3: &[u8] = &[240, 224, 38, 33, 176, 31, 241, 175];

pub const DISCRIMINATOR_OKX_SWAP_TOB_V3_WITH_RECEIVER: &[u8] =
    &[14, 191, 44, 246, 142, 225, 224, 157];

pub const DISCRIMINATOR_OKX_SWAP: &[u8] = &[248, 198, 158, 145, 225, 117, 135, 200];

pub const DISCRIMINATOR_DFLOW_SWAP: &[u8] = &[248, 198, 158, 145, 225, 117, 135, 200];

pub const DISCRIMINATOR_DFLOW_SWAP2: &[u8] = &[65, 75, 63, 76, 235, 91, 91, 136];
