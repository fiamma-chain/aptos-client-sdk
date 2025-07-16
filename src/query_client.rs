//! Query client implementation
//!
//! Provides functionality to query Aptos Bridge contract configuration and status.

use crate::{
    types::{parse_burn_event, parse_mint_event, BridgeEvent},
    BridgeBurnEvent, BridgeMintEvent,
};
use anyhow::{anyhow, Context, Result};
use aptos_sdk::{
    crypto::HashValue,
    rest_client::{aptos_api_types::Event, Client, Transaction},
};
use url::Url;

/// Query client
pub struct QueryClient {
    /// REST client
    rest_client: Client,
}

impl QueryClient {
    /// Create new query client
    pub fn new(node_url: &str) -> Result<Self> {
        let rest_client = Client::new(
            Url::parse(node_url).with_context(|| format!("Invalid node URL: {}", node_url))?,
        );

        Ok(Self { rest_client })
    }

    /// Query transaction status
    pub async fn get_transaction_by_hash(&self, tx_hash: &str) -> Result<Transaction> {
        // Parse transaction hash
        let tx_hash = HashValue::from_hex(tx_hash.trim_start_matches("0x"))
            .with_context(|| format!("Invalid transaction hash: {}", tx_hash))?;

        let response = self
            .rest_client
            .get_transaction_by_hash(tx_hash)
            .await
            .context("Failed to get transaction from Aptos node")?;

        Ok(response.inner().clone())
    }

    /// Get bridge events from user transaction hash
    pub async fn get_bridge_events_by_hash(
        &self,
        tx_hash: &str,
        bridge_contract_address: &str,
    ) -> Result<Vec<BridgeEvent>> {
        // Get transaction details
        let transaction = self.get_transaction_by_hash(tx_hash).await?;

        // Only process user transactions
        let events = match transaction {
            Transaction::UserTransaction(user_tx) => user_tx.events,
            _ => {
                return Err(anyhow!(
                    "Transaction {} is not a user transaction. Only user and not pending transactions are supported.",
                    tx_hash
                ));
            }
        };

        let mut bridge_events = Vec::new();

        // Parse each event
        for event in &events {
            if let Some(bridge_event) = self.parse_bridge_event(event, bridge_contract_address)? {
                bridge_events.push(bridge_event);
            }
        }

        Ok(bridge_events)
    }

    /// Parse a single event to check if it's a bridge event
    fn parse_bridge_event(
        &self,
        event: &Event,
        bridge_contract_address: &str,
    ) -> Result<Option<BridgeEvent>> {
        let event_type = &event.typ.to_string();

        // Check if this event is from our bridge contract
        if !event_type.starts_with(&format!("{}::", bridge_contract_address)) {
            return Ok(None);
        }

        // Parse Mint events
        if event_type.ends_with("::bridge::Mint") {
            let mint_event = parse_mint_event(&event.data)?;

            return Ok(Some(BridgeEvent::Mint(BridgeMintEvent {
                tx_version: 0,
                timestamp: 0,
                event: mint_event,
            })));
        }

        // Parse Burn events
        if event_type.ends_with("::bridge::Burn") {
            let burn_event = parse_burn_event(&event.data)?;

            return Ok(Some(BridgeEvent::Burn(BridgeBurnEvent {
                tx_version: 0,
                timestamp: 0,
                event: burn_event,
            })));
        }

        // Not a bridge event we're interested in
        Ok(None)
    }
}
