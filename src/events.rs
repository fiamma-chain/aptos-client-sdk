//! Event listener implementation
//!
//! Provides functionality to listen to Aptos Bridge contract events.

use crate::types::{constants::*, BridgeError, BridgeEvent, BridgeResult, BurnEvent, MintEvent};
use crate::utils::parse_account_address;

use aptos_sdk::{
    rest_client::Client,
    types::{account_address::AccountAddress, transaction::Version},
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle Mint event
    async fn handle_mint(&self, event: MintEvent) -> BridgeResult<()>;

    /// Handle Burn event
    async fn handle_burn(&self, event: BurnEvent) -> BridgeResult<()>;

    /// Handle error event
    async fn handle_error(&self, error: BridgeError) -> BridgeResult<()> {
        eprintln!("Event processing error: {}", error);
        Ok(())
    }
}

/// Event monitor
pub struct EventMonitor {
    /// REST client
    rest_client: Client,
    /// Bridge contract address
    contract_address: AccountAddress,
    /// Event handler
    handler: Box<dyn EventHandler>,
    /// Last processed version
    last_processed_version: Version,
    /// Batch size
    batch_size: u16,
    /// Confirmed blocks
    confirmed_blocks: u64,
    /// Poll interval
    poll_interval: std::time::Duration,
}

impl EventMonitor {
    /// Create new event monitor
    pub async fn new(
        node_url: &str,
        contract_address: &str,
        handler: Box<dyn EventHandler>,
        start_version: u64,
        batch_size: u16,
        poll_interval_secs: u64,
    ) -> BridgeResult<Self> {
        let rest_client = Client::new(
            Url::parse(node_url)
                .map_err(|e| BridgeError::Config(format!("Invalid node URL: {}", e)))?,
        );

        let contract_address = parse_account_address(contract_address)?;

        Ok(Self {
            rest_client,
            contract_address,
            handler,
            last_processed_version: start_version,
            batch_size,
            confirmed_blocks: 6,
            poll_interval: std::time::Duration::from_secs(poll_interval_secs),
        })
    }

    /// Start monitoring events
    pub async fn start_monitoring(&mut self) -> BridgeResult<()> {
        println!(
            "Starting event monitoring from version {}",
            self.last_processed_version
        );

        loop {
            match self.process_events().await {
                Ok(_) => {
                    tokio::time::sleep(self.poll_interval).await;
                }
                Err(e) => {
                    eprintln!("Error processing events: {}", e);
                    self.handler.handle_error(e).await?;
                    tokio::time::sleep(self.poll_interval).await;
                }
            }
        }
    }

    /// Process a batch of events
    async fn process_events(&mut self) -> BridgeResult<()> {
        // Get latest version
        let latest_version = self.get_latest_version().await?;

        if self.last_processed_version >= latest_version {
            return Ok(());
        }

        // Process events in batches
        let end_version = std::cmp::min(
            self.last_processed_version + self.batch_size as u64,
            latest_version,
        );

        let events = self
            .fetch_events(self.last_processed_version, end_version)
            .await?;

        for event in events {
            match event {
                BridgeEvent::Mint(mint_event) => {
                    self.handler.handle_mint(mint_event).await?;
                }
                BridgeEvent::Burn(burn_event) => {
                    self.handler.handle_burn(burn_event).await?;
                }
            }
        }

        self.last_processed_version = end_version;
        Ok(())
    }

    /// Get latest version from the network
    async fn get_latest_version(&self) -> BridgeResult<u64> {
        let ledger_info = self
            .rest_client
            .get_ledger_information()
            .await
            .map_err(|e| BridgeError::Aptos(e.to_string()))?;

        Ok(ledger_info.inner().version)
    }

    /// Fetch events from the network
    async fn fetch_events(
        &self,
        start_version: u64,
        end_version: u64,
    ) -> BridgeResult<Vec<BridgeEvent>> {
        let mut events = Vec::new();

        // This is a simplified implementation
        // In a real implementation, you would need to:
        // 1. Query transactions in the version range
        // 2. Filter for events from the bridge contract
        // 3. Parse the event data
        // 4. Convert to BridgeEvent instances

        // For now, return empty vector
        Ok(events)
    }
}

/// Default event handler implementation
pub struct DefaultEventHandler;

impl DefaultEventHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventHandler for DefaultEventHandler {
    async fn handle_mint(&self, event: MintEvent) -> BridgeResult<()> {
        println!(
            "🟢 Mint Event: {} to {}",
            crate::utils::format_btc_amount(event.amount),
            event.to
        );
        Ok(())
    }

    async fn handle_burn(&self, event: BurnEvent) -> BridgeResult<()> {
        println!(
            "🔴 Burn Event: {} from {} to {}",
            crate::utils::format_btc_amount(event.amount),
            event.from,
            event.btc_address
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ScriptType;

    #[test]
    fn test_default_event_handler() {
        let handler = DefaultEventHandler::new();
        // Add tests for event handler
    }
}
