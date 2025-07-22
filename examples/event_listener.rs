//! Event listener example
//!
//! This example shows how to use the Aptos Bridge SDK to listen to bridge events.

use anyhow::Result;
use aptos_client_sdk::{BurnEvent, EventHandler, EventMonitor, MintEvent, WithdrawByLPEvent};
use async_trait::async_trait;
use std::env;

struct CustomEventHandler;

#[async_trait]
impl EventHandler for CustomEventHandler {
    async fn handle_mint(&self, event: MintEvent) -> Result<()> {
        let event_data = format!(
            "🟢 Mint Event - To: {}, Amount: {}, BTC Block: {}, BTC Tx: {}, Version: {}, Timestamp: {}, Transaction Hash: {}",
            event.to_address,
            event.amount,
            event.btc_block_num,
            event.btc_tx_id,
            event.version.unwrap_or(0),
            event.timestamp.unwrap_or(0),
            event.transaction_hash.unwrap_or("N/A".to_string()),
        );

        println!("{}", event_data);

        Ok(())
    }

    async fn handle_burn(&self, event: BurnEvent) -> Result<()> {
        let event_data = format!(
            "🔴 Burn Event - From: {}, To: {}, Amount: {}, FeeRate: {}, Operator: {}, Version: {}, Timestamp: {}, Transaction Hash: {}",
            event.from_address,
            event.btc_address,
            event.amount,
            event.fee_rate,
            event.operator_id,
            event.version.unwrap_or(0),
            event.timestamp.unwrap_or(0),
            event.transaction_hash.unwrap_or("N/A".to_string()),
        );

        println!("{}", event_data);

        Ok(())
    }

    async fn handle_withdraw_by_lp(&self, event: WithdrawByLPEvent) -> Result<()> {
        let event_data = format!(
            "🟡 WithdrawByLP Event - From: {}, Withdraw ID: {}, Amount: {}, BTC Address: {}, LP ID: {}, Fee Rate: {}, Min Receive: {}, Version: {}, Timestamp: {}, Transaction Hash: {}",
            event.from_address,
            event.withdraw_id,
            event.amount,
            event.btc_address,
            event.lp_id,
            event.fee_rate,
            event.receive_min_amount,
            event.version.unwrap_or(0),
            event.timestamp.unwrap_or(0),
            event.transaction_hash.unwrap_or("N/A".to_string()),
        );

        println!("{}", event_data);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    println!("👂 Aptos Bridge Event Listener Example");

    // Use your custom GraphQL index URL
    let graphql_url =
        "https://api.testnet.aptoslabs.com/nocode/v1/api/cmde2pbrh0011s601ew8vlbgd/v1/graphql";
    let start_version = 0;
    let poll_interval = 10; // seconds

    // Get API keys from environment variables
    let graphql_api_key =
        env::var("GRAPHQL_API_KEY").expect("GRAPHQL_API_KEY environment variable is required");
    let aptos_api_key = env::var("APTOS_API_KEY").ok();

    // Aptos node URL
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";

    // Create event monitor with API key
    let monitor = EventMonitor::new(
        graphql_url,
        &graphql_api_key,
        node_url,
        aptos_api_key.as_deref(),
        Box::new(CustomEventHandler),
        start_version,
    )?;
    loop {
        match monitor.process().await {
            Ok(_) => {
                println!("✅ Processed events successfully");
            }
            Err(e) => {
                eprintln!("❌ Error processing events: {}", e);
            }
        }

        // Wait before next poll
        tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval)).await;
    }
}
