use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct KlineRecord {
    #[serde(rename = "Open time")]
    open_time: u64,
    #[serde(rename = "Open")]
    open: f64,
    #[serde(rename = "High")]
    high: f64,
    #[serde(rename = "Low")]
    low: f64,
    #[serde(rename = "Close")]
    close: f64,
    #[serde(rename = "Volume")]
    volume: f64,
    #[serde(rename = "Close time")]
    close_time: u64,
    #[serde(rename = "Quote asset volume")]
    quote_asset_volume: f64,
    #[serde(rename = "Number of trades")]
    number_of_trades: u64,
    #[serde(rename = "Taker buy base asset volume")]
    taker_buy_base_asset_volume: f64,
    #[serde(rename = "Taker buy quote asset volume")]
    taker_buy_quote_asset_volume: f64,
    #[serde(rename = "Ignore")]
    ignore: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KlineData {
    open_time: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    close_time: u64,
    quote_asset_volume: f64,
    number_of_trades: u64,
    taker_buy_base_asset_volume: f64,
    taker_buy_quote_asset_volume: f64,
    ignore: u64,
}

impl From<KlineRecord> for KlineData {
    fn from(rec: KlineRecord) -> Self {
        KlineData {
            open_time: rec.open_time,
            open: rec.open,
            high: rec.high,
            low: rec.low,
            close: rec.close,
            volume: rec.volume,
            close_time: rec.close_time,
            quote_asset_volume: rec.quote_asset_volume,
            number_of_trades: rec.number_of_trades,
            taker_buy_base_asset_volume: rec.taker_buy_base_asset_volume,
            taker_buy_quote_asset_volume: rec.taker_buy_quote_asset_volume,
            ignore: rec.ignore,
        }
    }
}
