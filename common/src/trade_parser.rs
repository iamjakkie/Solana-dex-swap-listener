use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiInstruction;

use crate::models::{TokenBalance, TradeInstruction};
use crate::utils::prepare_input_accounts;

fn parse_raydium_trade_instruction(
    bytes_stream: &Vec<u8>,
    input_accounts: Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
    base_address: &String,
    quote_address: &String,
) -> Option<TradeInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(1);
    let discriminator: u8 = u8::from(disc_bytes[0]);

    let mut result = None;

    match discriminator {
        9 => {
            result = Some(TradeInstruction {
                dapp_address: String::from("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                name: String::from("SwapBaseIn"),
                amm: input_accounts.get(1).unwrap().to_string(),
                vault_a: base_address.to_string(),
                vault_b: quote_address.to_string(),
                ..Default::default()
            });
        }
        11 => {
            result = Some(TradeInstruction {
                dapp_address: String::from("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                name: String::from("SwapBaseOut"),
                amm: input_accounts.get(1).unwrap().to_string(),
                vault_a: base_address.to_string(),
                vault_b: quote_address.to_string(),
                ..Default::default()
            });
        }
        _ => {}
    }

    return result;
}

fn parse_meteora_trade_instruction(
    bytes_stream: &Vec<u8>,
    accounts: &Vec<String>,
) -> Option<TradeInstruction>{
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);

    let mut result: Option<TradeInstruction> = None;

    let swap_with_partner_discriminator: u64 = u64::from_le_bytes([248, 198, 158, 145, 225, 117, 135, 200]);

    match discriminator {
        swap_with_partner_discriminator => {
            result = Some(TradeInstruction {
                dapp_address: String::from("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB"),
                name: String::from("Swap"),
                amm: accounts.get(0).unwrap().to_string(),
                vault_a: accounts.get(5).unwrap().to_string(),
                vault_b: accounts.get(6).unwrap().to_string(),
                ..Default::default()
            });
        },
        _ => {}
    }

    return result;
}

fn parse_meteora_dlmm_trade_instruction(

) -> Option<TradeInstruction> {
    
}

pub fn get_trade_instruction(
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
    base_address: &String,
    quote_address: &String,
) -> Option<TradeInstruction> {
    let input_accounts = prepare_input_accounts(account_indices, accounts);
    let mut result = None;
    match address.as_str() {
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => {
            result = parse_raydium_trade_instruction(
                &instruction_data,
                input_accounts,
                &post_token_balances,
                accounts,
                base_address,
                quote_address,
            );
        },
        "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB" => {
            result = parse_meteora_trade_instruction(
                &instruction_data,
                accounts,
            );
        },
        "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo" => {
            result = parse_meteora_dlmm_trade_instruction(

            )
        },
        _ => {}
    }

    result
}
