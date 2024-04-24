// src/api/add_jobs.rs
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use crate::services::redis::UnshieldQueue;  // Assume we have a way to reference the queue
use crate::helper::{to_uint_array, public_inputs_bytes_to_object};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobData {
    input: Vec<u8>,
    proof: Vec<u8>,
    action: String,
    ext_data: Vec<u8>,
    owner: String,
}

impl JobData {
    pub fn new(input: Vec<u8>, proof: Vec<u8>, action: String, ext_data: Vec<u8>, owner: String) -> Self {
        JobData {
            input,
            proof,
            action,
            ext_data,
            owner,
        }
    }
}

pub async fn add_unshield_job(queue: Arc<UnshieldQueue>, job_data: JobData) -> Result<(), Box<dyn std::error::Error>> {
    // Example: Deserialize ext_data if it's expected to be in a specific format
    let public_inputs = public_inputs_bytes_to_object(&job_data.ext_data);

    // Modify job_data with potentially transformed data
    let updated_job_data = JobData {
        input: job_data.input,
        proof: job_data.proof,
        action: job_data.action,
        ext_data: to_uint_array(job_data.ext_data),
        owner: job_data.owner,
    };

    // Enqueue the potentially modified job data
    queue.enqueue(updated_job_data).await?;
    Ok(())
}