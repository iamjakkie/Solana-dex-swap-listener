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
    /// Block timestamp (in seconds or milliseconds).
    pub block_time: i64,
    /// Block slot number.
    pub block_slot: u64,
    pub signature: String,
    pub exchange: String,
    /// The token being traded – the token that isn’t the quote asset.
    pub token: String,
    pub side: Side,
    /// The amount of the traded token.
    pub token_amount: f64,
    /// The symbol of the quote asset (e.g. "SOL" or "USDC").
    pub quote_asset: String,
    /// The amount of the quote asset.
    pub quote_amount: f64,
    /// Derived price: quote_amount / token_amount.
    pub derived_price: f64,
    /// USD price of the traded token. If the quote asset is USDC, this would be the same as derived_price,
    /// but if it's SOL, you’d convert SOL to USD using an external price feed.
    pub usd_price: f64,
    /// The traded volume (in token units).
    pub volume: f64,
    /// The token's market capitalization computed as (derived_price * total_supply), if available.
    pub market_cap: f64,
}