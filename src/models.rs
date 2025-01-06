use borsh::{BorshDeserialize, BorshSerialize};
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;

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
    pub signature: String,
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

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct MarketDataStruct {
    pub status: u64,
    pub nonce: u64,
    pub max_order: u64,
    pub depth: u64,
    pub base_decimal: u64,
    pub quote_decimal: u64,
    pub state: u64,
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub min_price_multiplier: u64,
    pub max_price_multiplier: u64,
    pub system_decimal_value: u64,
    pub min_separate_numerator: u64,
    pub min_separate_denominator: u64,
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub pnl_numerator: u64,
    pub pnl_denominator: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
    pub base_need_take_pnl: u64,
    pub quote_need_take_pnl: u64,
    pub quote_total_pnl: u64,
    pub base_total_pnl: u64,
    pub pool_open_time: u64,
    pub punish_pc_amount: u64,
    pub punish_coin_amount: u64,
    pub orderbook_to_init_time: u64,

    // 128-bit fields (u128 in Rust)
    pub swap_base_in_amount: u128,
    pub swap_quote_out_amount: u128,
    pub swap_base2_quote_fee: u64, // stays u64 per your snippet
    pub swap_quote_in_amount: u128,
    pub swap_base_out_amount: u128,
    pub swap_quote2_base_fee: u64, // stays u64 per your snippet

    // Each publicKey => 32 bytes
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub base_mint: [u8; 32],
    pub quote_mint: [u8; 32],
    pub lp_mint: [u8; 32],
    pub open_orders: [u8; 32],
    pub market_id: [u8; 32],
    pub market_program_id: [u8; 32],
    pub target_orders: [u8; 32],
    pub withdraw_queue: [u8; 32],
    pub lp_vault: [u8; 32],
    pub owner: [u8; 32],

    pub lp_reserve: u64,

    // 3 * u64 for padding
    pub padding: [u64; 3],
}