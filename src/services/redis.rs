//src/services/redis.rs
use redis::{AsyncCommands, Client, aio::Connection};
use tokio::sync::Mutex;
use std::{env, sync::Arc};
use dotenv::dotenv;
use log::{info, error};
use serde::{Serialize, Deserialize};

// Import from the `api` module
use crate::api::process_unshield::{ProcessUnshield, JobData};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Transaction {
    id: String,
    input_bytes: Vec<u8>,
    proof_bytes: Vec<u8>,
    ext_data_bytes: Vec<u8>,
    action: String,
    owner: String,
    token: String,
}

#[derive(Debug, Clone)]
struct Job {
    id: String,
    transactions: Vec<Transaction>,
}

struct UnshieldQueue {
    queue: Arc<Mutex<Vec<Job>>>,
    redis_conn: Arc<Mutex<Connection>>,
    process_unshield: ProcessUnshield,
}

impl UnshieldQueue {
    async fn new(redis_conn: Arc<Mutex<Connection>>) -> Self {
        let process_unshield = ProcessUnshield::new();
        UnshieldQueue {
            queue: Arc::new(Mutex::new(vec![])),
            redis_conn,
            process_unshield,
        }
    }

    async fn process_jobs(&self) {
        while let Some(job) = self.get_next_job().await {
            info!("Starting to process job: {}", job.id);
            for transaction in &job.transactions {
                let job_data = JobData {
                    input: transaction.input_bytes.clone(),
                    proof: transaction.proof_bytes.clone(),
                    action: transaction.action.clone(),
                    ext_data: transaction.ext_data_bytes.clone(),
                    owner: transaction.owner.clone(),
                };

                match self.process_unshield.process_unshield(job_data).await {
                    Ok(_) => info!("Successfully processed transaction {} for job {}", transaction.id, job.id),
                    Err(e) => error!("Failed to process transaction {}: {}", transaction.id, e),
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Simulate processing delay
            }
            info!("Job {} processed", job.id);
        }
    }

    async fn get_next_job(&self) -> Option<Job> {
        let mut lock = self.queue.lock().await;
        lock.pop()
    }
}

async fn setup_redis_connection() -> Arc<Mutex<Connection>> {
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").unwrap();
    let redis_username = env::var("REDIS_USERNAME").unwrap_or_else(|_| "default".to_string());
    let redis_password = env::var("REDIS_PASSWORD").unwrap();
    let client = Client::open(format!("redis://:{}@{}@{}", redis_password, redis_username, redis_url)).unwrap();
    let conn = client.get_async_connection().await.unwrap();
    Arc::new(Mutex::new(conn))
}

#[tokio::main]
async fn main() {
    let redis_conn = setup_redis_connection().await;
    let unshield_queue = UnshieldQueue::new(redis_conn).await;

    info!("Unshield service is running and waiting for jobs...");
    unshield_queue.process_jobs().await;
}
