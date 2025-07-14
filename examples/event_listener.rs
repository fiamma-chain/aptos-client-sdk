//! Event listener example
//!
//! This example shows how to use the Aptos Bridge SDK to listen to bridge events.

use aptos_bridge_sdk::{
    types::{BridgeError, BridgeResult, BurnEvent, MintEvent},
    utils::{format_btc_amount, timestamp_to_string},
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
    async fn save_event_to_file(&self, event_data: &str) -> BridgeResult<()> {
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
                .map_err(|e| BridgeError::Other(format!("Failed to open file: {}", e)))?;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let log_entry = format!("[{}] {}\n", timestamp, event_data);

            file.write_all(log_entry.as_bytes())
                .await
                .map_err(|e| BridgeError::Other(format!("Failed to write to file: {}", e)))?;

            file.flush()
                .await
                .map_err(|e| BridgeError::Other(format!("Failed to flush file: {}", e)))?;
        }

        Ok(())
    }

    /// Send notification (placeholder for actual notification system)
    async fn send_notification(&self, event_type: &str, message: &str) -> BridgeResult<()> {
        // This is a placeholder for actual notification implementation
        // You can implement webhook calls, email sending, etc.
        println!("🔔 Notification [{}]: {}", event_type, message);
        Ok(())
    }
}

#[async_trait]
impl EventHandler for CustomEventHandler {
    async fn handle_mint(&self, event: MintEvent) -> BridgeResult<()> {
        let mut count = self.event_count.lock().await;
        *count += 1;

        let event_data = format!(
            "Mint Event #{} - To: {}, Amount: {}, Block: {}, TxHash: {}, Timestamp: {}",
            *count,
            event.to,
            format_btc_amount(event.amount),
            event.block_num,
            event.transaction_hash,
            timestamp_to_string(event.block_timestamp)
        );

        println!("🟢 {}", event_data);

        // Save to file if enabled
        self.save_event_to_file(&event_data).await?;

        // Send notification
        self.send_notification(
            "MINT",
            &format!("Minted {} to {}", format_btc_amount(event.amount), event.to),
        )
        .await?;

        // Additional processing based on event data
        if event.amount >= 100_000_000 {
            println!(
                "🚨 Large mint detected: {}",
                format_btc_amount(event.amount)
            );
        }

        Ok(())
    }

    async fn handle_burn(&self, event: BurnEvent) -> BridgeResult<()> {
        let mut count = self.event_count.lock().await;
        *count += 1;

        let event_data = format!(
            "Burn Event #{} - From: {}, To: {}, Amount: {}, FeeRate: {}, Operator: {}, TxHash: {}, Timestamp: {}",
            *count,
            event.from,
            event.btc_address,
            format_btc_amount(event.amount),
            event.fee_rate,
            event.operator_id,
            event.transaction_hash,
            timestamp_to_string(event.block_timestamp)
        );

        println!("🔴 {}", event_data);

        // Save to file if enabled
        self.save_event_to_file(&event_data).await?;

        // Send notification
        self.send_notification(
            "BURN",
            &format!(
                "Burned {} from {} to {}",
                format_btc_amount(event.amount),
                event.from,
                event.btc_address
            ),
        )
        .await?;

        // Additional processing based on event data
        if event.fee_rate > 1000 {
            println!("⚠️  High fee rate detected: {}", event.fee_rate);
        }

        Ok(())
    }

    async fn handle_error(&self, error: BridgeError) -> BridgeResult<()> {
        let error_message = format!("Event processing error: {}", error);
        eprintln!("❌ {}", error_message);

        // Save error to file if enabled
        self.save_event_to_file(&error_message).await?;

        // Send error notification
        self.send_notification("ERROR", &error_message).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("👂 Aptos Bridge Event Listener Example");

    // Get configuration from environment variables
    let node_url = env::var("APTOS_NODE_URL")
        .unwrap_or_else(|_| "https://fullnode.devnet.aptoslabs.com/v1".to_string());
    let bridge_contract_address = env::var("BRIDGE_CONTRACT_ADDRESS")
        .expect("BRIDGE_CONTRACT_ADDRESS environment variable is required");
    let output_file = env::var("OUTPUT_FILE").ok();
    let save_to_file = output_file.is_some();
    let start_version = env::var("START_VERSION")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<u64>()
        .unwrap_or(0);
    let batch_size = env::var("BATCH_SIZE")
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
    println!("  Batch Size: {}", batch_size);
    println!("  Poll Interval: {}s", poll_interval);
    if let Some(ref file) = output_file {
        println!("  Output File: {}", file);
    }
    println!();

    // Create custom event handler
    let handler = CustomEventHandler::new(save_to_file, output_file);

    // Create event monitor
    let mut monitor = EventMonitor::new(
        &node_url,
        &bridge_contract_address,
        Box::new(handler),
        start_version,
        batch_size,
        poll_interval,
    )
    .await?;

    println!("✅ Event monitor created");
    println!("🔄 Starting event monitoring...");
    println!("Press Ctrl+C to stop");
    println!();

    // Create event statistics tracker
    let stats = Arc::new(Mutex::new(EventStatistics::new()));

    // Start monitoring in a separate task
    let stats_clone = stats.clone();
    let monitor_task = tokio::spawn(async move {
        match monitor.start_monitoring().await {
            Ok(_) => println!("✅ Event monitoring completed"),
            Err(e) => eprintln!("❌ Event monitoring failed: {}", e),
        }
    });

    // Start periodic statistics reporting
    let stats_clone2 = stats.clone();
    let stats_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let stats = stats_clone2.lock().await;
            stats.print_statistics();
        }
    });

    // Handle Ctrl+C gracefully
    let shutdown_task = tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        println!("\n🛑 Shutdown signal received, stopping event monitoring...");

        // Print final statistics
        let stats = stats.lock().await;
        println!("\n📊 Final Statistics:");
        stats.print_statistics();

        std::process::exit(0);
    });

    // Wait for any of the tasks to complete
    tokio::select! {
        _ = monitor_task => {},
        _ = stats_task => {},
        _ = shutdown_task => {},
    }

    Ok(())
}

/// Event statistics tracker
struct EventStatistics {
    total_events: u64,
    mint_events: u64,
    burn_events: u64,
    total_mint_amount: u64,
    total_burn_amount: u64,
    start_time: std::time::Instant,
}

impl EventStatistics {
    fn new() -> Self {
        Self {
            total_events: 0,
            mint_events: 0,
            burn_events: 0,
            total_mint_amount: 0,
            total_burn_amount: 0,
            start_time: std::time::Instant::now(),
        }
    }

    fn record_mint(&mut self, amount: u64) {
        self.total_events += 1;
        self.mint_events += 1;
        self.total_mint_amount += amount;
    }

    fn record_burn(&mut self, amount: u64) {
        self.total_events += 1;
        self.burn_events += 1;
        self.total_burn_amount += amount;
    }

    fn print_statistics(&self) {
        let runtime = self.start_time.elapsed();
        let hours = runtime.as_secs() / 3600;
        let minutes = (runtime.as_secs() % 3600) / 60;
        let seconds = runtime.as_secs() % 60;

        println!(
            "📊 Event Statistics ({}h {}m {}s):",
            hours, minutes, seconds
        );
        println!("  Total Events: {}", self.total_events);
        println!(
            "  Mint Events: {} ({})",
            self.mint_events,
            format_btc_amount(self.total_mint_amount)
        );
        println!(
            "  Burn Events: {} ({})",
            self.burn_events,
            format_btc_amount(self.total_burn_amount)
        );
        println!();
    }
}

fn print_usage() {
    println!("📖 Usage:");
    println!("Set the following environment variables:");
    println!("  APTOS_NODE_URL=https://fullnode.devnet.aptoslabs.com/v1");
    println!("  BRIDGE_CONTRACT_ADDRESS=contract_address_here");
    println!("  OUTPUT_FILE=events.log (optional)");
    println!("  START_VERSION=0 (optional)");
    println!("  BATCH_SIZE=10 (optional)");
    println!("  POLL_INTERVAL=5 (optional)");
    println!("\nExample:");
    println!("  export BRIDGE_CONTRACT_ADDRESS=0x123...");
    println!("  export OUTPUT_FILE=bridge_events.log");
    println!("  export START_VERSION=1000");
    println!("  cargo run --example event_listener");
}
