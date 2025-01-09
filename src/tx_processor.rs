use anyhow::Result;
use borsh::BorshDeserialize;
use solana_sdk::bs58;
use solana_transaction_status::{
    EncodedConfirmedBlock, EncodedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction, UiParsedInstruction
};
use spl_token::instruction::TokenInstruction;

use crate::{
    models::{PoolData, TokenBalance, TradeData, UiTokenAmount},
    trade_parser::get_trade_instruction,
    utils::{convert_to_date, get_amt, get_mint, get_signer_balance_change},
};

const RAYDIUM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

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

    if !accounts.contains(&RAYDIUM_PROGRAM_ID.to_string()) {
        return None;
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
            address: accounts.get(balance.account_index as usize).unwrap().to_string(),
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
            address: accounts.get(balance.account_index as usize).unwrap().to_string(),
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

        let first_instruction = trx_meta_inner.first();


        let mut first_instruction_ok: &UiInnerInstructions;

        if first_instruction.is_none() {
            // println!("Signature: {:?}", ui.signatures[0]);
            continue;
        } else {
            first_instruction_ok = first_instruction.unwrap();
        }


        let inner_instructions = first_instruction_ok.clone().instructions;

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

        let pool_data = PoolData::try_from_slice(&decoded_data);

        let base_add = &accounts.get(inst.accounts[5] as usize).expect("Base account not found");
        let quote_add = &accounts.get(inst.accounts[6] as usize).expect("Quote account not found");

        // print signature
        // println!("Signature: {:?}", ui.signatures[0]);

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
            0 as u32,
            base_add,
            quote_add,
        );

        if trade_data.is_some() {
            let td = trade_data.unwrap();
            // println!("Trade Data: {:?}", td);

            let td_name = td.name;
            let td_address = td.dapp_address;

            

            let trade = TradeData {
                block_date: convert_to_date(timestamp),
                tx_id: bs58::encode(&ui.signatures[0]).into_string(),
                block_slot: slot,
                block_time: timestamp,
                signature: ui.signatures[0].clone(),
                signer: accounts.get(0).unwrap().to_string(),
                pool_address: td.amm,
                base_mint: get_mint(
                    &td.vault_a,
                    &post_token_balances_vec,
                ).unwrap(),
                quote_mint: get_mint(
                    &td.vault_b,
                    &post_token_balances_vec,
                ).unwrap(),
                // base_mint: get_mint(
                //     &td.vault_a,
                //     &post_token_balances_vec,
                //     &accounts,
                //     td_address.clone(),
                // )
                // .unwrap(),
                // quote_mint: get_mint(
                //     &td.vault_b,
                //     &post_token_balances_vec,
                //     &accounts,
                //     "".to_string(),
                // )
                // .unwrap(),
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
                signer_lamports_change: get_signer_balance_change(&pre_balances, &post_balances),
            };

            trades.push(trade);

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

        // commented out for tests
        // TODO: check if correct
        // trx_meta.inner_instructions.clone()
        //     .expect("Inner instructions not found")
        //     .iter()
        //     .filter(|inner_instruction| inner_instruction.index == idx as u8)
        //     .for_each(|inner_instruction| {
        //         inner_instruction.instructions.iter().enumerate().for_each(
        //             |(inner_idx, inner_inst)| {
        //                 let inner_program =
        //                     &accounts[inst.program_id_index as usize];
        //                 let inner_trade_data = get_trade_instruction(
        //                     inner_program,
        //                     &decoded_data,
        //                     &inst.accounts,
        //                     &accounts,
        //                     &pre_token_balances_vec,
        //                     &post_token_balances_vec,
        //                     &program.to_string(),
        //                     true,
        //                     &inner_instructions,
        //                     inner_idx as u32,
        //                 );

        //                 if inner_trade_data.is_some() {
        //                     let inner_td = inner_trade_data.unwrap();

        //                     let inner_td_name = inner_td.name;
        //                     let inner_td_dapp_address = inner_td.dapp_address;

        //                     data.push(TradeData {
        //                         block_date: convert_to_date(timestamp),
        //                         tx_id: bs58::encode(&signature)
        //                             .into_string(),
        //                         block_slot: slot,
        //                         block_time: timestamp,
        //                         signer: accounts.get(0).unwrap().to_string(),
        //                         pool_address: inner_td.amm,
        //                         base_mint: get_mint(
        //                             &inner_td.vault_a,
        //                             &post_token_balances_vec,
        //                             &accounts,
        //                             inner_td_dapp_address.clone(),
        //                         ),
        //                         quote_mint: get_mint(
        //                             &inner_td.vault_b,
        //                             &post_token_balances_vec,
        //                             &accounts,
        //                             "".to_string(),
        //                         ),
        //                         base_amount: get_amt(
        //                             &inner_td.vault_a,
        //                             inner_idx as u32,
        //                             &trx_meta_inner,
        //                             &accounts,
        //                             &post_token_balances_vec,
        //                             inner_td_dapp_address.clone(),
        //                             pre_balances.clone(),
        //                             post_balances.clone(),
        //                         ),
        //                         quote_amount: get_amt(
        //                             &inner_td.vault_b,
        //                             inner_idx as u32,
        //                             &trx_meta_inner,
        //                             &accounts,
        //                             &post_token_balances_vec,
        //                             "".to_string(),
        //                             pre_balances.clone(),
        //                             post_balances.clone(),
        //                         ),
        //                         base_vault: inner_td.vault_a,
        //                         quote_vault: inner_td.vault_b,
        //                         is_inner_instruction: true,
        //                         instruction_index: idx as u32,
        //                         instruction_type: inner_td_name.clone(),
        //                         inner_instruction_index: inner_idx as u32,
        //                         outer_program: program.to_string(),
        //                         inner_program: inner_td_dapp_address.clone(),
        //                         txn_fee_lamports: trx_meta.fee,
        //                         signer_lamports_change: get_signer_balance_change(
        //                             &pre_balances,
        //                             &post_balances,
        //                         ),
        //                     });

        //                     // if inner_td.second_swap_amm.clone().unwrap_or_default()
        //                     //     != ""
        //                     // {
        //                     //     data.push(TradeData {
        //                     //         block_date: convert_to_date(timestamp),
        //                     //         tx_id: bs58::encode(&transaction.signatures[0])
        //                     //             .into_string(),
        //                     //         block_slot: slot,
        //                     //         block_time: timestamp,
        //                     //         signer: accounts.get(0).unwrap().to_string(),
        //                     //         pool_address: inner_td
        //                     //             .second_swap_amm
        //                     //             .clone()
        //                     //             .unwrap(),
        //                     //         base_mint: get_mint(
        //                     //             &inner_td.second_swap_vault_a.clone().unwrap(),
        //                     //             &post_token_balances,
        //                     //             &accounts,
        //                     //             "".to_string(),
        //                     //         ),
        //                     //         quote_mint: get_mint(
        //                     //             &inner_td.second_swap_vault_b.clone().unwrap(),
        //                     //             &post_token_balances,
        //                     //             &accounts,
        //                     //             "".to_string(),
        //                     //         ),
        //                     //         base_amount: get_amt(
        //                     //             &inner_td.second_swap_vault_a.clone().unwrap(),
        //                     //             inner_idx as u32,
        //                     //             &inner_instructions,
        //                     //             &accounts,
        //                     //             &post_token_balances,
        //                     //             "".to_string(),
        //                     //             pre_balances.clone(),
        //                     //             post_balances.clone(),
        //                     //         ),
        //                     //         quote_amount: get_amt(
        //                     //             &inner_td.second_swap_vault_b.clone().unwrap(),
        //                     //             inner_idx as u32,
        //                     //             &inner_instructions,
        //                     //             &accounts,
        //                     //             &post_token_balances,
        //                     //             "".to_string(),
        //                     //             pre_balances.clone(),
        //                     //             post_balances.clone(),
        //                     //         ),
        //                     //         base_vault: inner_td
        //                     //             .second_swap_vault_a
        //                     //             .clone()
        //                     //             .unwrap(),
        //                     //         quote_vault: inner_td
        //                     //             .second_swap_vault_b
        //                     //             .clone()
        //                     //             .unwrap(),
        //                     //         is_inner_instruction: true,
        //                     //         instruction_index: idx as u32,
        //                     //         instruction_type: inner_td_name.clone(),
        //                     //         inner_instruction_index: inner_idx as u32,
        //                     //         outer_program: program.to_string(),
        //                     //         inner_program: inner_td_dapp_address.clone(),
        //                     //         txn_fee_lamports: meta.fee,
        //                     //         signer_lamports_change: get_signer_balance_change(
        //                     //             &pre_balances,
        //                     //             &post_balances,
        //                     //         ),
        //                     //     });
        //                     // }
        //                 }
        //             },
        //         )
        //     });
    }
    Some(trades)
}
