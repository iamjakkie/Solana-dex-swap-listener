use solana_client::rpc_client::RpcClient;
use solana_transaction_status::EncodedConfirmedBlock;
use anyhow::Result;
use serde_json::json;
use crate::global::RPC_CLIENT;

pub async fn fetch_block_with_version(
    block_slot: u64,
) -> Result<EncodedConfirmedBlock> {
    let rpc_client = RPC_CLIENT.clone();
    let params = json!([
        block_slot,
        { "maxSupportedTransactionVersion": 0 }
    ]);

    let response: serde_json::Value = rpc_client.send(solana_client::rpc_request::RpcRequest::GetBlock, params)?;
    let block: EncodedConfirmedBlock = serde_json::from_value(response)?;

    Ok(block)
}