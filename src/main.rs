mod block_processor;
mod global;
mod models;
mod rpc_client;
mod trade_parser;
mod tx_processor;
mod utils;

use std::{sync::{Arc, Mutex}, time::Instant};

use block_processor::process_block;
use rpc_client::{fetch_block_with_version, get_latest_slot};
use tokio::sync::{RwLock, Semaphore};
use zmq;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Trade Indexer with Configurable Options", long_about = None)]
struct Cli {
    /// Minimum block to index
    #[arg(long, default_value_t = 0)]
    min_block: u64,

    /// Maximum block to index
    #[arg(long, default_value_t = u64::MAX)]
    max_block: u64,

    // filepath for missing blocks
    #[arg(long)]
    missing_blocks: String,

    /// Enable ZMQ publisher
    #[arg(long)]
    zmq: bool,

    /// Enable CSV saving
    #[arg(long)]
    csv: bool,

    /// Order of processing: asc or desc
    #[arg(long, default_value = "asc")]
    order: String,
}

async fn run_indexer(args: Cli) {
    let mut last_processed_slot: Option<u64> = None;

    loop {
        let latest_slot = get_latest_slot().await.expect("Failed to get latest slot");
        println!("Latest slot: {}", latest_slot);

        let start_slot = 315186613;

        let max_concurrent_tasks = 25; // Limit to 10 concurrent tasks
        let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));
        // 31.01.2025 - 317661530 (23:59:59)
        if start_slot <= latest_slot {
            for block_num in (start_slot..=317661530).rev() {
                let permit = semaphore.clone().acquire_owned().await.unwrap(); // Acquire a permit
                let publisher_clone = Arc::clone(&publisher_arc.clone());
                tokio::spawn(async move {
                    let start_time = Instant::now();
                    let block = fetch_block_with_version(block_num).await;
                    match block {
                        Ok(_) => {
                            let block = block.unwrap();
                            
                            println!("Processing block: {}", block_num);
                            // spawn a new thread to process_block
                            // tokio::spawn(async move {
                                process_block(block, publisher_clone).await;
                            // });
                            let elapsed = start_time.elapsed();
                            println!("Block {} processed in {:?}", block_num, elapsed);
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                    drop(permit);
                });
                last_processed_slot = Some(block_num);
                
            }
        }
    }
}

fn bind_zmq(port: &str) -> zmq::Socket {
    let ctx = zmq::Context::new();
    let publisher = ctx.socket(zmq::PUB).expect("Failed to create ZMQ PUB socket");
    publisher.bind(format!("tcp://*:{}", port).as_str()).expect("Failed to bind publisher");
    publisher
}

#[tokio::main]
async fn main() {
    
    // let ctx = zmq::Context::new();
    // let publisher = ctx.socket(zmq::PUB).expect("Failed to create ZMQ PUB socket");
    // publisher.bind("tcp://*:5555").expect("Failed to bind publisher");

    // // 2. Wrap the publisher in an Arc<Mutex> so we can share it
    // let publisher_arc = Arc::new(Mutex::new(publisher));
    
    
    run_indexer(publisher_arc).await;
    // TODO: options
    // 1. block limits - min, max
    // 2. order
    // 3. include csv save
    // 4. include zmq
    
}
