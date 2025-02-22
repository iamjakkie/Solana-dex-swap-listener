
use std::{collections::{BTreeSet, HashMap}, env, fs::{self, File}, path::Path, sync::Arc};
use anyhow::{Ok, Result};
use common::models::TradeData;
use lazy_static::lazy_static;
use native_tls::{TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use tokio::sync::Mutex;


use crate::models::TokenMeta;

lazy_static!(
    // SOLSCAN API KEY FROM ENV
    pub static ref SOLSCAN_API_KEY: String = env::var("SOLSCAN_API_KEY").expect("SOLSCAN_API_KEY must be set");
);

pub struct Preprocessor<'a>{
    pub path: &'a Path,
    pub db_client: tokio_postgres::Client,
    token_meta_map: Arc<Mutex<HashMap<String, TokenMeta>>>,
}

impl<'a> Preprocessor<'a>{
    pub async fn new(path: &'a str) -> Self {
        let base_path = Path::new(path);
        if !base_path.exists() {
            panic!("Directory does not exist!");
        }

        let connector = TlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap();
        let connector = MakeTlsConnector::new(connector);

        let (client, connection) =
            tokio_postgres::connect(&get_database_url(), connector)
                .await
                .unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let preprocessor = Preprocessor{
            path: base_path,
            db_client: client,
            token_meta_map: Arc::new(Mutex::new(HashMap::new())),
        };

        preprocessor.load_token_meta().await.expect("Failed to load token meta");

        preprocessor
    }

    async fn load_token_meta(&self) -> Result<()> {
        let rows = self.db_client
            .query("SELECT * FROM token_meta", &[])
            .await
            .expect("Failed to fetch token meta");

        let mut token_meta_map = self.token_meta_map.lock().await;
        for row in rows {
            let token_meta = TokenMeta {
                contract_address: row.get(0),
                token_name: row.get(1),
                token_symbol: row.get(2),
                decimals: row.get(3),
                total_supply: row.get(4),
                creator: row.get(5),
                created_time: row.get(6),
                twitter: row.get(7),
                website: row.get(8),
            };

            token_meta_map.insert(token_meta.contract_address.clone(), token_meta);
        }

        Ok(())
    }

    fn get_raw_files(&self, dir: &str) -> Vec<String> {
        let mut files = vec![];

        println!("{:?}", dir);

        for entry in fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "csv" || ext == "avro") {
                files.push(path.to_string_lossy().into_owned());
            }
        }

        files.sort(); // Ensure processing in order
        files
    }

    async fn save_missing_slots(&self, missing_slots: &[u64]) -> Result<()> {
        let mut query = "INSERT INTO missing_slots (slot) VALUES ".to_string();
        for slot in missing_slots {
            query.push_str(&format!("({}),", slot));
        }

        query.pop(); // Remove trailing comma
        query.push(';');

        self.db_client
            .execute(query.as_str(), &[])
            .await
            .expect("Failed to save missing slots");

        println!("📝 Saved {} missing slots to lacking.txt", missing_slots.len());
        Ok(())
    }

    async fn get_token_meta(&self, token_address: &str) -> Result<()> {
        // try to find in self.token_meta_map first
        let token_meta_map = self.token_meta_map.lock().await;
        if let Some(token_meta) = token_meta_map.get(token_address) {
            println!("{:?}", token_meta);
            return Ok(());
        }

        // else go to API
        let url = format!("https://pro-api.solscan.io/v2.0/token/meta?address={}", token_address);
        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .header("token", &*SOLSCAN_API_KEY)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

            let data = res.get("data").expect("Failed to get data");

            let token_meta = TokenMeta {
                contract_address: token_address.to_string(),
                token_name: data.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                token_symbol: data.get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                decimals: data.get("decimals")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as i32,
                total_supply: data.get("totalSupply")
                    .and_then(|v| v.as_f64()),
                creator: data.get("creator")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                created_time: data.get("createdTime")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as i64)
                    .unwrap_or(0),
                twitter: data.get("twitter")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                website: data.get("website")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };

            // add to self.token_meta_map
            let mut token_meta_map = self.token_meta_map.lock().await;
            token_meta_map.insert(token_meta.contract_address.clone(), token_meta);

        Ok(())
    }

    async fn merge_into_hourly(&self, raw_files: &Vec<String>, folder: &String) -> Result<()> {
    //     // combine into hourly
    //     // insert to db empty files
    //     // get metadata for tokens
        fs::create_dir_all(&folder)?;

        for file in raw_files {
            if file.ends_with(".csv") {
                let mut rdr = csv::Reader::from_path(&file)?;
                for result in rdr.deserialize::<TradeData>() {
                    let trade = result?;
                    // get token which is not So11111111111111111111111111111111111111112
                    let traded_token = if trade.base_mint != "So11111111111111111111111111111111111111112" {
                        trade.base_mint.clone()
                    } else {
                        trade.quote_mint.clone()
                    };

                    let meta = self.get_token_meta(&traded_token).await.expect("Failed to get token meta");
                }
            } else if file.ends_with(".avro") {
                let mut rdr = avro_rs::Reader::new(File::open(&file)?)?;
                for result in rdr {
                    let value = result?;
                    let trade: TradeData = avro_rs::from_value(&value)?;
                    let traded_token = if trade.base_mint != "So11111111111111111111111111111111111111112" {
                        trade.base_mint.clone()
                    } else {
                        trade.quote_mint.clone()
                    };

                    let meta = self.get_token_meta(&traded_token).await.expect("Failed to get token meta");
                }
            } else {
                continue;
            }

                // let mut record = Record::new(&AVRO_SCHEMA).unwrap();
                // record.put("block_time", trade.block_time);
                // record.put("block_slot", trade.block_slot as i64);
                // record.put("signature", trade.signature);
                // record.put("tx_id", trade.tx_id);
                // record.put("signer", trade.signer);
                // record.put("pool_address", trade.pool_address);
                // record.put("base_mint", trade.base_mint);
                // record.put("quote_mint", trade.quote_mint);
                // record.put("base_amount", trade.base_amount);
                // record.put("quote_amount", trade.quote_amount);
                // record.put("instruction_type", trade.instruction_type);

                // writer.append(record)?;
            }

            Ok(())
        }

    async fn process(&self, folder: &str) -> Result<()> {
        let raw_files = self.get_raw_files(folder);
        // let missing_slots = check_missing_slots(&raw_files)?;

        // if !missing_slots.is_empty() {
        //     self.save_missing_slots(&missing_slots).await.expect("Failed to save missing slots");
        // }

        self.merge_into_hourly(&raw_files, &format!("{}/hourly", folder)).await.expect("Failed to merge into hourly");


        Ok(())
    }

    pub async fn run(&self) {
        // get all dates
        // check if for these dates _hourly folder exist

        //1 . get all folders
        let folders = list_directories(self.path.to_str().unwrap());

        for folder in folders {
            self.process(&format!("{}{}", self.path.to_str().unwrap(), folder)).await;
        }

        // for entry in fs::read_dir(self.path).unwrap() {
        //     let entry = entry.unwrap();
        //     let path = entry.path();

        //     println!("{:?}", path);
            
        //     if path.is_dir() {
        //         println!("Is dir");
        //         let folder_name = path.file_name().unwrap().to_str().unwrap();
        //         let hourly_folder = format!("{:?}/{}/hourly", self.path, folder_name);

        //         if !Path::new(&hourly_folder).exists() {
        //             println!("Does not exist");
        //             let raw_files = self.get_raw_files();
        //             let missing_slots = check_missing_slots(&raw_files).unwrap();

        //             if !missing_slots.is_empty() {
        //                 self.save_missing_slots(&missing_slots).await.expect("Failed to save missing slots");
        //             }



        //             // merge_into_hourly(&csv_files, &hourly_folder)?;
        //         }
        //     }
        // }
    }
}

fn check_missing_slots(raw_files: &[String]) -> Result<Vec<u64>> {
    let mut slots: BTreeSet<u64> = BTreeSet::new();

    for file in raw_files {
        if let Some(slot) = extract_slot_from_filename(file) {
            slots.insert(slot);
        }
    }

    let min_slot = *slots.iter().next().unwrap_or(&0);
    let max_slot = *slots.iter().last().unwrap_or(&0);

    let mut missing_slots = vec![];
    for slot in min_slot..=max_slot {
        if !slots.contains(&slot) {
            missing_slots.push(slot);
        }
    }

    Ok(missing_slots)
}

fn extract_slot_from_filename(filename: &str) -> Option<u64> {
    filename
        .rsplit('/')
        .next()?
        .strip_suffix(".csv")?
        .parse::<u64>()
        .ok()
}

fn list_directories(path: &str) -> Vec<String> {
    let mut folders = vec![];

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            let folder_name = path.file_name().unwrap().to_str().unwrap();
            folders.push(folder_name.to_string());
        }
    }

    folders
}

pub fn get_database_url() -> String {
    let host = env::var("DB_HOST").expect("DB_HOST must be set");
    let user = env::var("DB_USER").expect("DB_USER must be set");
    let password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");

    format!("postgres://{}:{}@{}/{}", user, password, host, db_name)
}