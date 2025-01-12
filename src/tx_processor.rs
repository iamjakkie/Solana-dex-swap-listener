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
    utils::{convert_to_date, get_amt, get_mint, get_signer_balance_change},
};

const RAYDIUM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
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

    let mut trades = vec![];

    for (idx, inst) in msg.instructions.into_iter().enumerate() {
        let trx_meta_inner = trx_meta.inner_instructions.clone().unwrap_or(vec![]);
        let fee = trx_meta.fee;

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
                        &trx_meta_inner
                            .first()
                            .expect("Inner instructions not found")
                            .instructions,
                        timestamp,
                        slot,
                        &signature,
                        idx,
                        &trx_meta_inner,
                        &pre_balances,
                        &post_balances,
                        fee,
                    ) {
                        trades.push(trade);
                    }
                }
            }
            JUPITER_PROGRAM_ID => {
                // Gather all Raydium (base, quote) pairs within Jupiter instructions
                trx_meta_inner.iter().for_each(|inner| {

                        let inner_instructions = inner.instructions.clone();

                        let jupiter_trades: Vec<TradeData> = inner_instructions
                            .iter()
                            .filter_map(|inner_inst| {
                                if let UiInstruction::Compiled(compiled) = inner_inst {
                                    let program_data =
                                        bs58::decode(compiled.data.clone()).into_vec().unwrap();
                                    let program_add =
                                        all_addresses.get(compiled.program_id_index as usize)?;
                                    if program_add == RAYDIUM_PROGRAM_ID {
                                        // Hardcoded indices 4, 5, as before
                                        let base_add = all_addresses
                                            .get(compiled.accounts[4] as usize)
                                            .expect("Base account not found")
                                            .clone();
                                        let quote_add = all_addresses
                                            .get(compiled.accounts[5] as usize)
                                            .expect("Quote account not found")
                                            .clone();

                                        return build_trade_data(
                                            program_add,
                                            &program_data,
                                            &compiled.accounts,
                                            &all_addresses,
                                            &pre_token_balances_vec,
                                            &post_token_balances_vec,
                                            &base_add,
                                            &quote_add,
                                            &inner_instructions,
                                            timestamp,
                                            slot,
                                            &signature,
                                            idx,
                                            &trx_meta_inner,
                                            &pre_balances,
                                            &post_balances,
                                            fee,
                                        );

                                    }
                                }
                                None
                            })
                            .collect();
                        trades.extend(jupiter_trades);
                    });
                }

            _ => {}
        };

        // let first_instruction = trx_meta_inner.first();

        // let mut first_instruction_ok: &UiInnerInstructions;

        // if first_instruction.is_none() {
        //     // println!("Signature: {:?}", ui.signatures[0]);
        //     continue;
        // } else {
        //     first_instruction_ok = first_instruction.unwrap();
        // }

        // let inner_instructions = first_instruction_ok.clone().instructions;

        // // decode data using base58
        // let decoded_data = bs58::decode(inst.data.clone()).into_vec().unwrap();

        // let pool_data = PoolData::try_from_slice(&decoded_data);

        // println!("Signature: {:?}", ui.signatures[0]);
        // println!("Instruction accs: {:?}", inst.accounts);
        // println!("Accounts: Len: {:?}, Entries: {:?}", accounts.len(), accounts);

        // println!("Signature: {:?}", ui.signatures[0]);

        // println!("Base: {:?}, Quote: {:?}", base_add, quote_add);

        // let trade_data = get_trade_instruction(
        //     program,
        //     &decoded_data,
        //     &inst.accounts,
        //     &accounts,
        //     &pre_token_balances_vec,
        //     &post_token_balances_vec,
        //     &"".to_string(),
        //     false,
        //     &inner_instructions,
        //     0 as u32,
        //     &base_add,
        //     &quote_add,
        // );

        // if trade_data.is_some() {
        //     let td = trade_data.unwrap();
        //     // println!("Trade Data: {:?}", td);

        //     let td_name = td.name;
        //     let td_address = td.dapp_address;

        //     let trade = TradeData {
        //         block_date: convert_to_date(timestamp),
        //         tx_id: bs58::encode(&ui.signatures[0]).into_string(),
        //         block_slot: slot,
        //         block_time: timestamp,
        //         signature: ui.signatures[0].clone(),
        //         signer: accounts.get(0).unwrap().to_string(),
        //         pool_address: td.amm,
        //         base_mint: get_mint(&td.vault_a, &post_token_balances_vec).unwrap(),
        //         quote_mint: get_mint(&td.vault_b, &post_token_balances_vec).unwrap(),
        //         base_amount: get_amt(
        //             &td.vault_a,
        //             0 as u32,
        //             &trx_meta_inner,
        //             &accounts,
        //             &post_token_balances_vec,
        //             td_address.clone(),
        //             pre_balances.clone(),
        //             post_balances.clone(),
        //         ),
        //         quote_amount: get_amt(
        //             &td.vault_b,
        //             0 as u32,
        //             &trx_meta_inner,
        //             &accounts,
        //             &post_token_balances_vec,
        //             "".to_string(),
        //             pre_balances.clone(),
        //             post_balances.clone(),
        //         ),
        //         base_vault: td.vault_a,
        //         quote_vault: td.vault_b,
        //         is_inner_instruction: false,
        //         instruction_index: idx as u32,
        //         instruction_type: td_name.clone(),
        //         inner_instruction_index: 0,
        //         outer_program: td_address.clone(),
        //         inner_program: "".to_string(),
        //         txn_fee_lamports: trx_meta.fee,
        //         signer_lamports_change: get_signer_balance_change(&pre_balances, &post_balances),
        //     };

        //     trades.push(trade);
        // }
    }
    Some(trades)
}

fn build_trade_data(
    program: &String,
    decoded_data: &Vec<u8>,
    inst_accounts: &Vec<u8>,
    accounts: &Vec<String>,
    pre_token_balances_vec: &Vec<TokenBalance>,
    post_token_balances_vec: &Vec<TokenBalance>,
    base_add: &String,
    quote_add: &String,
    // Additional parameters from your snippet
    inner_instructions: &Vec<UiInstruction>,
    timestamp: i64,
    slot: u64,
    signature: &String,
    idx: usize,
    trx_meta_inner: &Vec<UiInnerInstructions>,
    pre_balances: &Vec<u64>,
    post_balances: &Vec<u64>,
    fee: u64,
) -> Option<TradeData> {
    // println!()
    let trade_data = get_trade_instruction(
        program,
        decoded_data,
        inst_accounts,
        accounts,
        pre_token_balances_vec,
        post_token_balances_vec,
        &"".to_string(),
        false,
        inner_instructions,
        0,
        base_add,
        quote_add,
    );

    // 2. If there's a return, build the TradeData struct
    if let Some(td) = trade_data {
        let td_name = td.name;
        let td_address = td.dapp_address;

        let trade = TradeData {
            block_date: convert_to_date(timestamp),
            tx_id: bs58::encode(signature).into_string(),
            block_slot: slot,
            block_time: timestamp,
            signature: signature.to_string(),
            signer: accounts.get(0).unwrap().to_string(),
            pool_address: td.amm,
            base_mint: get_mint(&td.vault_a, post_token_balances_vec).unwrap(),
            quote_mint: get_mint(&td.vault_b, post_token_balances_vec).unwrap(),
            base_amount: get_amt(
                &td.vault_a,
                0,
                trx_meta_inner,
                accounts,
                post_token_balances_vec,
                td_address.clone(),
                pre_balances.clone(),
                post_balances.clone(),
            ),
            quote_amount: get_amt(
                &td.vault_b,
                0,
                trx_meta_inner,
                accounts,
                post_token_balances_vec,
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
            txn_fee_lamports: fee,
            signer_lamports_change: get_signer_balance_change(pre_balances, post_balances),
        };

        Some(trade)
    } else {
        None
    }
}
