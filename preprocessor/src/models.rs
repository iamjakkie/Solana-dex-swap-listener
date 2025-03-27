use std::fmt;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Side::Buy => write!(f, "Buy"),
            Side::Sell => write!(f, "Sell"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessedTrade {
    /// The date of the block (e.g. "2025-01-30") for grouping and logging.
    pub block_date: String,
    /// Block timestamp (in seconds or milliseconds, as per your design).
    pub block_time: i64,
    /// Block slot number.
    pub block_slot: u64,
    pub signature: String,
    pub exchange: String,
    /// The token being traded – the token that isn’t SOL.
    pub token: String,
    pub side: Side,
    pub token_amount: f64,
    pub sol_amount: f64,
    pub sol_usd_price: f64,
    /// The derived price of the traded token (e.g. computed as quote_amount / base_amount).
    pub sol_price: f64,
    /// The USD price computed as token_price multiplied by the SOL price at the trade time.
    pub usd_price: f64,
    /// The traded volume in units of the traded token.
    pub volume: f64,
    // The token's market capitalization computed as token_price * total_supply (if available).
    pub market_cap: f64,
}