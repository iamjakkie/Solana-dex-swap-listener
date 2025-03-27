use anyhow::Result;
use std::{
    collections::HashMap, sync::{Arc, Mutex}, thread::current, time::{SystemTime, UNIX_EPOCH}
};

use crate::{
    global::OUTPUT_PATH,
    models::{TokenBalance, TradeData, UiTokenAmount, ZmqData},
    tx_processor::process_tx,
    utils::{
        convert_to_date, get_amt, get_mint, get_signer_balance_change, save_trades_to_avro,
        save_trades_to_csv,
    },
};
use chrono::{DateTime, Utc};
use solana_transaction_status::{EncodedConfirmedBlock, UiInnerInstructions};
use std::time::Duration;

pub async fn process_block(
    slot: u64, // node returns wrong slot
    block: EncodedConfirmedBlock,
    publisher_clone: Option<Arc<Mutex<zmq::Socket>>>,
) -> Result<()> {
    let timestamp = block.block_time.expect("Block time not found");
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

    save_trades_to_avro(&data, &date_str, slot)
        .await?;

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

    Ok(())
}
