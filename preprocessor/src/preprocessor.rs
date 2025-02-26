use anyhow::{Ok, Result};
use common::{
    block_processor::process_block, models::{KlineData, TradeData}, rpc_client::fetch_block_with_version,
};

use lazy_static::lazy_static;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    env,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{Mutex, Semaphore},
    time,
};

use crate::models::TokenMeta;

lazy_static!(
    // SOLSCAN API KEY FROM ENV
    pub static ref SOLSCAN_API_KEY: String = env::var("SOLSCAN_API_KEY").expect("SOLSCAN_API_KEY must be set");
);

pub struct Preprocessor {
    pub path: PathBuf,
    pub db_client: tokio_postgres::Client,
    token_meta_map: Arc<Mutex<HashMap<String, TokenMeta>>>,
    sol_prices: Vec<KlineData>,
}

impl Preprocessor {
    pub async fn new(path: &str) -> Self {
        let base_path = Path::new(path);
        if !base_path.exists() {
            panic!("Directory does not exist!");
        }

        let connector = TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
        let connector = MakeTlsConnector::new(connector);

        let (client, connection) = tokio_postgres::connect(&get_database_url(), connector)
            .await
            .unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let date = path.split('/').last().unwrap();

        let prices = load_prices(date).await.expect("Failed to load prices");

        let preprocessor = Preprocessor {
            path: base_path.to_path_buf(),
            db_client: client,
            token_meta_map: Arc::new(Mutex::new(HashMap::new())),
        };

        preprocessor
            .load_token_meta()
            .await
            .expect("Failed to load token meta");

        preprocessor
    }

    pub async fn start_token_meta_dump(&self) {
        // Choose an interval (e.g., every 10 minutes)
        let mut interval = time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            println!("DUMP");
            if let Err(e) = self.dump_token_meta_to_db().await {
                println!("Error dumping token meta to DB: {}", e);
            }
        }
    }

    pub async fn dump_token_meta_to_db(&self) -> Result<()> {
        let token_meta_map = self.token_meta_map.lock().await;

        if token_meta_map.is_empty() {
            return Ok(());
        }
        // Load current state from DB into a local HashMap keyed by contract_address.
        let rows = self
            .db_client
            .query("SELECT contract_address FROM token_meta", &[])
            .await?;

        let mut db_state: HashSet<String> = HashSet::new();
        rows.iter().for_each(|row| {
            db_state.insert(row.get(0));
        });

        // calculate the difference
        let token_meta_set: HashSet<String> = token_meta_map.keys().cloned().collect();

        let new_tokens: HashSet<String> = token_meta_set.difference(&db_state).cloned().collect();

        // construct query

        let mut query = "INSERT INTO token_meta (contract_address, token_name, token_symbol, decimals, total_supply, creator, created_time, twitter, website) VALUES ".to_string();

        for contract in new_tokens {
            let meta = token_meta_map.get(&contract).unwrap();
            query.push_str(&format!(
                "('{}', '{}', '{}', {}, {}, '{}', {}, '{}', '{}'),",
                meta.contract_address,
                meta.token_name,
                meta.token_symbol,
                meta.decimals,
                meta.total_supply.unwrap_or(0.0),
                meta.creator,
                meta.created_time,
                meta.twitter.as_deref().unwrap_or(""),
                meta.website.as_deref().unwrap_or("")
            ));
        }

        query.pop(); // Remove trailing comma

        self.db_client.execute(query.as_str(), &[]).await?;

        println!("Token meta successfully dumped to DB at");
        Ok(())
    }

    async fn load_token_meta(&self) -> Result<()> {
        let rows = self
            .db_client
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
            if path
                .extension()
                .map_or(false, |ext| ext == "csv" || ext == "avro")
            {
                files.push(path.to_string_lossy().into_owned());
            }
        }

        files.sort(); // Ensure processing in order
        files
    }

    fn verify_slot(&self, file: &str) -> bool {
        // check size of the file
        let file = format!("{}/{}", self.path.to_str().unwrap(), file);
        let metadata = fs::metadata(file).expect("Failed to get metadata");
        if metadata.len() <= 10 {
            return false;
        } else {
            return true;
        }
    }

    fn check_missing_slots(&self, raw_files: &[String]) -> Result<Vec<u64>> {
        let mut slots: BTreeSet<u64> = BTreeSet::new();

        for file in raw_files {
            if let Some(slot) = extract_slot_from_filename(file) {
                // check if file is empty and/or corrupted
                if self.verify_slot(file.as_str()) {
                    continue;
                } else {
                    slots.insert(slot);
                }
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

    async fn reprocess_slots(&self, missing_slots: &Vec<u64>) -> Result<()> {
        let max_concurrent_tasks = 10;
        let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));
        for slot in missing_slots.clone() {
            let permit = semaphore.clone().acquire_owned().await?;
            tokio::spawn(async move {
                let block = fetch_block_with_version(slot)
                    .await
                    .expect("Failed to fetch block");
                process_block(block, None).await;
                drop(permit);
            });
        }
        Ok(())
    }

    async fn get_token_meta(&self, token_address: &str) -> Result<()> {
        // try to find in self.token_meta_map first
        {
            let token_meta_map = self.token_meta_map.lock().await;
            if let Some(token_meta) = token_meta_map.get(token_address) {
                return Ok(());
            }
        }

        // else go to API
        let url = format!(
            "https://pro-api.solscan.io/v2.0/token/meta?address={}",
            token_address
        );
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
            token_name: data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            token_symbol: data
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            decimals: data.get("decimals").and_then(|v| v.as_u64()).unwrap_or(0) as i32,
            total_supply: data.get("totalSupply").and_then(|v| v.as_f64()),
            creator: data
                .get("creator")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            created_time: data
                .get("createdTime")
                .and_then(|v| v.as_u64())
                .map(|v| v as i64)
                .unwrap_or(0),
            twitter: data
                .get("twitter")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            website: data
                .get("website")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        // add to self.token_meta_map
        let mut token_meta_map = self.token_meta_map.lock().await;
        token_meta_map.insert(token_meta.contract_address.clone(), token_meta.clone());

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
                    let traded_token =
                        if trade.base_mint != "So11111111111111111111111111111111111111112" {
                            trade.base_mint.clone()
                        } else {
                            trade.quote_mint.clone()
                        };

                    let meta = self
                        .get_token_meta(&traded_token)
                        .await
                        .expect("Failed to get token meta");
                    // get sol price
                }
            } else if file.ends_with(".avro") {
                let mut rdr = avro_rs::Reader::new(File::open(&file)?)?;
                for result in rdr {
                    let value = result?;
                    let trade: TradeData = avro_rs::from_value(&value)?;
                    let traded_token =
                        if trade.base_mint != "So11111111111111111111111111111111111111112" {
                            trade.base_mint.clone()
                        } else {
                            trade.quote_mint.clone()
                        };

                    let meta = self
                        .get_token_meta(&traded_token)
                        .await
                        .expect("Failed to get token meta");
                }
            } else {
                continue;
            }

        }

        Ok(())
    }

    async fn process(&self) -> Result<()> {
        let folder = self.path.to_str().unwrap();
        let raw_files = self.get_raw_files(folder);
        let missing_slots = self.check_missing_slots(&raw_files)?;

        if !missing_slots.is_empty() {
            self.reprocess_slots(&missing_slots).await?;
        }

        self.merge_into_hourly(&raw_files, &format!("{}_hourly", folder))
            .await
            .expect("Failed to merge into hourly");

        Ok(())
    }

    fn cleanup(&self) -> Result<()> {

    }

    pub async fn run(self: Arc<Self>) {
        let preprocessor_clone = Arc::clone(&self);
        tokio::spawn(async move {
            preprocessor_clone.start_token_meta_dump().await;
        });

        let _ = self.process().await;

        self.cleanup().expect("Failed to cleanup");
    }
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

fn load_prices(date: &str) -> Result<Vec<KlineData>> {
    
    
}