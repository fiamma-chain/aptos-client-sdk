//! Event listener example
//!
//! This example shows how to use the Aptos Bridge SDK to listen to bridge events.

use anyhow::Result;
use aptos_bridge_sdk::{BurnEvent, EventHandler, EventMonitor, MintEvent};
use async_trait::async_trait;
use std::env;

struct CustomEventHandler;

#[async_trait]
impl EventHandler for CustomEventHandler {
    async fn handle_mint(&self, event: MintEvent) -> Result<()> {
        let event_data = format!(
            "🟢 Mint Event - To: {}, Amount: {}, BTC Block: {}, BTC Tx: {}, Version: {}, Timestamp: {}",
            event.to_address,
            event.amount,
            event.btc_block_num,
            event.btc_tx_id,
            event.version.unwrap_or(0),
            event.timestamp.unwrap_or(0),
        );

        println!("{}", event_data);

        Ok(())
    }

    async fn handle_burn(&self, event: BurnEvent) -> Result<()> {
        let event_data = format!(
            "🔴 Burn Event - From: {}, To: {}, Amount: {}, FeeRate: {}, Operator: {}, Version: {}, Timestamp: {}",
            event.from_address,
            event.btc_address,
            event.amount,
            event.fee_rate,
            event.operator_id,
            event.version.unwrap_or(0),
            event.timestamp.unwrap_or(0),
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
        "https://api.testnet.aptoslabs.com/nocode/v1/api/cmd66memj007os601224cvlmd/v1/graphql";
    let start_version = 0;
    let poll_interval = 10; // seconds

    // Get API key from environment variable
    let api_key =
        env::var("GRAPHQL_API_KEY").expect("GRAPHQL_API_KEY environment variable is required");

    // Create event monitor with API key
    let monitor = EventMonitor::new(
        graphql_url,
        &api_key,
        Box::new(CustomEventHandler),
        start_version,
    );
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
