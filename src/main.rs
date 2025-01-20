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
use tokio::sync::RwLock;
use zmq;


#[tokio::main]
async fn main() {
    
    let ctx = zmq::Context::new();
    let publisher = ctx.socket(zmq::PUB).expect("Failed to create ZMQ PUB socket");
    publisher.bind("tcp://*:5555").expect("Failed to bind publisher");

    // 2. Wrap the publisher in an Arc<Mutex> so we can share it
    let publisher_arc = Arc::new(Mutex::new(publisher));
    
    // let tx =
    //     "36GpLZ4iAb7QPXdj8h8V5JQCaLQLuZF87NnmE75hn8yp2PgFjnNeNWv851HLjRaTRu7WSUQ5CKVg5K7D3Ekd43bs";
    // let encoded_tx = rpc_client::get_signature(tx).await.unwrap();
    // let td = process_tx(
    //     encoded_tx.transaction,
    //     encoded_tx.slot,
    //     encoded_tx.block_time.unwrap(),
    // )
    // .await.unwrap();

    // println!("Number of trades: {}", td.len());

    // // .unwrap();
    // // let trade = td.first().unwrap();
    // // println!("{:?}", trade);
    // // get_amm_data(&trade.pool_address);
    // return;

    // let block_slot = 281418454;
    // let block = fetch_block_with_version(block_slot).await.unwrap();

    // every 2s check new blocks
    let mut last_processed_slot: Option<u64> = None;

    loop {
        let latest_slot = get_latest_slot().await.expect("Failed to get latest slot");
        // let latest_slot = 312769636;
        // println!("Latest slot: {}", latest_slot);

        let start_slot = match last_processed_slot {
            Some(slot) => slot,
            None => latest_slot,
        };

        if start_slot <= latest_slot {
            for block_num in start_slot..=latest_slot {
                let start_time = Instant::now();
                let block = fetch_block_with_version(block_num).await;
                match block {
                    Ok(_) => {
                        let block = block.unwrap();
                        let publisher_clone = Arc::clone(&publisher_arc);
                        println!("Processing block: {}", block_num);
                        // spawn a new thread to process_block
                        tokio::spawn(async move {
                            process_block(block, publisher_clone).await;
                        });
                        let elapsed = start_time.elapsed();
                        println!("Block {} processed in {:?}", block_num, elapsed);
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
                last_processed_slot = Some(block_num);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
