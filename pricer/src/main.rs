mod models;

use std::{fs::File, io::{Cursor, Read, Write}};

use anyhow::Result;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use models::{KlineData, KlineRecord};
use zip::ZipArchive;

async fn fetch_klines_for_date(symbol: &str, date: NaiveDate) -> Result<Vec<KlineData>> {
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
    let mut reader = ReaderBuilder::new().has_headers(false).from_reader(csv_data.as_bytes());
    let mut klines = Vec::new();
    for result in reader.deserialize() {
        let record: KlineRecord = result?;
        klines.push(KlineData::from(record));
    }
    Ok(klines)
}

fn store_klines(symbol: &str, date: &str, klines: &Vec<KlineData>) -> Result<()> {
    let filename = format!("{}_{}.bin", symbol, date);
    let encoded: Vec<u8> = bincode::serialize(klines)?;
    let mut file = File::create(filename)?;
    file.write_all(&encoded)?;
    println!("Stored {} klines for {}.", klines.len(), symbol);
    Ok(())
}



#[tokio::main]
async fn main() -> Result<()> {
    // Starting Jan 1st, adjust year if needed.
    let start_date = NaiveDate::from_ymd(2025, 1, 1);
    let end_date = chrono::Utc::today().naive_utc();
    let symbols = vec!["BTC", "SOL"];
    
    let mut current_date = start_date;
    // Loop through each day.
    while current_date <= end_date {
        for symbol in symbols.iter() {
            println!("Fetching {} klines for {}", symbol, current_date);
            match fetch_klines_for_date(symbol, current_date).await {
                Ok(data) => {
                    store_klines(symbol, current_date.to_string().as_str(), &data)?;
                },
                Err(e) => {
                    println!("Error fetching {} on {}: {}", symbol, current_date, e);
                },
            }
            current_date = current_date.succ();
        }
    }
    Ok(())
}