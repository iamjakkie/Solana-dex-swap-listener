use std::collections::HashMap;

use anyhow::Result;
use borsh::BorshDeserialize;
use solana_sdk::{address_lookup_table::program, bs58};
use solana_transaction_status::{
    EncodedConfirmedBlock, EncodedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction,
    UiParsedInstruction,
};
use spl_token::instruction::TokenInstruction;

use crate::{
    models::{PoolData, TokenBalance, TradeData, UiTokenAmount},
    trade_parser::get_trade_instruction,
    utils::{convert_to_date, get_amount, get_amt, get_mint, get_signer_balance_change},
};

const RAYDIUM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
const METEORA_PROGRAM_ID: &str = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB";
const METEORA_DLMM_PROGRAM_ID: &str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";
const ORCA_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
const SERUM_ADD: &str = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";

pub async fn process_tx(
    trx: EncodedTransactionWithStatusMeta,
    slot: u64,
    timestamp: i64,
) -> Option<Vec<TradeData>> {
    let trx_meta = trx.meta.unwrap();
    if trx_meta.err.is_some() {
        return None;
    }

    let transaction = trx.transaction.clone();
    let ui = match transaction {
        solana_transaction_status::EncodedTransaction::Json(ui_transaction) => ui_transaction,
        _ => return None,
    };

    let signature = ui.signatures[0].clone();

    let msg = match ui.message {
        solana_transaction_status::UiMessage::Raw(raw_msg) => raw_msg,
        _ => return None,
    };

    let accounts = msg.account_keys;

    let mut all_addresses = accounts.clone();

    let loaded_addresses = trx_meta.loaded_addresses.clone();

    if loaded_addresses.is_some() {
        let loaded_addresses = loaded_addresses.unwrap();
        loaded_addresses.writable.iter().for_each(|add| {
            all_addresses.push(add.clone());
        });
        loaded_addresses.readonly.iter().for_each(|add| {
            all_addresses.push(add.clone());
        });
    }

    let pre_balances = trx_meta.pre_balances;
    let post_balances = trx_meta.post_balances;
    let pre_token_balances = trx_meta
        .pre_token_balances
        .expect("Pre token balances not found");
    // convert to Vec<TokenBalance>
    let mut pre_token_balances_vec: Vec<TokenBalance> = vec![];
    for (idx, balance) in pre_token_balances.iter().enumerate() {
        let token_balance = TokenBalance {
            account_index: idx as u32,
            address: all_addresses
                .get(balance.account_index as usize)
                .unwrap_or(&"".to_string())
                .to_string(),
            mint: balance.mint.clone(),
            ui_token_amount: UiTokenAmount {
                ui_amount: balance.ui_token_amount.ui_amount.unwrap_or(0.0),
                decimals: balance.ui_token_amount.decimals as u32,
                amount: balance.ui_token_amount.amount.clone(),
                ui_amount_string: balance.ui_token_amount.ui_amount_string.clone(),
            },
            owner: balance.owner.clone().unwrap(),
            program_id: balance.program_id.clone().unwrap(),
        };
        pre_token_balances_vec.push(token_balance);
    }

    let post_token_balances = trx_meta
        .post_token_balances
        .expect("Post token balances not found");
    let mut post_token_balances_vec: Vec<TokenBalance> = vec![];
    for (idx, balance) in post_token_balances.iter().enumerate() {
        let token_balance = TokenBalance {
            account_index: idx as u32,
            address: all_addresses
                .get(balance.account_index as usize)
                .unwrap_or(&"".to_string())
                .to_string(),
            mint: balance.mint.clone(),
            ui_token_amount: UiTokenAmount {
                ui_amount: balance.ui_token_amount.ui_amount.unwrap_or(0.0),
                decimals: balance.ui_token_amount.decimals as u32,
                amount: balance.ui_token_amount.amount.clone(),
                ui_amount_string: balance.ui_token_amount.ui_amount_string.clone(),
            },
            owner: balance.owner.clone().unwrap(),
            program_id: balance.program_id.clone().unwrap(),
        };
        post_token_balances_vec.push(token_balance);
    }

    let mut trades: Vec<TradeData> = vec![];

    let fee = trx_meta.fee;

    let inners = trx_meta.inner_instructions.clone().unwrap_or(vec![]);

    // iterate over inners
    for inner in inners.iter() {
        for (idx, inner_inst) in inner.instructions.iter().enumerate() {
            if let solana_transaction_status::UiInstruction::Compiled(compiled) = inner_inst {
                let program_data = match bs58::decode(compiled.data.clone()).into_vec() {
                    Ok(data) => data,
                    Err(_) => continue,
                };
                let program_add = all_addresses.get(compiled.program_id_index as usize)?;
                
                match program_add.as_str() {
                    RAYDIUM_PROGRAM_ID => {
                        let (base_add, quote_add) = match compiled.accounts.len() {
                            17 => {
                                let base_add = all_addresses.get(compiled.accounts[4] as usize)?.clone();
                                let quote_add = all_addresses.get(compiled.accounts[5] as usize)?.clone();
                                (base_add, quote_add)
                            },
                            18 => {
                                let base_add = all_addresses.get(compiled.accounts[5] as usize)?.clone();
                                let quote_add = all_addresses.get(compiled.accounts[6] as usize)?.clone();
                                (base_add, quote_add)
                            },
                            _ => {
                                continue;
                            }
                        };
                        
                        if let Some(trade) = build_trade_data(
                            program_add,
                            &program_data,
                            &compiled.accounts,
                            &all_addresses,
                            &pre_token_balances_vec,
                            &post_token_balances_vec,
                            &base_add,
                            &quote_add,
                            // &inners
                            //     .first()
                            //     .expect("Inner instructions not found")
                            //     .instructions,
                            timestamp,
                            slot,
                            &signature,
                            idx,
                            // &inners,
                            &pre_balances,
                            &post_balances,
                            fee,
                        ).await {
                            trades.push(trade);
                        }
                    },
                    ORCA_PROGRAM_ID => {
                        if let Some(trade) = build_trade_data(
                            program_add,
                            &program_data,
                            &compiled.accounts,
                            &all_addresses,
                            &pre_token_balances_vec,
                            &post_token_balances_vec,
                            &"".to_string(),
                            &"".to_string(),
                            // &inners
                            //     .first()
                            //     .expect("Inner instructions not found")
                            //     .instructions,
                            timestamp,
                            slot,
                            &signature,
                            idx,
                            // &inners,
                            &pre_balances,
                            &post_balances,
                            fee,
                        ).await {
                            trades.push(trade);
                        }
                    },
                    METEORA_PROGRAM_ID => {
                        let base_add = all_addresses.get(6)?.clone();
                        let quote_add = all_addresses.get(7)?.clone();
                        if let Some(trade) = build_trade_data(
                            program_add,
                            &program_data,
                            &compiled.accounts,
                            &all_addresses,
                            &pre_token_balances_vec,
                            &post_token_balances_vec,
                            &base_add,
                            &quote_add,
                            // &inners
                            //     .first()
                            //     .expect("Inner instructions not found")
                            //     .instructions,
                            timestamp,
                            slot,
                            &signature,
                            idx,
                            // &inners,
                            &pre_balances,
                            &post_balances,
                            fee,
                        ).await {
                            trades.push(trade);
                        }
                    },
                    METEORA_DLMM_PROGRAM_ID => {
                        if let Some(trade) = build_trade_data(
                            program_add,
                            &program_data,
                            &compiled.accounts,
                            &all_addresses,
                            &pre_token_balances_vec,
                            &post_token_balances_vec,
                            &"".to_string(),
                            &"".to_string(),
                            // &inners
                            //     .first()
                            //     .expect("Inner instructions not found")
                            //     .instructions,
                            timestamp,
                            slot,
                            &signature,
                            idx,
                            // &inners,
                            &pre_balances,
                            &post_balances,
                            fee,
                        ).await {
                            trades.push(trade);
                        }
                    }
                    // if not found - check inner instructions
                    _ => {                        
                    }
                }
            }
        }
    }
    
    for (idx, inst) in msg.instructions.into_iter().enumerate() {

        let trx_meta_inner = trx_meta.inner_instructions.clone().unwrap_or(vec![]);

        let decoded_data = bs58::decode(inst.data.clone()).into_vec().unwrap();
        
        let main_program = all_addresses.get(inst.program_id_index as usize).unwrap();
        match main_program.as_str() {
            RAYDIUM_PROGRAM_ID => {
                // standard raydium - srmq add
                if let Some(pos) = inst
                    .accounts
                    .iter()
                    .position(|&ix| all_addresses[ix as usize] == SERUM_ADD)
                {
                    // no extra checks, just do pos-2 and pos-1 as before
                    let base_add = all_addresses
                        .get(inst.accounts[pos - 2] as usize)
                        .expect("Base account not found")
                        .clone();

                    let quote_add = all_addresses
                        .get(inst.accounts[pos - 1] as usize)
                        .expect("Quote account not found")
                        .clone();

                    if let Some(trade) = build_trade_data(
                        main_program,
                        &decoded_data,
                        &inst.accounts,
                        &all_addresses,
                        &pre_token_balances_vec,
                        &post_token_balances_vec,
                        &base_add,
                        &quote_add,
                        // &inners
                        //     .first()
                        //     .expect("Inner instructions not found")
                        //     .instructions,
                        timestamp,
                        slot,
                        &signature,
                        idx,
                        // &inners,
                        &pre_balances,
                        &post_balances,
                        fee,
                    ).await {
                        trades.push(trade);
                    }
                }
            }
            ORCA_PROGRAM_ID => {
                if let Some(trade) = build_trade_data(
                    main_program,
                    &decoded_data,
                    &inst.accounts,
                    &all_addresses,
                    &pre_token_balances_vec,
                    &post_token_balances_vec,
                    &"".to_string(),
                    &"".to_string(),
                    // &inners
                    //     .first()
                    //     .expect("Inner instructions not found")
                    //     .instructions,
                    timestamp,
                    slot,
                    &signature,
                    idx,
                    // &inners,
                    &pre_balances,
                    &post_balances,
                    fee,
                ).await {
                    trades.push(trade);
                }
            },
            METEORA_PROGRAM_ID => {
                let base_add = all_addresses.get(6)?.clone();
                let quote_add = all_addresses.get(7)?.clone();
                if let Some(trade) = build_trade_data(
                    main_program,
                    &decoded_data,
                    &inst.accounts,
                    &all_addresses,
                    &pre_token_balances_vec,
                    &post_token_balances_vec,
                    &base_add,
                    &quote_add,
                    timestamp,
                    slot,
                    &signature,
                    idx,
                    &pre_balances,
                    &post_balances,
                    fee,
                ).await {
                    trades.push(trade);
                }
            },
            METEORA_DLMM_PROGRAM_ID => {
                if let Some(trade) = build_trade_data(
                    main_program,
                    &decoded_data,
                    &inst.accounts,
                    &all_addresses,
                    &pre_token_balances_vec,
                    &post_token_balances_vec,
                    &"".to_string(),
                    &"".to_string(),
                    timestamp,
                    slot,
                    &signature,
                    idx,
                    &pre_balances,
                    &post_balances,
                    fee,
                ).await {
                    trades.push(trade);
                }
            }
            // if not found - check inner instructions
            _ => {
                // something's fucked up here - inner is repeated 4 times (clone)
                
            }
        };
    }
    Some(trades)
}

async fn build_trade_data(
    program: &String,
    decoded_data: &Vec<u8>,
    inst_accounts: &Vec<u8>,
    accounts: &Vec<String>,
    pre_token_balances_vec: &Vec<TokenBalance>,
    post_token_balances_vec: &Vec<TokenBalance>,
    base_add: &String,
    quote_add: &String,
    timestamp: i64,
    slot: u64,
    signature: &String,
    idx: usize,
    pre_balances: &Vec<u64>,
    post_balances: &Vec<u64>,
    fee: u64,
) -> Option<TradeData> {
    let trade_data = get_trade_instruction(
        program,
        decoded_data,
        inst_accounts,
        accounts,
        pre_token_balances_vec,
        post_token_balances_vec,
        &"".to_string(),
        false,
        0,
        base_add,
        quote_add,
    );
    // 2. If there's a return, build the TradeData struct
    if let Some(td) = trade_data {
        let td_name = td.name;
        let td_address = td.dapp_address;

        let trade = TradeData {
            block_date: convert_to_date(timestamp).await,
            tx_id: bs58::encode(signature).into_string(),
            block_slot: slot,
            block_time: timestamp,
            signature: signature.to_string(),
            signer: accounts.get(0).unwrap().to_string(),
            pool_address: td.amm,
            base_mint: get_mint(&td.vault_a, post_token_balances_vec).await.ok_or(format!("Base mint not found for signature {}, vault: {}", signature, td.vault_a)).unwrap(),
            quote_mint: get_mint(&td.vault_b, post_token_balances_vec).await.ok_or(format!("Quote mint not found for signature {}, vault: {}", signature, td.vault_b)).unwrap(),
            base_amount: get_amount(&td.vault_a, pre_token_balances_vec, post_token_balances_vec).await,
            quote_amount: get_amount(&td.vault_b, pre_token_balances_vec, post_token_balances_vec).await,
            base_vault: td.vault_a,
            quote_vault: td.vault_b,
            is_inner_instruction: false,
            instruction_index: idx as u32,
            instruction_type: td_name.clone(),
            inner_instruction_index: 0,
            outer_program: td_address.clone(),
            inner_program: "".to_string(),
            txn_fee_lamports: fee,
            signer_lamports_change: get_signer_balance_change(pre_balances, post_balances).await,
        };

        Some(trade)
    } else {
        None
    }
}
