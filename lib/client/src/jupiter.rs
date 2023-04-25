use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub data: Vec<QueryRoute>,
    pub time_taken: f64,
    pub context_slot: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryRoute {
    pub in_amount: String,
    pub out_amount: String,
    pub price_impact_pct: f64,
    pub market_infos: Vec<QueryMarketInfo>,
    pub amount: String,
    pub slippage_bps: u64,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub fees: Option<QueryRouteFees>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryMarketInfo {
    pub id: String,
    pub label: String,
    pub input_mint: String,
    pub output_mint: String,
    pub not_enough_liquidity: bool,
    pub in_amount: String,
    pub out_amount: String,
    pub min_in_amount: Option<String>,
    pub min_out_amount: Option<String>,
    pub price_impact_pct: Option<f64>,
    pub lp_fee: QueryFee,
    pub platform_fee: QueryFee,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryFee {
    pub amount: String,
    pub mint: String,
    pub pct: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryRouteFees {
    pub signature_fee: f64,
    pub open_orders_deposits: Vec<f64>,
    pub ata_deposits: Vec<f64>,
    pub total_fee_and_deposits: f64,
    #[serde(rename = "minimalSOLForTransaction")]
    pub minimal_sol_for_transaction: f64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapRequest {
    pub route: QueryRoute,
    pub user_public_key: String,
    #[serde(rename = "wrapUnwrapSOL")]
    pub wrap_unwrap_sol: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    pub setup_transaction: Option<String>,
    pub swap_transaction: String,
    pub cleanup_transaction: Option<String>,
}
