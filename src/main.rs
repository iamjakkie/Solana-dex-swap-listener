use chrono::{DateTime, NaiveDateTime, Utc};
use solana_sdk::inner_instruction;
use solana_sdk::{bs58, commitment_config::CommitmentConfig};
use solana_transaction_status::{EncodedConfirmedBlock, UiInnerInstructions, UiInstruction};
use solana_client::{
    rpc_client::RpcClient,
    rpc_request::RpcRequest,
};
use solana_sdk::pubkey::Pubkey;

use tokio::time::{Interval, Duration};
use borsh::{BorshDeserialize, BorshSerialize};
use core::time;
use std::{clone, env};
use std::str::FromStr;
use std::sync::Arc;
use serde_json::{json, Value};
use anyhow::Result;

const RAYDIUM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";


#[derive(Debug)]
pub struct TradeInstruction {
    pub dapp_address: String,
    pub name: String,
    pub amm: String,
    pub vault_a: String,
    pub vault_b: String,
}

impl Default for TradeInstruction {
    fn default() -> Self {
        TradeInstruction {
            dapp_address: "".to_string(),
            name: "".to_string(),
            amm: "".to_string(),
            vault_a: "".to_string(),
            vault_b: "".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct UiTokenAmount {
    pub ui_amount: f64,
    pub decimals: u32,
    pub amount: String,
    pub ui_amount_string: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TokenBalance {
    pub account_index: u32,
    pub mint: String,
    pub ui_token_amount: UiTokenAmount,
    pub owner: String,
    pub program_id: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct InnerInstruction {
    pub program_id_index: u32,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
    pub stack_height: Option<u32>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct InnerInstructions {
    pub index: u32,
    pub instructions: Vec<InnerInstruction>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TradeData {
    pub block_date: String,
    pub block_time: i64,
    pub block_slot: u64,
    pub tx_id: String,
    pub signer: String,
    pub pool_address: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub base_vault: String,
    pub quote_vault: String,
    pub base_amount: f64,
    pub quote_amount: f64,
    pub is_inner_instruction: bool,
    pub instruction_index: u32,
    pub instruction_type: String,
    pub inner_instruction_index: u32,
    pub outer_program: String,
    pub inner_program: String,
    pub txn_fee_lamports: u64,
    pub signer_lamports_change: i64,
}

#[derive(Clone, PartialEq)]
pub struct Output {
    pub data: Vec<TradeData>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct Transfer {
    pub amount: u64,
}


pub fn parse_trade_instruction(
    bytes_stream: &Vec<u8>,
    input_accounts: Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> Option<TradeInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(1);
    let discriminator: u8 = u8::from(disc_bytes[0]);

    let mut result = None;

    println!("Discriminator: {:?}", discriminator);

    match discriminator {
        9 => {
            result = Some(TradeInstruction {
                dapp_address: String::from("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                name: String::from("SwapBaseIn"),
                amm: input_accounts.get(1).unwrap().to_string(),
                vault_a: get_vault_a(&input_accounts, post_token_balances, accounts),
                vault_b: get_vault_b(&input_accounts, post_token_balances, accounts),
                ..Default::default()
            });
        }
        11 => {
            result = Some(TradeInstruction {
                dapp_address: String::from("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                name: String::from("SwapBaseOut"),
                amm: input_accounts.get(1).unwrap().to_string(),
                vault_a: get_vault_a(&input_accounts, post_token_balances, accounts),
                vault_b: get_vault_b(&input_accounts, post_token_balances, accounts),
                ..Default::default()
            });
        }
        _ => {}
    }

    return result;
}

pub fn get_mint(
    address: &String,
    token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
    dapp_address: String,
) -> String {
    if dapp_address.eq("MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG")
        || dapp_address.eq("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P")
    {
        return "So11111111111111111111111111111111111111112".to_string();
    }
    println!("Accounts: {:?}", accounts);

    let index = accounts.iter().position(|r| r == address).unwrap();
    println!("Index: {:?}", index);
    let mut result: String = String::new();

    println!("Token Balances: {:?}", token_balances);
    token_balances
        .iter()
        .filter(|token_balance| token_balance.account_index == index as u32)
        .for_each(|token_balance| {
            result = token_balance.mint.clone();
        });
    return result;
}

fn get_vault_a(
    input_accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> String {
    let mut vault_a = input_accounts.get(4).unwrap().to_string();
    let mint_a = get_mint(&vault_a, post_token_balances, accounts, "".to_string());

    if mint_a.is_empty() {
        vault_a = input_accounts.get(5).unwrap().to_string();
    }

    return vault_a;
}

fn get_vault_b(
    input_accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> String {
    let mut vault_a_index = 4;

    let mut vault_a = input_accounts.get(4).unwrap().to_string();
    let mint_a = get_mint(&vault_a, post_token_balances, accounts, "".to_string());

    if mint_a.is_empty() {
        vault_a_index += 1;
        vault_a = input_accounts.get(vault_a_index).unwrap().to_string();
    }

    let mut vault_b_index = vault_a_index + 1;
    let mut vault_b = input_accounts.get(vault_b_index).unwrap().to_string();

    if vault_a == vault_b {
        vault_b_index += 1;
        vault_b = input_accounts.get(vault_b_index).unwrap().to_string();
    }

    return vault_b;
}

async fn fetch_block_with_version(rpc_client: &RpcClient, block_slot: u64) -> Result<EncodedConfirmedBlock> {
    let params = json!([
        block_slot,
        {
            "maxSupportedTransactionVersion": 0
        }
    ]);

    let raw_response: Value = rpc_client.send(RpcRequest::GetBlock, params)?;

    let block: EncodedConfirmedBlock = serde_json::from_value(raw_response)?;

    Ok(block)
}

fn process_block(block: EncodedConfirmedBlock) {
    let timestamp = block.block_time.expect("Block time not found");
    let slot = block.parent_slot;
    let mut data: Vec<TradeData> = vec![];
    for trx in block.transactions {
        let trx_meta = trx.meta.unwrap();
        if trx_meta.err.is_some() {
            continue;
        }

        let transaction = trx.transaction.clone();
        let ui = match transaction {
            solana_transaction_status::EncodedTransaction::Json(ui_transaction) => ui_transaction,
            _ => continue,
        };

        let signature = ui.signatures[0].clone();

        let msg = match ui.message {
            solana_transaction_status::UiMessage::Raw(raw_msg) => raw_msg,
            _ => continue,
        };

        let accounts = msg.account_keys;


        if !accounts.contains(&RAYDIUM_PROGRAM_ID.to_string()) {
            continue;
        }

        let pre_balances = trx_meta.pre_balances;
        let post_balances = trx_meta.post_balances;
        let pre_token_balances = trx_meta.pre_token_balances.expect("Pre token balances not found");
        // convert to Vec<TokenBalance>
        let mut pre_token_balances_vec: Vec<TokenBalance> = vec![];
        for (idx, balance) in pre_token_balances.iter().enumerate() {
            let token_balance = TokenBalance {
                account_index: idx as u32,
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

        let post_token_balances = trx_meta.post_token_balances.expect("Post token balances not found");
        let mut post_token_balances_vec: Vec<TokenBalance> = vec![];
        for (idx, balance) in post_token_balances.iter().enumerate() {
            let token_balance = TokenBalance {
                account_index: idx as u32,
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

        for (idx, inst) in msg.instructions.into_iter().enumerate() {
            
            let trx_meta_inner = trx_meta.inner_instructions.clone().expect("Inner instructions not found");
            let first_instruction = trx_meta_inner.first().expect("First instruction not found");
            let inner_instructions = first_instruction.clone().instructions;

            // println!("Instruction: {:?}", inst);
            // println!("Inner Instructions: {:?}", trx_meta_inner);
            // println!("First Instruction: {:?}", first_instruction);

            // let mut instructions = Vec::<InnerInstruction>::new();
            // let first_instruction_index = first_instruction.index as u32;

            // for inner_inst in first_instruction.instructions.iter() {
            //     match inner_inst {
            //         UiInstruction::Parsed(_) => continue,
            //         UiInstruction::Compiled(compiled) => {
            //             let program_id_index = compiled.program_id_index as u32;
            //             let accounts = compiled.accounts.clone();
            //             let data = compiled.data.clone().into_bytes();
            //             let stack_height = compiled.stack_height;
            //             let inner_instruction = InnerInstruction {
            //                 program_id_index,
            //                 accounts,
            //                 data,
            //                 stack_height,
            //             };
            //             instructions.push(inner_instruction);
            //         },
            //     };
            // let inner_instructions = InnerInstructions {
            //     index: first_instruction_index,
            //     instructions,
            // };
            // println!("Inner Instructions: {:?}", inner_instructions);
                // let inner_program = &accounts[inner_inst as usize];
                // let inner_accounts = inner_inst.accounts.clone();
                // let inner_data = inner_inst.data.clone();
                // let inner_stack_height = inner_inst.stack_height.clone();

                // let inner_instruction = InnerInstruction {
                //     program_id_index: inner_inst.program_id_index,
                //     accounts: inner_accounts,
                //     data: inner_data,
                //     stack_height: inner_stack_height,
                // };

                // let inner_instructions = InnerInstructions {
                //     index: first_instruction.index,
                //     instructions: vec![inner_instruction],
                // };

                // inner_instruction_vec.push(inner_instructions);
            // }

            let program = &accounts[inst.program_id_index as usize];
            
            if program != RAYDIUM_PROGRAM_ID {
                continue;
            }

            // decode data using base58
            let decoded_data = bs58::decode(inst.data.clone()).into_vec().unwrap();

            // print signature
            println!("Signature: {:?}", ui.signatures[0]);

            let trade_data = get_trade_instruction(
                program,
                &decoded_data,
                &inst.accounts,
                &accounts,
                &pre_token_balances_vec,
                &post_token_balances_vec,
                &"".to_string(),
                false,
                &inner_instructions,
                0 as u32
            );

            if trade_data.is_some() {
                let td = trade_data.unwrap();
                println!("Trade Data: {:?}", td);

                let td_name = td.name;
                let td_address = td.dapp_address;

                data.push(TradeData {
                    block_date: convert_to_date(timestamp),
                    tx_id: bs58::encode(&ui.signatures[0]).into_string(),
                    block_slot: block.parent_slot,
                    block_time: timestamp,
                    signer: accounts.get(0).unwrap().to_string(),
                    pool_address: td.amm,
                    base_mint: get_mint(
                        &td.vault_a,
                        &post_token_balances_vec,
                        &accounts,
                        td_address.clone(),
                    ),
                    quote_mint: get_mint(
                        &td.vault_b,
                        &post_token_balances_vec,
                        &accounts,
                        "".to_string(),
                    ),
                    base_amount: get_amt(
                        &td.vault_a,
                        0 as u32,
                        &trx_meta_inner,
                        &accounts,
                        &post_token_balances_vec,
                        td_address.clone(),
                        pre_balances.clone(),
                        post_balances.clone(),
                    ),
                    quote_amount: get_amt(
                        &td.vault_b,
                        0 as u32,
                        &trx_meta_inner,
                        &accounts,
                        &post_token_balances_vec,
                        "".to_string(),
                        pre_balances.clone(),
                        post_balances.clone(),
                    ),
                    base_vault: td.vault_a,
                    quote_vault: td.vault_b,
                    is_inner_instruction: false,
                    instruction_index: idx as u32,
                    instruction_type: td_name.clone(),
                    inner_instruction_index: 0,
                    outer_program: td_address.clone(),
                    inner_program: "".to_string(),
                    txn_fee_lamports: trx_meta.fee,
                    signer_lamports_change: get_signer_balance_change(
                        &pre_balances,
                        &post_balances,
                    ),
                });

                // if td.second_swap_amm.clone().unwrap_or_default() != "" {
                //     data.push(TradeData {
                //         block_date: convert_to_date(timestamp),
                //         tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                //         block_slot: slot,
                //         block_time: timestamp,
                //         signer: accounts.get(0).unwrap().to_string(),
                //         pool_address: td.second_swap_amm.clone().unwrap(),
                //         base_mint: get_mint(
                //             &td.second_swap_vault_a.clone().unwrap(),
                //             &post_token_balances,
                //             &accounts,
                //             "".to_string(),
                //         ),
                //         quote_mint: get_mint(
                //             &td.second_swap_vault_b.clone().unwrap(),
                //             &post_token_balances,
                //             &accounts,
                //             "".to_string(),
                //         ),
                //         base_amount: get_amt(
                //             &td.second_swap_vault_a.clone().unwrap(),
                //             0 as u32,
                //             &inner_instructions,
                //             &accounts,
                //             &post_token_balances,
                //             "".to_string(),
                //             pre_balances.clone(),
                //             post_balances.clone(),
                //         ),
                //         quote_amount: get_amt(
                //             &td.second_swap_vault_b.clone().unwrap(),
                //             0 as u32,
                //             &inner_instructions,
                //             &accounts,
                //             &post_token_balances,
                //             "".to_string(),
                //             pre_balances.clone(),
                //             post_balances.clone(),
                //         ),
                //         base_vault: td.second_swap_vault_a.clone().unwrap(),
                //         quote_vault: td.second_swap_vault_b.clone().unwrap(),
                //         is_inner_instruction: false,
                //         instruction_index: idx as u32,
                //         instruction_type: td_name.clone(),
                //         inner_instruction_index: 0,
                //         outer_program: td_dapp_address.clone(),
                //         inner_program: "".to_string(),
                //         txn_fee_lamports: meta.fee,
                //         signer_lamports_change: get_signer_balance_change(
                //             &pre_balances,
                //             &post_balances,
                //         ),
                //     });
                // }
            }


            trx_meta.inner_instructions.clone()
                .expect("Inner instructions not found")
                .iter()
                .filter(|inner_instruction| inner_instruction.index == idx as u8)
                .for_each(|inner_instruction| {
                    inner_instruction.instructions.iter().enumerate().for_each(
                        |(inner_idx, inner_inst)| {
                            let inner_program =
                                &accounts[inst.program_id_index as usize];
                            let inner_trade_data = get_trade_instruction(
                                inner_program,
                                &decoded_data,
                                &inst.accounts,
                                &accounts,
                                &pre_token_balances_vec,
                                &post_token_balances_vec,
                                &program.to_string(),
                                true,
                                &inner_instructions,
                                inner_idx as u32,
                            );

                            if inner_trade_data.is_some() {
                                let inner_td = inner_trade_data.unwrap();

                                let inner_td_name = inner_td.name;
                                let inner_td_dapp_address = inner_td.dapp_address;

                                

                                data.push(TradeData {
                                    block_date: convert_to_date(timestamp),
                                    tx_id: bs58::encode(&signature)
                                        .into_string(),
                                    block_slot: slot,
                                    block_time: timestamp,
                                    signer: accounts.get(0).unwrap().to_string(),
                                    pool_address: inner_td.amm,
                                    base_mint: get_mint(
                                        &inner_td.vault_a,
                                        &post_token_balances_vec,
                                        &accounts,
                                        inner_td_dapp_address.clone(),
                                    ),
                                    quote_mint: get_mint(
                                        &inner_td.vault_b,
                                        &post_token_balances_vec,
                                        &accounts,
                                        "".to_string(),
                                    ),
                                    base_amount: get_amt(
                                        &inner_td.vault_a,
                                        inner_idx as u32,
                                        &trx_meta_inner,
                                        &accounts,
                                        &post_token_balances_vec,
                                        inner_td_dapp_address.clone(),
                                        pre_balances.clone(),
                                        post_balances.clone(),
                                    ),
                                    quote_amount: get_amt(
                                        &inner_td.vault_b,
                                        inner_idx as u32,
                                        &trx_meta_inner,
                                        &accounts,
                                        &post_token_balances_vec,
                                        "".to_string(),
                                        pre_balances.clone(),
                                        post_balances.clone(),
                                    ),
                                    base_vault: inner_td.vault_a,
                                    quote_vault: inner_td.vault_b,
                                    is_inner_instruction: true,
                                    instruction_index: idx as u32,
                                    instruction_type: inner_td_name.clone(),
                                    inner_instruction_index: inner_idx as u32,
                                    outer_program: program.to_string(),
                                    inner_program: inner_td_dapp_address.clone(),
                                    txn_fee_lamports: trx_meta.fee,
                                    signer_lamports_change: get_signer_balance_change(
                                        &pre_balances,
                                        &post_balances,
                                    ),
                                });

                                // if inner_td.second_swap_amm.clone().unwrap_or_default()
                                //     != ""
                                // {
                                //     data.push(TradeData {
                                //         block_date: convert_to_date(timestamp),
                                //         tx_id: bs58::encode(&transaction.signatures[0])
                                //             .into_string(),
                                //         block_slot: slot,
                                //         block_time: timestamp,
                                //         signer: accounts.get(0).unwrap().to_string(),
                                //         pool_address: inner_td
                                //             .second_swap_amm
                                //             .clone()
                                //             .unwrap(),
                                //         base_mint: get_mint(
                                //             &inner_td.second_swap_vault_a.clone().unwrap(),
                                //             &post_token_balances,
                                //             &accounts,
                                //             "".to_string(),
                                //         ),
                                //         quote_mint: get_mint(
                                //             &inner_td.second_swap_vault_b.clone().unwrap(),
                                //             &post_token_balances,
                                //             &accounts,
                                //             "".to_string(),
                                //         ),
                                //         base_amount: get_amt(
                                //             &inner_td.second_swap_vault_a.clone().unwrap(),
                                //             inner_idx as u32,
                                //             &inner_instructions,
                                //             &accounts,
                                //             &post_token_balances,
                                //             "".to_string(),
                                //             pre_balances.clone(),
                                //             post_balances.clone(),
                                //         ),
                                //         quote_amount: get_amt(
                                //             &inner_td.second_swap_vault_b.clone().unwrap(),
                                //             inner_idx as u32,
                                //             &inner_instructions,
                                //             &accounts,
                                //             &post_token_balances,
                                //             "".to_string(),
                                //             pre_balances.clone(),
                                //             post_balances.clone(),
                                //         ),
                                //         base_vault: inner_td
                                //             .second_swap_vault_a
                                //             .clone()
                                //             .unwrap(),
                                //         quote_vault: inner_td
                                //             .second_swap_vault_b
                                //             .clone()
                                //             .unwrap(),
                                //         is_inner_instruction: true,
                                //         instruction_index: idx as u32,
                                //         instruction_type: inner_td_name.clone(),
                                //         inner_instruction_index: inner_idx as u32,
                                //         outer_program: program.to_string(),
                                //         inner_program: inner_td_dapp_address.clone(),
                                //         txn_fee_lamports: meta.fee,
                                //         signer_lamports_change: get_signer_balance_change(
                                //             &pre_balances,
                                //             &post_balances,
                                //         ),
                                //     });
                                // }
                            }
                        },
                    )
                });

            // println!("{:?}", trade_data);
            
        
            println!("{:?}", data);
            break;
        }
    }
}

fn get_signer_balance_change(pre_balances: &Vec<u64>, post_balances: &Vec<u64>) -> i64 {
    return post_balances[0] as i64 - pre_balances[0] as i64;
}

pub fn convert_to_date(ts: i64) -> String {
    let nt = NaiveDateTime::from_timestamp_opt(ts, 0);
    let dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(nt.unwrap(), Utc);
    let res = dt.format("%Y-%m-%d");
    return res.to_string();
}

pub fn get_amt(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<UiInnerInstructions>,
    accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    dapp_address: String,
    pre_balances: Vec<u64>,
    post_balances: Vec<u64>,
) -> f64 {
    let mut result: f64 = 0.0;

    let source_transfer_amt = get_token_transfer(
        address,
        input_inner_idx,
        inner_instructions,
        accounts,
        "source".to_string(),
        dapp_address.clone(),
        pre_balances.clone(),
        post_balances.clone(),
    );

    let destination_transfer_amt = get_token_transfer(
        address,
        input_inner_idx,
        inner_instructions,
        accounts,
        "destination".to_string(),
        dapp_address.clone(),
        pre_balances.clone(),
        post_balances.clone(),
    );

    if source_transfer_amt != 0.0 {
        result = source_transfer_amt;
    } else if destination_transfer_amt != 0.0 {
        result = destination_transfer_amt;
    }

    if result != 0.0 {
        let index = accounts.iter().position(|r| r == address).unwrap();
        post_token_balances
            .iter()
            .filter(|token_balance| token_balance.account_index == index as u32)
            .for_each(|token_balance: &TokenBalance| {
                let decimals = token_balance.ui_token_amount.clone().decimals;
                result = result / (u64::pow(10, decimals)) as f64;
            });
    }

    result
}

pub fn get_token_transfer(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<UiInnerInstructions>,
    accounts: &Vec<String>,
    account_name_to_check: String,
    dapp_address: String,
    pre_balances: Vec<u64>,
    post_balances: Vec<u64>,
) -> f64 {
    if dapp_address.eq("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P") {
        return get_system_program_transfer(
            address,
            input_inner_idx,
            inner_instructions,
            accounts,
            account_name_to_check,
            pre_balances,
            post_balances,
        );
    }

    let mut result = 0.0;
    let mut result_assigned = false;

    inner_instructions.iter().for_each(|inner_instruction| {
        inner_instruction
            .instructions
            .iter()
            .enumerate()
            .for_each(|(inner_idx, inner_inst)| {
                let inner_inst = match inner_inst {
                    UiInstruction::Parsed(_) => return,
                    UiInstruction::Compiled(compiled) => compiled,
                };
                let inner_program = &accounts[inner_inst.program_id_index as usize];
                if inner_program
                    .as_str()
                    .eq("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                {
                    println!("Inner Program: {:?}", inner_program);
                    // println!("Data: {:?}", inner_inst.data.clone().into_bytes());
                    let data = bs58::decode(inner_inst.data.clone()).into_vec().expect("Error decoding data");
                    let (discriminator_bytes, rest) = data.split_at(1);
                    let discriminator: u8 = u8::from(discriminator_bytes[0]);

                    match discriminator {
                        3 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(1).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = -1.0 * data.amount as f64;
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = data.amount as f64;
                                    result_assigned = true;
                                }
                            }
                        }
                        12 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(2).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = -1.0 * data.amount as f64;
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = data.amount as f64;
                                    result_assigned = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
    });

    if !result_assigned {
        let _result = get_token_22_transfer(
            address,
            input_inner_idx,
            inner_instructions,
            accounts,
            account_name_to_check,
        );
        if _result.is_some() {
            result = _result.unwrap();
        }
    }

    result
}

pub fn get_token_22_transfer(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<UiInnerInstructions>,
    accounts: &Vec<String>,
    account_name_to_check: String,
) -> Option<f64> {
    let mut result = None;
    let mut result_assigned = false;

    inner_instructions.iter().for_each(|inner_instruction| {
        inner_instruction
            .instructions
            .iter()
            .enumerate()
            .for_each(|(inner_idx, inner_inst)| {
                let inner_inst = match inner_inst {
                    UiInstruction::Parsed(_) => return,
                    UiInstruction::Compiled(compiled) => compiled,
                };
                let inner_program = &accounts[inner_inst.program_id_index as usize];

                if inner_program
                    .as_str()
                    .eq("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
                {
                    let data = bs58::decode(inner_inst.data.clone()).into_vec().expect("Error decoding data");                    let (discriminator_bytes, rest) = data.split_at(1);
                    let discriminator: u8 = u8::from(discriminator_bytes[0]);

                    match discriminator {
                        3 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(1).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(-1.0 * data.amount as f64);
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(data.amount as f64);
                                    result_assigned = true;
                                }
                            }
                        }
                        12 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(2).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(-1.0 * data.amount as f64);
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(data.amount as f64);
                                    result_assigned = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
    });

    result
}


fn prepare_input_accounts(account_indices: &Vec<u8>, accounts: &Vec<String>) -> Vec<String> {
    let mut instruction_accounts: Vec<String> = vec![];
    for (index, &el) in account_indices.iter().enumerate() {
        let account = &accounts[el as usize];
        instruction_accounts.push(account.to_string());
    }
    instruction_accounts
}

fn get_trade_instruction(
    address: &String,
    instruction_data: &Vec<u8>,
    account_indices: &Vec<u8>,
    accounts: &Vec<String>,
    pre_token_balances: &Vec<TokenBalance>,
    post_token_balances: &Vec<TokenBalance>,
    outer_program: &String,
    is_inner: bool,
    inner_instructions: &Vec<UiInstruction>,
    input_inner_idx: u32,
) -> Option<TradeInstruction> {
    let input_accounts = prepare_input_accounts(account_indices, accounts);

    let mut result = None;
    match address.as_str() {
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => {
            result = 
                parse_trade_instruction(
                    &instruction_data,
                    input_accounts,
                    &post_token_balances,
                    accounts,
                );
        }
        _ => {}
    }

    result
}

fn get_system_program_transfer(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<UiInnerInstructions>,
    accounts: &Vec<String>,
    account_name_to_check: String,
    pre_balances: Vec<u64>,
    post_balances: Vec<u64>,
) -> f64 {
    let mut result = 0.0;
    let mut result_assigned = false;

    inner_instructions.iter().for_each(|inner_instruction| {
        inner_instruction
            .instructions
            .iter()
            .enumerate()
            .for_each(|(inner_idx, inner_inst)| {
                let inner_inst = match inner_inst {
                    UiInstruction::Parsed(_) => return,
                    UiInstruction::Compiled(compiled) => compiled,
                };
                let inner_program = &accounts[inner_inst.program_id_index as usize];

                if inner_program
                    .as_str()
                    .eq("11111111111111111111111111111111")
                {
                    // decode hex
                    let data = bs58::decode(inner_inst.data.clone()).into_vec().expect("Error decoding data");                    let (discriminator_bytes, rest) = data.split_at(4);

                    let disc_bytes_arr: [u8; 4] = discriminator_bytes.to_vec().try_into().unwrap();
                    let discriminator: u32 = u32::from_le_bytes(disc_bytes_arr);

                    match discriminator {
                        2 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(1).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = -1.0 * data.amount as f64;
                                    result /= 10f64.powi(9);
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = Transfer::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = 1.0 * data.amount as f64;
                                    result /= 10f64.powi(9);
                                    result_assigned = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
    });

    if !result_assigned {
        let index = accounts.iter().position(|r| r == address).unwrap();
        let _result = post_balances[index] as f64 - pre_balances[index] as f64;
        result = 1.0 * _result as f64;
        result /= 10f64.powi(9);
    }

    result
}

#[tokio::main]
async fn main() {
    let rpc_url = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL is not set");
    let rpc_connection = Arc::new(RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed()));

    process_block(fetch_block_with_version(&rpc_connection, 281418454).await.unwrap());
}
