use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, transaction::Transaction, signature::{Keypair, Signer},
    instruction::{Instruction, AccountMeta}, commitment_config::CommitmentConfig
};
use std::sync::Arc;
use dotenv::dotenv;
use std::env;
use crate::api::send::Sender;  
use crate::helper::{self, PublicInputs};  

#[derive(Debug, Clone)]
pub struct JobData {
    pub input: Vec<u8>,
    pub proof: Vec<u8>,
    pub action: String,
    pub ext_data: Vec<u8>,
    pub owner: String,
}

pub struct ProcessUnshield {
    rpc_client: Arc<RpcClient>,
    admin_keypair: Keypair,
    ix_per_tx: usize,
    sender: Sender,
}

impl ProcessUnshield {
    pub fn new() -> Self {
        dotenv().ok(); // Load .env file at runtime
        let rpc_url = env::var("RPC_URL").expect("Expected RPC_URL");
        let rpc_client = Arc::new(RpcClient::new(rpc_url));
        let keypair_path = env::var("ADMIN_KEYPAIR_PATH").expect("Expected ADMIN_KEYPAIR_PATH");
        let admin_keypair = Keypair::from_bytes(&std::fs::read(keypair_path).expect("Failed to read keypair file")).expect("Failed to create keypair");
        let ix_per_tx = env::var("IX_PER_TX").expect("Expected IX_PER_TX").parse::<usize>().expect("IX_PER_TX must be a number");
        let sender = Sender::new(rpc_client.clone(), admin_keypair.clone());

        ProcessUnshield {
            rpc_client,
            admin_keypair,
            ix_per_tx,
            sender,
        }
    }

    pub async fn process_unshield(&self, job: JobData) -> Result<(), Box<dyn std::error::Error>> {
        let transactions = self.create_transaction_pool(&job)?;
        self.sender.send_transactions(transactions, 5).await?;  // Use sender to manage transaction sending with retries and confirmed commitment
        Ok(())
    }

    fn create_transaction_pool(&self, job: &JobData) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await?.0;
        let mut transactions = vec![];

        // Convert inputs and build the instructions based on job data
        let inputs = PublicInputs::new(job.input.clone(), job.ext_data.clone(), job.action.clone(), job.owner.clone());
        let mut instructions = vec![];

        // You might have a function to generate multiple instructions based on the inputs
        instructions.push(Instruction::new_with_bincode(
            Pubkey::from_str(&job.owner)?,
            &inputs.serialize(), // Serialize your inputs into a byte array
            vec![AccountMeta::new(Pubkey::from_str(&job.owner)?, true)],
        ));

        // Create transactions from instructions
        let mut current_tx = Transaction::new_with_payer(&[], Some(&self.admin_keypair.pubkey()));
        for (i, instruction) in instructions.iter().enumerate() {
            if i % self.ix_per_tx == 0 && i != 0 {
                current_tx.sign(&[&self.admin_keypair], recent_blockhash);
                transactions.push(current_tx);
                current_tx = Transaction::new_with_payer(&[], Some(&self.admin_keypair.pubkey()));
            }
            current_tx.add(instruction.clone());
        }

        if !current_tx.signatures.is_empty() {
            current_tx.sign(&[&self.admin_keypair], recent_blockhash);
            transactions.push(current_tx);
        }

        Ok(transactions)
    }
}
