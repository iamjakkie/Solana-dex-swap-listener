mod block_processor;
mod global;
mod models;
mod rpc_client;
mod trade_parser;
mod tx_processor;
mod utils;

use std::time::Instant;

use block_processor::process_block;
use rpc_client::{fetch_block_with_version, get_latest_slot};
use tx_processor::process_tx;
use utils::get_amm_data;

#[tokio::main]
async fn main() {
    let tx =
        "3GFsViNSsRkVGVky7GYYq9VT5XnHJDsSsa1jUtjEs8jYpuF327rB6FU9e4hzto5pG5SuT9essSDDbBnDPd8Wtq66";
    let encoded_tx = rpc_client::get_signature(tx).await.unwrap();
    let td = process_tx(
        encoded_tx.transaction,
        encoded_tx.slot,
        encoded_tx.block_time.unwrap(),
    )
    .await
    .unwrap();
    let trade = td.first().unwrap();
    get_amm_data(&trade.pool_address);
    return;

    // let block_slot = 281418454;
    // let block = fetch_block_with_version(block_slot).await.unwrap();

    // every 2s check new blocks
    let mut last_processed_slot: Option<u64> = None;

    loop {
        // let latest_slot = get_latest_slot().await.expect("Failed to get latest slot");
        let latest_slot = 312300763;
        // println!("Latest slot: {}", latest_slot);

        let start_slot = match last_processed_slot {
            Some(slot) => slot - 1,
            None => latest_slot,
        };

        if start_slot <= latest_slot {
            for block_num in start_slot..=latest_slot {
                let start_time = Instant::now();
                let block = fetch_block_with_version(block_num).await;
                while block.is_err() {
                    println!("Failed to fetch block: {}", block_num);
                    let block_err = block.as_ref().err();
                    println!("{:?}", block_err);
                    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                    let block = fetch_block_with_version(block_num).await;
                }
                let block = block.unwrap();
                println!("Processing block: {}", block_num);
                // spawn a new thread to process_block
                tokio::spawn(async move {
                    process_block(block).await;
                });
                let elapsed = start_time.elapsed();
                println!("Block {} processed in {:?}", block_num, elapsed);
                last_processed_slot = Some(block_num);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
