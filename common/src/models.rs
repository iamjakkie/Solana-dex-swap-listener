use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
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
    pub address: String,
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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
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

// TODO: This works but requires 1 extra call, the sama data can be parsed
// out of the inner intructions/instructions from Raydium
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
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub open_orders: Pubkey,
    pub market_id: Pubkey,
    pub market_program_id: Pubkey,
    pub target_orders: Pubkey,
    pub withdraw_queue: Pubkey,
    pub lp_vault: Pubkey,
    pub owner: Pubkey,

    pub lp_reserve: u64,

    // 3 * u64 for padding
    pub padding: [u64; 3],
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct PoolData {
    /// #1 - Token Program
    pub token_program: Pubkey,

    /// #2 - Amm
    pub amm: Pubkey,

    /// #3 - Amm Authority
    pub amm_authority: Pubkey,

    /// #4 - Amm Open Orders
    pub amm_open_orders: Pubkey,

    /// #5 - Amm Target Orders
    pub amm_target_orders: Pubkey,

    /// #6 - Pool Coin Token Account
    pub pool_coin_token_account: Pubkey,

    /// #7 - Pool Pc Token Account
    pub pool_pc_token_account: Pubkey,

    /// #8 - Serum Program
    pub serum_program: Pubkey,

    /// #9 - Serum Market
    pub serum_market: Pubkey,

    /// #10 - Serum Bids
    pub serum_bids: Pubkey,

    /// #11 - Serum Asks
    pub serum_asks: Pubkey,

    /// #12 - Serum Event Queue
    pub serum_event_queue: Pubkey,

    /// #13 - Serum Coin Vault Account
    pub serum_coin_vault_account: Pubkey,

    /// #14 - Serum Pc Vault Account
    pub serum_pc_vault_account: Pubkey,

    /// #15 - Serum Vault Signer
    pub serum_vault_signer: Pubkey,

    /// #16 - User Source Token Account
    pub user_source_token_account: Pubkey,

    /// #17 - User Destination Token Account
    pub user_destination_token_account: Pubkey,

    /// #18 - User Source Owner
    pub user_source_owner: Pubkey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZmqData {
    pub slot: u64,
    pub date: String,
    pub data: Vec<TradeData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KlineData {
    pub open_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: u64,
    pub quote_asset_volume: f64,
    pub number_of_trades: u64,
    pub taker_buy_base_asset_volume: f64,
    pub taker_buy_quote_asset_volume: f64,
    pub ignore: u64,
}

#[derive(Debug, Deserialize)]
pub struct KlineRecord {
    #[serde(rename = "Open time")]
    open_time: u64,
    #[serde(rename = "Open")]
    open: f64,
    #[serde(rename = "High")]
    high: f64,
    #[serde(rename = "Low")]
    low: f64,
    #[serde(rename = "Close")]
    close: f64,
    #[serde(rename = "Volume")]
    volume: f64,
    #[serde(rename = "Close time")]
    close_time: u64,
    #[serde(rename = "Quote asset volume")]
    quote_asset_volume: f64,
    #[serde(rename = "Number of trades")]
    number_of_trades: u64,
    #[serde(rename = "Taker buy base asset volume")]
    taker_buy_base_asset_volume: f64,
    #[serde(rename = "Taker buy quote asset volume")]
    taker_buy_quote_asset_volume: f64,
    #[serde(rename = "Ignore")]
    ignore: u64,
}


impl From<KlineRecord> for KlineData {
    fn from(rec: KlineRecord) -> Self {
        KlineData {
            open_time: rec.open_time,
            open: rec.open,
            high: rec.high,
            low: rec.low,
            close: rec.close,
            volume: rec.volume,
            close_time: rec.close_time,
            quote_asset_volume: rec.quote_asset_volume,
            number_of_trades: rec.number_of_trades,
            taker_buy_base_asset_volume: rec.taker_buy_base_asset_volume,
            taker_buy_quote_asset_volume: rec.taker_buy_quote_asset_volume,
            ignore: rec.ignore,
        }
    }
}
