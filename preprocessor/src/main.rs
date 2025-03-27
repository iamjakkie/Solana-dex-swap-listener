mod models;
mod preprocessor;

use anyhow::Result;
use avro_rs::types::{Record, Value};
use chrono::{Datelike, NaiveDateTime, Timelike};
use common::models::{TradeData};
use csv::Reader;
use lazy_static::lazy_static;
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

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

lazy_static::lazy_static! {
    pub static ref AVRO_SCHEMA_TRADE: Schema = Schema::parse_str(r#"
    {
        "type": "record",
        "name": "TradeData",
        "fields": [
            { "name": "block_date", "type": "string" },
            { "name": "block_time", "type": "long" },
            { "name": "block_slot", "type": "long" },
            { "name": "signature", "type": "string" },
            { "name": "tx_id", "type": "string" },
            { "name": "signer", "type": "string" },
            { "name": "pool_address", "type": "string" },
            { "name": "base_mint", "type": "string" },
            { "name": "quote_mint", "type": "string" },
            { "name": "base_vault", "type": "string" },
            { "name": "quote_vault", "type": "string" },
            { "name": "base_amount", "type": "double" },
            { "name": "quote_amount", "type": "double" },
            { "name": "is_inner_instruction", "type": "boolean" },
            { "name": "instruction_index", "type": "int" },
            { "name": "instruction_type", "type": "string" },
            { "name": "inner_instruction_index", "type": "int" },
            { "name": "outer_program", "type": "string" },
            { "name": "inner_program", "type": "string" },
            { "name": "txn_fee_lamports", "type": "long" },
            { "name": "signer_lamports_change", "type": "long" }
        ]
    }
    "#).expect("Failed to parse Avro schema");
}

fn fix(path: &str) {
    // 1. rename folder to _old
    // 2. create new output folder
    // 3. combine files with same slot into single file
    // 4. save combined files to new output folder
    // 5. delete _old folder

    let mut files: HashMap<u64, Vec<String>> = HashMap::new();

    for folder in vec!["RAYDIUM", "METEORA", "ORCA"] {
        let folder_path = format!("{}/{}", path, folder);
        let paths = fs::read_dir(folder_path).unwrap();
        for path in paths {
            let path = path.unwrap().path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let slot = file_name.split(".").next().unwrap().parse::<u64>().unwrap();
            let file = files.entry(slot).or_insert(vec![]);
            file.push(path.to_str().unwrap().to_string());
        }
    }

    // combine files with same slot into single file

    for (slot, file_paths) in files {
        // read avro
        let mut records = vec![];
        for file_path in file_paths {
            let file = File::open(file_path).unwrap();
            let reader = avro_rs::Reader::new(BufReader::new(file)).unwrap();
            let trades: Vec<TradeData> = reader.map(|r| avro_rs::from_value(&r.unwrap()).unwrap()).collect();
            records.extend(trades);
        }

        let output_file_path = format!("{}/{}.avro", path, slot);
        let output_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(output_file_path)
            .unwrap();
        let mut writer = Writer::new(&AVRO_SCHEMA_TRADE, output_file);
        for trade in records {
            let mut record = Record::new(&AVRO_SCHEMA_TRADE).expect("Failed to create Avro record");
            record.put("block_date", trade.block_date.clone());
            record.put("block_time", trade.block_time);
            record.put("block_slot", trade.block_slot as i64);
            record.put("signature", trade.signature.clone());
            record.put("tx_id", trade.tx_id.clone());
            record.put("signer", trade.signer.clone());
            record.put("pool_address", trade.pool_address.clone());
            record.put("base_mint", trade.base_mint.clone());
            record.put("quote_mint", trade.quote_mint.clone());
            record.put("base_vault", trade.base_vault.clone());
            record.put("quote_vault", trade.quote_vault.clone());
            record.put("base_amount", trade.base_amount);
            record.put("quote_amount", trade.quote_amount);
            record.put("is_inner_instruction", trade.is_inner_instruction);
            record.put("instruction_index", trade.instruction_index as i32);
            record.put("instruction_type", trade.instruction_type.clone());
            record.put(
                "inner_instruction_index",
                trade.inner_instruction_index as i32,
            );
            record.put("outer_program", trade.outer_program.clone());
            record.put("inner_program", trade.inner_program.clone());
            record.put("txn_fee_lamports", trade.txn_fee_lamports as i64);
            record.put(
                "signer_lamports_change",
                trade.signer_lamports_change as i64,
            );
            writer.append(record).unwrap();
        }
        
        writer.flush().unwrap();
    }
}

#[tokio::main]
async fn main() {
    let path = format!("{}", env::var("DATA_PATH").unwrap());
    // fix(&path);
    let preprocessor = preprocessor::Preprocessor::new(&path, "2025-01-01").await;

    let preprocessor = Arc::new(preprocessor);
    let start_time = Instant::now();
    preprocessor.run().await; 
    println!("Time taken: {:?}", start_time.elapsed());
}
