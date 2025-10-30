#![allow(dead_code)]

// add_liquidity2
pub const DISCRIMINATOR_ADD_LIQUIDITY: &[u8; 8] = &[228, 162, 78, 28, 70, 219, 116, 115];

// remove_liquidity2
pub const DISCRIMINATOR_REMOVE_LIQUIDITY: &[u8; 8] = &[230, 215, 82, 127, 241, 101, 227, 146];

// instant_increase_position
pub const DISCRIMINATOR_INSTANT_INCREASE_POSITION: &[u8; 8] =
    &[164, 126, 68, 182, 223, 166, 64, 183];

// instant_decrease_position
pub const DISCRIMINATOR_INSTANT_DECREASE_POSITION: &[u8; 8] = &[46, 23, 240, 44, 30, 138, 94, 140];

// instant_create_limit_order
pub const DISCRIMINATOR_INSTANT_CREATE_LIMIT_ORDER: &[u8; 8] =
    &[194, 37, 195, 123, 40, 127, 126, 156];
