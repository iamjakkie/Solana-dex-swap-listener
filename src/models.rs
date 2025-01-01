use borsh::{BorshDeserialize, BorshSerialize};

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