mod models;
mod preprocessor;

use anyhow::Result;
use avro_rs::types::{Record, Value};
use chrono::{Datelike, NaiveDateTime, Timelike};
use csv::Reader;
use lazy_static::lazy_static;
use std::collections::{BTreeSet, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

use avro_rs::{Codec, Schema, Writer};

lazy_static! {
    pub static ref AVRO_SCHEMA: Schema = Schema::parse_str(
        r#"
    {
        "type": "record",
        "name": "TradeData",
        "fields": [
            { "name": "block_time", "type": "long" },
            { "name": "block_slot", "type": "long" },
            { "name": "signature", "type": "string" },
            { "name": "tx_id", "type": "string" },
            { "name": "signer", "type": "string" },
            { "name": "pool_address", "type": "string" },
            { "name": "base_mint", "type": "string" },
            { "name": "quote_mint", "type": "string" },
            { "name": "base_amount", "type": "double" },
            { "name": "quote_amount", "type": "double" },
            { "name": "instruction_type", "type": "string" }
        ]
    }
    "#
    )
    .expect("Failed to parse Avro schema");
}

// pub fn csv_to_avro_per_slot(csv_files: &Vec<String>, avro_folder: &str) -> Result<()> {
//     std::fs::create_dir_all(avro_folder)?;

//     for csv_path in csv_files {
//         let csv_path = std::path::Path::new(csv_path);

//         if csv_path.extension().map_or(false, |ext| ext == "csv") {
//             let slot_filename = csv_path.file_stem().unwrap().to_str().unwrap();
//             let avro_file_path = format!("{}/{}.avro", avro_folder, slot_filename);

//             let file = OpenOptions::new().create(true).append(true).open(&avro_file_path)?;
//             let mut writer = Writer::with_codec(&AVRO_SCHEMA, BufWriter::new(file), Codec::Deflate);

//             let mut rdr = Reader::from_path(&csv_path)?;
//             for result in rdr.deserialize::<TradeData>() {
//                 let trade = result?;

//                 let mut record = Record::new(&AVRO_SCHEMA).unwrap();
//                 record.put("block_time", trade.block_time);
//                 record.put("block_slot", trade.block_slot as i64);
//                 record.put("signature", trade.signature);
//                 record.put("tx_id", trade.tx_id);
//                 record.put("signer", trade.signer);
//                 record.put("pool_address", trade.pool_address);
//                 record.put("base_mint", trade.base_mint);
//                 record.put("quote_mint", trade.quote_mint);
//                 record.put("base_amount", trade.base_amount);
//                 record.put("quote_amount", trade.quote_amount);
//                 record.put("instruction_type", trade.instruction_type);

//                 writer.append(record)?;
//             }

//             writer.flush()?;
//             println!("✅ Converted {} → {}", csv_path.display(), avro_file_path);
//         }
//     }

//     Ok(())
// }

pub fn merge_avro_per_hour(raw_avro_folder: &str, output_folder: &str) -> Result<()> {
    // Ensure output directory exists
    fs::create_dir_all(output_folder)?;

    let mut hourly_writers: HashMap<String, Writer<BufWriter<File>>> = HashMap::new();

    // Process all slot Avro files
    for entry in fs::read_dir(raw_avro_folder)? {
        let entry = entry?;
        let avro_path = entry.path();

        println!("Processing {}", avro_path.display());

        if avro_path.extension().map_or(false, |ext| ext == "avro") {
            if fs::metadata(&avro_path)?.len() == 0 {
                println!("🛑 Skipping empty file: {}", avro_path.display());
                continue;
            }
            let file = File::open(&avro_path)?;
            let reader = avro_rs::Reader::new(file)?;

            // Process each record in the slot Avro file
            for value in reader {
                if let Value::Record(record) = value? {
                    if let Some((_, Value::Long(block_time))) =
                        record.iter().find(|(key, _)| key == "block_time")
                    {
                        let dt = NaiveDateTime::from_timestamp_opt(*block_time, 0)
                            .expect("Invalid timestamp");

                        // Construct the correct hourly key: YYYY-MM-DD-HH
                        let hour_key = format!(
                            "{:04}-{:02}-{:02}-{:02}",
                            dt.year(),
                            dt.month(),
                            dt.day(),
                            dt.hour()
                        );
                        let output_avro_path = format!("{}/{}.avro", output_folder, hour_key);

                        // Get or create the writer for this hour
                        let writer = hourly_writers.entry(hour_key.clone()).or_insert_with(|| {
                            let file = OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(&output_avro_path)
                                .unwrap();
                            Writer::with_codec(&AVRO_SCHEMA, BufWriter::new(file), Codec::Deflate)
                        });

                        // Append the record to the correct hourly Avro file
                        writer.append(Value::Record(record))?;
                    }
                }
            }
        }
    }

    // Flush all writers to finalize Avro files
    for (hour, mut writer) in hourly_writers {
        writer.flush()?;
        println!("✅ Merged slot files → {}/{}.avro", output_folder, hour);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Assumption - this gets triggered once every day
    let binding = (chrono::Utc::today().naive_utc() - chrono::Duration::days(1))
        .to_string();
    let yesterday = binding
        .as_str();
    let path = "/Users/jakkie/Dev/solana_data/test/";
    let preprocessor = preprocessor::Preprocessor::new(path, "2025-01-30").await;
    println!("Preprocessor {:?}", preprocessor.path);
    let preprocessor = Arc::new(preprocessor);
    
    preprocessor.run().await; 
}
