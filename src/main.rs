mod rpc_client;
mod block_processor;
mod tx_processor;
mod trade_parser;
mod utils;
mod models;
mod global;

use rpc_client::fetch_block_with_version;
use block_processor::process_block;

#[tokio::main]
async fn main() {
    let block_slot = 281418454;
    let block = fetch_block_with_version(block_slot).await.unwrap();
    process_block(block).await;
}
