//! Event listener example
//!
//! This example shows how to use the Aptos Bridge SDK to listen to bridge events.

use anyhow::Result;
use aptos_bridge_sdk::{
    types::{BurnEvent, MintEvent},
    EventHandler, EventMonitor,
};
use async_trait::async_trait;
use std::sync::Arc;

struct CustomEventHandler;

#[async_trait]
impl EventHandler for CustomEventHandler {
    async fn handle_mint(
        &self,
        mint_version: u64,
        mint_sequence_number: u64,
        event: MintEvent,
    ) -> Result<()> {
        let event_data = format!(
            "Mint Event To: {}, Amount: {}, Block: {}, TxHash: {}, Timestamp: {}",
            event.to, event.amount, event.block_num, mint_version, mint_sequence_number,
        );

        println!("🟢 {}", event_data);

        Ok(())
    }

    async fn handle_burn(
        &self,
        burn_version: u64,
        burn_sequence_number: u64,
        event: BurnEvent,
    ) -> Result<()> {
        let event_data = format!(
            "Burn Event - From: {}, To: {}, Amount: {}, FeeRate: {}, Operator: {}, TxHash: {}, Timestamp: {}",
            event.from,
            event.btc_address,
            event.amount,
            event.fee_rate,
            event.operator_id,
            burn_version,
            burn_sequence_number,
        );

        println!("🔴 {}", event_data);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    println!("👂 Aptos Bridge Event Listener Example");

    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let bridge_contract_address =
        "0x2e5df32d3db81510b01dc0ec2fd6220b43b29b1e2a98b48a013a774f10726e5b";
    let mint_start = 0;
    let burn_start = 0;

    let poll_interval = 3;

    // Create event monitor
    let monitor = EventMonitor::new(
        &node_url,
        &bridge_contract_address,
        Box::new(CustomEventHandler),
        mint_start,
        burn_start,
    )?;

    let monitor = Arc::new(monitor);

    loop {
        match monitor.process().await {
            Ok(_) => {
                println!("📊 Processed events successfully");
            }
            Err(e) => {
                eprintln!("❌ Error processing events: {}", e);
            }
        }

        // Wait before next poll
        tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval)).await;
    }
}
