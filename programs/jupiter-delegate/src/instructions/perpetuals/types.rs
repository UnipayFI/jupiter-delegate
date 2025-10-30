#[derive(Debug, PartialEq)]
pub struct AddLiquidity2Params {
    token_amount_in: u64,
    min_lp_amount_out: u64,
    token_amount_pre_swap: Option<u64>,
}

impl AddLiquidity2Params {
    pub fn new(
        token_amount_in: u64,
        min_lp_amount_out: u64,
        token_amount_pre_swap: Option<u64>,
    ) -> Self {
        Self {
            token_amount_in,
            min_lp_amount_out,
            token_amount_pre_swap,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RemoveLiquidity2Params {
    lp_amount_in: u64,
    min_amount_out: u64,
}

impl RemoveLiquidity2Params {
    pub fn new(lp_amount_in: u64, min_amount_out: u64) -> Self {
        Self {
            lp_amount_in,
            min_amount_out,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum JupiterPerpetualsAction {
    AddLiquidity(AddLiquidity2Params),
    RemoveLiquidity(RemoveLiquidity2Params),
}

impl JupiterPerpetualsAction {
    pub fn get_input_amount(&self) -> u64 {
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
