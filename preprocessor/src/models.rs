use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMeta {
    pub contract_address: String,
    pub token_name: String,
    pub token_symbol: String,
    pub decimals: i32,
    pub total_supply: Option<f64>,
    pub creator: String,
    pub created_time: i64,
    pub twitter: Option<String>,
    pub website: Option<String>,
}

