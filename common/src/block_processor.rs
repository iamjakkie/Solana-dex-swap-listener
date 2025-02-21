use std::{
    sync::{Arc, Mutex},
    thread::current,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    global::OUTPUT_PATH,
    models::{TokenBalance, TradeData, UiTokenAmount, ZmqData},
    tx_processor::process_tx,
    utils::{convert_to_date, get_amt, get_mint, get_signer_balance_change, save_trades_to_avro, save_trades_to_csv},
};
use chrono::{DateTime, Utc};
use solana_transaction_status::{EncodedConfirmedBlock, UiInnerInstructions};
use std::time::Duration;

pub async fn process_block(block: EncodedConfirmedBlock, publisher_clone: Arc<Mutex<zmq::Socket>>) {
    let timestamp = block.block_time.expect("Block time not found");
    let slot = block.parent_slot;
    let mut data: Vec<TradeData> = vec![];

    // convert timestamp to human readable timestamp
    let d = UNIX_EPOCH + Duration::from_secs(timestamp.try_into().unwrap());
    // Create DateTime from SystemTime
    let datetime = DateTime::<Utc>::from(d);
    // Formats the combined date and time with the specified format string.
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();

    let date_str = datetime.format("%Y-%m-%d").to_string();

    for trx in block.transactions {
        match process_tx(trx, slot, timestamp).await {
            Some(trades) => {
                data.extend(trades);
            }
            None => {}
        }
    }

    let current_time = SystemTime::now();
    let current_datetime = DateTime::<Utc>::from(current_time);
    let current_timestamp_str = current_datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();

    println!(
        "Block time: {:?}, processed at: {:?}",
        timestamp_str, current_timestamp_str
    );

    let file_path = format!("{}/{}/{}.avro", OUTPUT_PATH.as_str(), date_str, slot);
    println!("Saving trades to: {}", file_path);

    // save_trades_to_csv(&data, file_path.as_str()).await.expect("Failed to save trades to csv");
    save_trades_to_avro(&data, file_path.as_str())
        .await
        .expect("Failed to save trades to avro");

    // TODO: ZMQ
    // let zmq_data: ZmqData = ZmqData {
    //     slot: slot,
    //     date: date_str,
    //     data: data,
    // };

    // let json_str = serde_json::to_string(&zmq_data).unwrap();
    // let sock = publisher_clone.lock().unwrap();
    // sock.send("", zmq::SNDMORE).unwrap(); // optional topic
    // sock.send(&json_str, 0).unwrap();
}
