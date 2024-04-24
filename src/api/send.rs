// src/api/send.rs

use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcSendTransactionConfig, RpcTransactionConfig},
    rpc_response::RpcResult,
};
use solana_sdk::{
    signature::{Signer, Keypair},
    transaction::Transaction,
    commitment_config::CommitmentConfig,
};
use std::{sync::Arc, time::Duration, error::Error};
use tokio::time::sleep;

pub struct Sender {
    pub rpc_client: Arc<RpcClient>,
    pub admin_keypair: Keypair,
}

impl Sender {
    pub fn new(rpc_client: Arc<RpcClient>, admin_keypair: Keypair) -> Self {
        Self { rpc_client, admin_keypair }
    }

    /// Sends a batch of transactions with retries and commitment configuration.
    pub async fn send_transactions(
        &self,
        transactions: Vec<Transaction>,
        max_retries: usize,
    ) -> Result<(), Box<dyn Error>> {
        for mut transaction in transactions {
            let mut retries = 0;
            while retries <= max_retries {
                // Fetch the latest blockhash for each retry to avoid "Blockhash not found" errors
                let blockhash = self.rpc_client.get_latest_blockhash().await?.0;
                transaction.sign(&[&self.admin_keypair], blockhash);

                // Send and confirm the transaction with a commitment level of 'confirmed'
                let config = RpcTransactionConfig {
                    skip_preflight: true,
                    preflight_commitment: Some(CommitmentConfig::confirmed()), // Ensure we use 'confirmed' commitment for preflight
                    ..Default::default()
                };

                match self.rpc_client.send_and_confirm_transaction_with_spinner_and_config(
                    &transaction,
                    CommitmentConfig::confirmed(),
                    config
                ).await {
                    Ok(_) => break,
                    Err(e) if retries >= max_retries => return Err(Box::new(e)),
                    Err(_) => {
                        retries += 1;
                        sleep(Duration::from_secs(retries.pow(2) as u64)).await; // Exponential backoff
                    },
                }
            }
        }
        Ok(())
    }
}
