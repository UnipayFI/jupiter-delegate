use anchor_lang::prelude::*;

#[constant]
pub const VAULT_SEED: &str = "jupiter-delegate-vault";

#[constant]
pub const CONFIG_SEED: &str = "jupiter-delegate-config";

#[constant]
pub const ACCESS_SEED: &str = "jupiter-delegate-access";

#[constant]
pub const MINIMUM_TRADE_INTERVAL: i64 = 60; // 1 minute
