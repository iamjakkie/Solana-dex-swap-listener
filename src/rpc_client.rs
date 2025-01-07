use std::str::FromStr;

use crate::global::RPC_CLIENT;
use anyhow::{Error, Result};
use serde_json::json;
use solana_client::{client_error::ClientError, rpc_client::RpcClient};
use solana_sdk::{
    bs58,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    signature::Signature,
};
use solana_transaction_status::{
    EncodedConfirmedBlock, EncodedConfirmedTransactionWithStatusMeta,
    EncodedTransactionWithStatusMeta,
};

pub async fn fetch_block_with_version(block_slot: u64) -> Result<EncodedConfirmedBlock, Error> {
    let rpc_client = RPC_CLIENT.clone();
    let params = json!([
        block_slot,
        { "maxSupportedTransactionVersion": 0 ,
          "commitment": CommitmentLevel::Confirmed }
    ]);

    let response: serde_json::Value =
        rpc_client.send(solana_client::rpc_request::RpcRequest::GetBlock, params)?;
    let block: EncodedConfirmedBlock = serde_json::from_value(response)?;

    Ok(block)
}

pub async fn get_latest_slot() -> Result<u64, ClientError> {
    let rpc_client = RPC_CLIENT.clone();
    let slot = rpc_client.get_slot_with_commitment(CommitmentConfig::confirmed());
    // let response: serde_json::Value = rpc_client.send(solana_client::rpc_request::RpcRequest::GetSlot, json!([]))?;
    slot
}

pub async fn get_signature(tx: &str) -> Result<EncodedConfirmedTransactionWithStatusMeta, Error> {
    let rpc_client = RPC_CLIENT.clone();
    let signature = Signature::from_str(tx).unwrap();
    let params = json!([
        tx,
        { "maxSupportedTransactionVersion": 0 }
    ]);
    let res = rpc_client.send(
        solana_client::rpc_request::RpcRequest::GetTransaction,
        params,
    )?;
    // let res = rpc_client.get_transaction(&signature, solana_transaction_status::UiTransactionEncoding::Base58)?;
    // let response: serde_json::Value = rpc_client.send(solana_client::rpc_request::RpcRequest::GetSignatureStatus, json!([tx]))?;
    // let block: EncodedConfirmedBlock = serde_json::from_value(response)?;

    Ok(res)
}
