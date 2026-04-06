use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::Value;

/// Represents an upgrade program instruction
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct HeliusParsedTransaction {
    /// Description of the instruction
    pub description: String,
    /// Type of instruction
    #[serde(rename = "type")]
    pub instruction_type: String,
    /// Source of the instruction
    pub source: String,
    /// Transaction fee
    pub fee: u64,
    /// Fee payer's address
    pub feePayer: String,
    /// Transaction signature
    pub signature: String,
    /// Blockchain slot number
    pub slot: u64,
    /// Transaction timestamp
    pub timestamp: u64,
    /// Token transfer details
    pub tokenTransfers: Vec<Value>,
    /// Native token transfer details
    pub nativeTransfers: Vec<Value>,
    /// Account data changes
    pub accountData: Vec<AccountData>,
    /// Transaction error if any
    pub transactionError: Option<String>,
    /// List of instructions in the transaction
    pub instructions: Vec<Instruction>,
    /// Associated events
    pub events: Value,
}

/// Represents account data changes in a transaction
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct AccountData {
    /// Account address
    pub account: String,
    /// Change in native token balance
    pub nativeBalanceChange: i64,
    /// Changes in token balances
    pub tokenBalanceChanges: Vec<Value>,
}

/// Represents an instruction in a transaction
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct Instruction {
    /// List of account addresses involved
    pub accounts: Vec<String>,
    /// Instruction data
    pub data: String,
    /// Program ID that processes this instruction
    pub programId: String,
    /// Inner instructions generated during execution
    pub innerInstructions: Vec<Value>,
}

/// Extracts and validates the upgrade instruction from the payload
pub fn parse_helius_transaction(
    payload: &[Value],
) -> Result<HeliusParsedTransaction, (StatusCode, &'static str)> {
    if payload.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Empty payload"));
    }

    serde_json::from_value(payload[0].clone()).map_err(|e| {
        tracing::error!("Failed to parse instruction payload: {}", e);
        (StatusCode::BAD_REQUEST, "Invalid payload")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// If this test fails, the Helius API may have changed the structure of the transaction payload.
    /// Review the deserialization of `HeliusParsedTransaction` against the current Helius API response.
    #[tokio::test]
    async fn test_parse_helius_transaction_from_api() {
        dotenv::dotenv().ok();
        let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");
        // Helius RPC URL: https://mainnet.helius-rpc.com/?api-key=KEY
        // Helius API URL: https://api.helius.xyz/v0/transactions/?api-key=KEY
        let url = rpc_url.replace("mainnet.helius-rpc.com/", "api.helius.xyz/v0/transactions/");

        let tx_sig = "31AUfFXG6BJQjaqwBsCjjZV5ojEL4zbrJ9gKQfKHDMosPvJKQBy6dKTiZgkkjoKbG1StD11csqgWn1KU5EwQsUgX";

        let client = reqwest::Client::new();
        let body = serde_json::json!({ "transactions": [tx_sig] });

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .expect("Failed to call Helius API");

        assert!(
            response.status().is_success(),
            "Helius API returned non-success status: {}",
            response.status()
        );

        let payload: Vec<Value> = response
            .json()
            .await
            .expect("Failed to deserialize Helius response");

        let parsed =
            parse_helius_transaction(&payload).expect("Failed to parse transaction payload");

        assert_eq!(parsed.signature, tx_sig);
        assert!(
            !parsed.instructions.is_empty(),
            "Expected at least one instruction"
        );
    }
}
