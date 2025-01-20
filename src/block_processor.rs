

use std::{sync::{Arc, Mutex}, thread::current, time::{SystemTime, UNIX_EPOCH}};

use crate::{
    models::{TokenBalance, TradeData, UiTokenAmount},
    tx_processor::process_tx,
    utils::{convert_to_date, get_amt, get_mint, get_signer_balance_change, save_to_csv},
};
use std::time::Duration;
use chrono::{DateTime, Utc};
use solana_transaction_status::{EncodedConfirmedBlock, UiInnerInstructions};

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

    println!("Block time: {:?}, processed at: {:?}", timestamp_str, current_timestamp_str);

    // println!("Length of data: {}", data.len());

    // data.iter().for_each(|trade| {
    //     println!("Signature: {}", trade.signature);
    // });

    // print distinct signatures
    // let mut signature_counts = HashMap::new();
    // let mut ordered_signatures = Vec::new();

    // for trade in data.iter() {
    //     let count = signature_counts.entry(trade.signature.clone()).or_insert(0);
    //     *count += 1;

    //     // If this is the first time we're seeing this signature, add it to the ordered list
    //     if *count == 1 {
    //         ordered_signatures.push(trade.signature.clone());
    //     }
    // }

    // ordered_signatures.iter().for_each(|signature| {
    //     println!("{:?}: {:?}", signature, signature_counts.get(signature).unwrap());
    // });

    // println!("Number of distinct signatures: {}", ordered_signatures.len());



    // save_to_csv(data);
    // 43
    //

    let json_str = serde_json::to_string(&data).unwrap();
    let sock = publisher_clone.lock().unwrap();
    sock.send("", zmq::SNDMORE).unwrap(); // optional topic
    sock.send(&json_str, 0).unwrap();
}
