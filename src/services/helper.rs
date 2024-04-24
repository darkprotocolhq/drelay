// helper.rs
use hex::ToHex;
use rand::{distributions::Alphanumeric, Rng};
use tokio::time::{self, Duration};
use ethers::types::U256;

/// Converts a byte slice to a `Vec<u8>`.
pub fn to_uint_array(value: &[u8]) -> Vec<u8> {
    value.to_vec()
}

/// Generates a random nonce of specified length.
pub fn new_nonce(length: usize) -> Vec<u8> {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)  // Example: nacl.box.nonceLength could be 24
        .collect()
}

/// Converts a byte array to a hex string.
pub fn to_hex_string(bytes: &[u8]) -> String {
    bytes.encode_hex::<String>()
}

/// Rounds a floating point number to a specified number of decimal places.
pub fn round_to(amount: f64, decimals: u32) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (amount * factor).round() / factor
}

/// Asynchronous sleep function.
pub async fn sleep(ms: u64) {
    time::sleep(Duration::from_millis(ms)).await;
}

/// Converts a `U256` number or a buffer to a fixed-length hex string.
pub fn to_fixed_hex(number: U256, length: usize) -> String {
    let mut hex = format!("{:x}", number); // Converts to hex without 0x prefix
    hex = format!("0x{}", hex);
    hex = format!("{:0>width$}", hex, width = length * 2 + 2); // Pad with zeros
    hex
}

/// Decodes a private key from a comma-separated string of bytes.
pub fn decoded_privkey(privkey_bytes: &str) -> Vec<u8> {
    privkey_bytes.split(',')
        .map(|b| b.parse::<u8>().unwrap_or(0))
        .collect()
}

/// Structure for public input bytes.
pub struct PublicInputs {
    pub recipient: Vec<u8>,
    pub ext_amount: Vec<u8>,
    pub relayer: Vec<u8>,
    pub fee: Vec<u8>,
    pub merkle_tree_pubkey_bytes: Vec<u8>,
    pub merkle_tree_index: u8,
    pub encrypted_output1: Vec<u8>,
    pub nonce1: Vec<u8>,
    pub sender_throw_away_pubkey1: Vec<u8>,
    pub encrypted_output2: Vec<u8>,
    pub nonce2: Vec<u8>,
    pub sender_throw_away_pubkey2: Vec<u8>,
}

/// Converts a byte slice into a `PublicInputs` object.
pub fn public_inputs_bytes_to_object(inputs: &[u8]) -> PublicInputs {
    PublicInputs {
        recipient: inputs[0..32].to_vec(),
        ext_amount: inputs[32..40].to_vec(),
        relayer: inputs[40..72].to_vec(),
        fee: inputs[72..80].to_vec(),
        merkle_tree_pubkey_bytes: inputs[80..112].to_vec(),
        merkle_tree_index: inputs[112],
        encrypted_output1: inputs[113..168].to_vec(),
        nonce1: inputs[168..192].to_vec(),
        sender_throw_away_pubkey1: inputs[192..224].to_vec(),
        encrypted_output2: inputs[224..279].to_vec(),
        nonce2: inputs[279..303].to_vec(),
        sender_throw_away_pubkey2: inputs[303..335].to_vec(),
    }
}
