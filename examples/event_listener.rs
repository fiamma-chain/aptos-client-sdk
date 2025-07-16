//! Event listener example
//!
//! This example shows how to use the Aptos Bridge SDK to listen to bridge events.

use anyhow::Result;
use aptos_bridge_sdk::{
    types::{BridgeBurnEvent, BridgeMintEvent},
    EventHandler, EventMonitor,
};
use async_trait::async_trait;
use std::env;

struct CustomEventHandler;

#[async_trait]
impl EventHandler for CustomEventHandler {
    async fn handle_mint(&self, event: BridgeMintEvent) -> Result<()> {
        let event_data = format!(
            "🟢 Mint Event - To: {}, Amount: {}, Block: {}, Version: {}, Timestamp: {}",
            event.event.to,
            event.event.amount,
            event.event.block_num,
            event.tx_version,
            event.timestamp,
        );

        println!("{}", event_data);

        Ok(())
    }

    async fn handle_burn(&self, event: BridgeBurnEvent) -> Result<()> {
        let event_data = format!(
            "🔴 Burn Event - From: {}, To: {}, Amount: {}, FeeRate: {}, Operator: {}, Version: {}, Timestamp: {}",
            event.event.from,
            event.event.btc_address,
            event.event.amount,
            event.event.fee_rate,
            event.event.operator_id,
            event.tx_version,
            event.timestamp,
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
        "https://api.testnet.aptoslabs.com/nocode/v1/api/cmd62f87p006ks601xpbky5mx/v1/graphql";
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
