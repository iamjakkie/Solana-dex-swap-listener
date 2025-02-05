use crate::global::RPC_CLIENT;
use crate::models::{MarketDataStruct, TokenBalance, TradeData, Transfer};
use borsh::BorshDeserialize;
use chrono::{DateTime, NaiveDateTime, Utc};
use csv::WriterBuilder;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{bs58, inner_instruction};
use solana_transaction_status::{UiInnerInstructions, UiInstruction};
use std::fs::{create_dir_all, OpenOptions};
use std::path::Path;
use std::str::FromStr;

// pub fn get_mint(
//     address: &String,
//     token_balances: &Vec<TokenBalance>,
//     accounts: &Vec<String>,
//     dapp_address: String,
// ) -> Option<String> {
//     if dapp_address.eq("MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG")
//         || dapp_address.eq("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P")
//     {
//         return Some("So11111111111111111111111111111111111111112".to_string());
//     }
//     // get spl token address for token account (address)
//     let add = Pubkey::from_str(address).unwrap();
//     let rpc_client = RPC_CLIENT.clone();
//     let acc_data = rpc_client.get_account(&add).unwrap();
//     let token_data = spl_token::state::Account::unpack(acc_data.data.as_slice());
//     match token_data {
//         Ok(token_data) => {
//             let mint = token_data.mint.to_string();
//             Some(mint)
//         }
//         Err(_) => {
//             let index = accounts.iter().position(|r| r == address).unwrap();
//             let mut result: String = String::new();
//             token_balances
//                 .iter()
//                 .filter(|token_balance| token_balance.account_index == index as u32)
//                 .for_each(|token_balance| {
//                     result = token_balance.mint.clone();
//                 });
//             Some(result)
//         }
//     }
// }

pub fn get_mint(address: &String, token_balances: &Vec<TokenBalance>) -> Option<String> {
    let index = token_balances.iter().position(|r| r.address == *address);
    match index {
        None => None,
        Some(index) => {
            let mint = token_balances.get(index).unwrap().mint.clone();
            Some(mint)
        }
    }
}

pub fn get_amm_data(amm_address: &String) {
    let add = Pubkey::from_str(amm_address).unwrap();
    let rpc_client = RPC_CLIENT.clone();
    let acc_data = rpc_client.get_account(&add).unwrap().data;
    let decoded_data = MarketDataStruct::try_from_slice(&acc_data).unwrap();
    // println!("Account Data: {:?}", decoded_data);
}

// pub fn get_vault_a(
//     input_accounts: &Vec<String>,
//     post_token_balances: &Vec<TokenBalance>,
//     accounts: &Vec<String>,
// ) -> String {
//     let mut vault_a = input_accounts.get(4).unwrap().to_string();
//     let mint_a = get_mint(&vault_a, post_token_balances, accounts, "".to_string());

//     if mint_a.is_some() {
//         vault_a = input_accounts.get(5).unwrap().to_string();
//     }

//     return vault_a;
// }

// pub fn get_vault_b(
//     input_accounts: &Vec<String>,
//     post_token_balances: &Vec<TokenBalance>,
//     accounts: &Vec<String>,
// ) -> String {
//     println!("Input Accounts: {:?}", input_accounts);

//     let mut vault_a_index = 4;

//     let mut vault_a = input_accounts.get(4).unwrap().to_string();
//     let mint_a = get_mint(&vault_a, post_token_balances, accounts, "".to_string());

//     if mint_a.is_some() {
//         vault_a_index += 1;
//         vault_a = input_accounts.get(vault_a_index).unwrap().to_string();
//     }

//     let mut vault_b_index = vault_a_index + 1;
//     let mut vault_b = input_accounts.get(vault_b_index).unwrap().to_string();

//     if vault_a == vault_b {
//         vault_b_index += 1;
//         vault_b = input_accounts.get(vault_b_index).unwrap().to_string();
//     }

//     return vault_b;
// }

pub fn get_signer_balance_change(pre_balances: &Vec<u64>, post_balances: &Vec<u64>) -> i64 {
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

    let mint = get_mint(address, post_token_balances).unwrap();

    if mint == "So11111111111111111111111111111111111111112" {
        // get solana balance change
        return (get_signer_balance_change(&pre_balances, &post_balances) as f64)
            / (u64::pow(10, 9)) as f64;
    }

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
        // let index = accounts.iter().position(|r| r == address).unwrap();
        post_token_balances
            .iter()
            .filter(|token_balance| token_balance.address == *address)
            .for_each(|token_balance: &TokenBalance| {
                let decimals = token_balance.ui_token_amount.clone().decimals;
                result = result / (u64::pow(10, decimals)) as f64;
            });
    }

    -result
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
                let program_id_index = inner_inst.program_id_index as usize;
                if program_id_index >= accounts.len() {
                    return;
                }
                let inner_program = &accounts[program_id_index];
                if inner_program
                    .as_str()
                    .eq("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                {
                    // println!("Inner Program: {:?}", inner_program);
                    // println!("Data: {:?}", inner_inst.data.clone().into_bytes());
                    let data = bs58::decode(inner_inst.data.clone())
                        .into_vec()
                        .expect("Error decoding data");
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
                    let data = bs58::decode(inner_inst.data.clone())
                        .into_vec()
                        .expect("Error decoding data");
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

pub fn prepare_input_accounts(account_indices: &Vec<u8>, accounts: &Vec<String>) -> Vec<String> {
    let mut instruction_accounts: Vec<String> = vec![];
    for (index, &el) in account_indices.iter().enumerate() {
        if el >= accounts.len() as u8 {
            continue;
        }
        let account = &accounts[el as usize];
        instruction_accounts.push(account.to_string());
    }
    instruction_accounts
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
                    let data = bs58::decode(inner_inst.data.clone())
                        .into_vec()
                        .expect("Error decoding data");
                    let (discriminator_bytes, rest) = data.split_at(4);

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

pub async fn save_trades_to_csv(trades: &Vec<TradeData>, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(file_path).parent() {
        create_dir_all(parent)?;
    }
    
    // Check if the file exists
    let file_exists = Path::new(file_path).exists();

    // Open the file in append mode
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    // Configure the writer
    let mut writer = WriterBuilder::new()
        .has_headers(!file_exists) // Only write headers if the file doesn't exist
        .from_writer(file);

    // Write each trade
    for trade in trades {
        writer.serialize(trade)?;
    }

    writer.flush()?; // Ensure all data is written to the file
    println!("Saved {} trades to {}", trades.len(), file_path);

    Ok(())
}