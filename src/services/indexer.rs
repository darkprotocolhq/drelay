// indexer.rs 

use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_client::rpc_filter::RpcFilterType;
use solana_sdk::transaction::Transaction;
use std::collections::HashSet;
use log::{info, error};
use tokio;

struct LightTransaction {
    signature: String,
    block_time: i64,  // Assuming block_time is an integer timestamp
}

#[derive(Debug)]
struct LightToken {
    pub mint: String,
    pub owner: String,
}

async fn index_streamed_transactions(
    rpc_client: &RpcClient,
    new_raw_transactions: Vec<Transaction>,
    token: &LightToken,
) {
    info!("webhook sol, updating jobs");

    let new_parsed_transactions: Vec<LightTransaction> = new_raw_transactions.iter().map(|tx| {
        parse_transactions(tx, rpc_client, token)
    }).collect::<Result<Vec<_>, _>>().await.unwrap_or_else(|e| {
        error!("Failed to parse transactions: {:?}", e);
        Vec::new()
    });

    // Dummy implementation for merging and sorting transactions
    let merged_transactions = merge_and_sort_transactions(Vec::new(), vec![new_parsed_transactions]);
}

async fn index_transactions(
    rpc_client: &RpcClient,
    token: &LightToken,
) {
    let older_transactions = search_backward(rpc_client, token).await.unwrap_or_default();
    let newer_transactions = search_forward(rpc_client, token).await.unwrap_or_default();

    let deduped_transactions = merge_and_sort_transactions(Vec::new(), vec![older_transactions, newer_transactions]);
    info!("{} -- new total: {} transactions", token.mint, deduped_transactions.len());
}

fn merge_and_sort_transactions(
    db_transactions: Vec<LightTransaction>,
    new_transactions: Vec<Vec<LightTransaction>>,
) -> Vec<LightTransaction> {
    let mut all_transactions = db_transactions;
    for transactions in new_transactions {
        all_transactions.extend(transactions);
    }

    let mut unique_signatures = HashSet::new();
    let deduped_transactions: Vec<LightTransaction> = all_transactions.into_iter().filter(|tx| unique_signatures.insert(tx.signature.clone())).collect();
    deduped_transactions.sort_by(|a, b| b.block_time.cmp(&a.block_time));
    deduped_transactions
}
