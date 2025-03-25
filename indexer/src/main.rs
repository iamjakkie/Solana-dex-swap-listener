use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use common::{
    block_processor::process_block,
    rpc_client::{fetch_block_with_version},
};
use tokio::sync::{RwLock, Semaphore};
use zmq;

async fn run_indexer(/*publisher_arc: Option<Arc<Mutex<zmq::Socket>>>*/) {
        println!("Starting indexer");
        let start_slot = 251293572;
        let end_slot = 323756378;
        // let start_slot = 311292619;
        // let end_slot = start_slot + 20;

        let max_concurrent_tasks = 25; // Limit to 10 concurrent tasks
        let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));
        let mut handles = Vec::new();
        for block_num in (start_slot..end_slot).rev() {
            let permit = semaphore.clone().acquire_owned().await.unwrap(); // Acquire a permit
            // if publisher_arc.is_some() {
            //     publisher_clone = Arc::clone(&publisher_arc.unwrap().clone());
            // }
            
            let handle = tokio::spawn(async move {
                // let start_time = Instant::now();
                let block = fetch_block_with_version(block_num).await;
                // let block_end_time = start_time.elapsed();
                // println!("Block {} fetched in {:?}", block_num, block_end_time);
                match block {
                    Ok(_) => {
                        let block = block.unwrap();

                        // println!("Processing block: {}", block_num);
                        // spawn a new thread to process_block
                        // tokio::spawn(async move {
                        // let start_time = Instant::now();
                        process_block(block_num, block, /*Some(publisher_clone)*/None).await;
                        // });
                        // let elapsed = start_time.elapsed();
                        // println!("Block {} processed in {:?}", block_num, elapsed);
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
                drop(permit);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
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

    let start = Instant::now();
    run_indexer().await;
    let duration = start.elapsed();
    println!("Indexer ran for {:?}", duration);
}
