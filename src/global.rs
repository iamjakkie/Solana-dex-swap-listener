use lazy_static::lazy_static;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::{env, sync::Arc};

lazy_static! {
    pub static ref RPC_CLIENT: Arc<RpcClient> = {
        let rpc_url = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL is not set");
        Arc::new(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ))
    };
}

lazy_static! {
    // set output file path for avro files based on OUTPUT_PATH env variable
    pub static ref OUTPUT_PATH: String = {
        env::var("OUTPUT_PATH").expect("OUTPUT_PATH is not set")
    };
}
