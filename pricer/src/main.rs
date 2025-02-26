use anyhow::Result;
use chrono::NaiveDate;
use common::{pricer::{fetch_klines_for_date, store_klines}};

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
                }
                Err(e) => {
                    println!("Error fetching {} on {}: {}", symbol, current_date, e);
                }
            }
            current_date = current_date.succ();
        }
    }
    Ok(())
}
