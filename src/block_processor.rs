use std::collections::{HashMap, HashSet};
use std::os::unix::process;

use crate::trade_parser::get_trade_instruction;
use crate::{
    models::{TokenBalance, TradeData, UiTokenAmount},
    tx_processor::process_tx,
    utils::{convert_to_date, get_amt, get_mint, get_signer_balance_change, save_to_csv},
};
use solana_sdk::{bs58, commitment_config::CommitmentConfig};
use solana_transaction_status::{EncodedConfirmedBlock, UiInnerInstructions};

pub async fn process_block(block: EncodedConfirmedBlock) {
    let timestamp = block.block_time.expect("Block time not found");
    let slot = block.parent_slot;
    let mut data: Vec<TradeData> = vec![];

    // println!("Transactions: {}", block.transactions.len());

    for trx in block.transactions {
        match process_tx(trx, slot, timestamp).await {
            Some(trades) => {
                data.extend(trades);
            }
            None => {}
        }
    }

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



    save_to_csv(data);
    // 43
    //
}
