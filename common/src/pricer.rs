use chrono::NaiveDate;
use anyhow::Result;
use std::{
    fs::File,
    io::{Cursor, Read, Write},
};
use csv::ReaderBuilder;
use crate::models::{KlineData, KlineRecord};
use zip::ZipArchive;

pub async fn fetch_klines_for_date(symbol: &str, date: NaiveDate) -> Result<Vec<KlineData>> {
    // Map our simple symbols to Binance tickers.
    let ticker = match symbol {
        "BTC" => "BTCUSDC",
        "SOL" => "SOLUSDC",
        _ => return Err(anyhow::Error::msg("Unknown symbol")),
    };

    let date_str = date.format("%Y-%m-%d").to_string();
    // URL for 1‑hour klines (adjust the interval if needed)
    // https://data.binance.vision/data/spot/daily/klines/SOLUSDC/1s/SOLUSDC-1s-2025-02-19.zip
    let url = format!(
        "https://data.binance.vision/data/spot/daily/klines/{ticker}/1s/{ticker}-1s-{date_str}.zip",
        ticker = ticker,
        date_str = date_str
    );

    println!("Downloading {} klines data from {}", symbol, url);
    let response = reqwest::get(&url).await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }
    let bytes = response.bytes().await?;

    // Unzip the CSV in-memory.
    let cursor = Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor)?;
    let mut csv_file = zip.by_index(0)?;
    let mut csv_data = String::new();
    csv_file.read_to_string(&mut csv_data)?;

    // Parse the CSV.
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv_data.as_bytes());
    let mut klines = Vec::new();
    for result in reader.deserialize() {
        let record: KlineRecord = result?;
        klines.push(KlineData::from(record));
    }
    Ok(klines)
}

pub fn store_klines(symbol: &str, date: &str, klines: &Vec<KlineData>) -> Result<()> {
    let filename = format!("{}_{}.bin", symbol, date);
    let encoded: Vec<u8> = bincode::serialize(klines)?;
    let mut file = File::create(filename.clone())?;
    file.write_all(&encoded)?;
    println!("Storing {} klines for {} to {}", klines.len(), symbol, filename);
    Ok(())
}