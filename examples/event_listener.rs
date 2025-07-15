//! Event listener example
//!
//! This example shows how to use the Aptos Bridge SDK to listen to bridge events.

use anyhow::{Context, Result};
use aptos_bridge_sdk::{
    types::{BurnEvent, MintEvent},
    EventHandler, EventMonitor,
};
use async_trait::async_trait;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Custom event handler
pub struct CustomEventHandler {
    /// Event counter
    event_count: Arc<Mutex<u64>>,
    /// Whether to save events to file
    save_to_file: bool,
    /// Output file path
    output_file: Option<String>,
}

impl CustomEventHandler {
    pub fn new(save_to_file: bool, output_file: Option<String>) -> Self {
        Self {
            event_count: Arc::new(Mutex::new(0)),
            save_to_file,
            output_file,
        }
    }

    /// Get the number of processed events
    pub async fn get_event_count(&self) -> u64 {
        *self.event_count.lock().await
    }

    /// Save event to file
    async fn save_event_to_file(&self, event_data: &str) -> Result<()> {
        if !self.save_to_file {
            return Ok(());
        }

        if let Some(file_path) = &self.output_file {
            use tokio::fs::OpenOptions;
            use tokio::io::AsyncWriteExt;

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)
                .await
                .with_context(|| format!("Failed to open file: {}", file_path))?;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let log_entry = format!("[{}] {}\n", timestamp, event_data);

            file.write_all(log_entry.as_bytes())
                .await
                .context("Failed to write to file")?;

            file.flush().await.context("Failed to flush file")?;
        }

        Ok(())
    }

    /// Send notification (placeholder for actual notification system)
    async fn send_notification(&self, event_type: &str, message: &str) -> Result<()> {
        // This is a placeholder for actual notification implementation
        // You can implement webhook calls, email sending, etc.
        println!("🔔 Notification [{}]: {}", event_type, message);
        Ok(())
    }
}

#[async_trait]
impl EventHandler for CustomEventHandler {
    async fn handle_mint(
        &self,
        mint_version: u64,
        mint_sequence_number: u64,
        event: MintEvent,
    ) -> Result<()> {
        let mut count = self.event_count.lock().await;
        *count += 1;

        let event_data = format!(
            "Mint Event #{} - To: {}, Amount: {}, Block: {}, TxHash: {}, Timestamp: {}",
            *count, event.to, event.amount, event.block_num, mint_version, mint_sequence_number,
        );

        println!("🟢 {}", event_data);

        // Save to file if enabled
        self.save_event_to_file(&event_data).await?;

        // Send notification
        self.send_notification("MINT", &event_data).await?;

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

        // Save to file if enabled
        self.save_event_to_file(&event_data).await?;

        // Send notification
        self.send_notification("BURN", &event_data).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("👂 Aptos Bridge Event Listener Example");

    // Get configuration from environment variables
    let node_url = env::var("APTOS_NODE_URL")
        .unwrap_or_else(|_| "https://fullnode.devnet.aptoslabs.com/v1".to_string());
    let bridge_contract_address = env::var("BRIDGE_CONTRACT_ADDRESS")
        .expect("BRIDGE_CONTRACT_ADDRESS environment variable is required");
    let output_file = env::var("OUTPUT_FILE").ok();
    let save_to_file = output_file.is_some();
    let start_version = 0;
    let _batch_size = env::var("BATCH_SIZE")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u16>()
        .unwrap_or(10);
    let poll_interval = env::var("POLL_INTERVAL")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u64>()
        .unwrap_or(5);

    println!("Configuration:");
    println!("  Node URL: {}", node_url);
    println!("  Bridge Contract: {}", bridge_contract_address);
    println!("  Start Version: {}", start_version);
    println!("  Poll Interval: {}s", poll_interval);
    if let Some(ref file) = output_file {
        println!("  Output File: {}", file);
    }
    println!();

    // Create custom event handler
    let handler = CustomEventHandler::new(save_to_file, output_file);

    // Create event monitor
    let monitor = EventMonitor::new(
        &node_url,
        &bridge_contract_address,
        Box::new(handler),
        start_version,
        start_version,
    )?;

    println!("✅ Event monitor created");
    println!("🔄 Starting event monitoring...");
    println!("Press Ctrl+C to stop");
    println!();

    // Set up Ctrl+C handler for graceful shutdown
    let monitor = Arc::new(monitor);

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        println!("\n👋 Received Ctrl+C, shutting down gracefully...");
        std::process::exit(0);
    });

    // Start monitoring loop
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
