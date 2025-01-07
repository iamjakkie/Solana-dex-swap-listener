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

    for trx in block.transactions {
        match process_tx(trx, slot, timestamp).await {
            Some(trades) => {
                data.extend(trades);
            }
            None => {}
        }
    }

    save_to_csv(data);
    // 43
    //
}
