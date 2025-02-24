use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use common::{
    block_processor::process_block,
    rpc_client::{fetch_block_with_version, get_latest_slot},
};
use tokio::sync::{RwLock, Semaphore};
use zmq;

async fn run_indexer(publisher_arc: Arc<Mutex<zmq::Socket>>) {
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
    let publisher = ctx
        .socket(zmq::PUB)
        .expect("Failed to create ZMQ PUB socket");
    publisher
        .bind(format!("tcp://*:{}", port).as_str())
        .expect("Failed to bind publisher");
    publisher
}

#[tokio::main]
async fn main() {
    let ctx = zmq::Context::new();
    let publisher = ctx
        .socket(zmq::PUB)
        .expect("Failed to create ZMQ PUB socket");
    publisher
        .bind("tcp://*:5555")
        .expect("Failed to bind publisher");

    // // 2. Wrap the publisher in an Arc<Mutex> so we can share it
    let publisher_arc = Arc::new(Mutex::new(publisher));

    run_indexer(publisher_arc).await;
    // TODO: options
    // 1. block limits - min, max
    // 2. order
    // 3. include csv save
    // 4. include zmq
}
