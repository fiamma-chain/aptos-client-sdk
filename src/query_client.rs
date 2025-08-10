//! Query client implementation
//!
//! Provides functionality to query Aptos Bridge contract configuration and status.

use crate::types::{BridgeEvent, BurnEventBCS, MintEventBCS, WithdrawByLPEventBCS};
use anyhow::{anyhow, Result};
use aptos_sdk::{
    crypto::HashValue,
    rest_client::{aptos_api_types::TransactionData, AptosBaseUrl, Client, ClientBuilder},
    types::{account_address::AccountAddress, contract_event::ContractEvent},
};

use std::str::FromStr;
use url::Url;
/// Query client
pub struct QueryClient {
    /// REST client
    rest_client: Client,
}

impl QueryClient {
    /// Create new query client
    pub fn new(node_url: &str, aptos_api_key: Option<&str>) -> Result<Self> {
        let mut client_builder = ClientBuilder::new(AptosBaseUrl::Custom(
            Url::parse(node_url).map_err(|e| anyhow!("Invalid node URL '{}': {}", node_url, e))?,
        ));

        if let Some(api_key) = aptos_api_key {
            client_builder = client_builder.api_key(api_key)?;
        }

        let rest_client = client_builder.build();

        Ok(Self { rest_client })
    }

    /// Query transaction status
    pub async fn get_transaction_by_hash(&self, tx_hash: &str) -> Result<TransactionData> {
        // Parse transaction hash
        let tx_hash = HashValue::from_hex(tx_hash.trim_start_matches("0x"))
            .map_err(|e| anyhow!("Invalid transaction hash '{}': {}", tx_hash, e))?;

        let response = self
            .rest_client
            .get_transaction_by_hash_bcs(tx_hash)
            .await
            .map_err(|e| anyhow!("Failed to get transaction from Aptos node: {}", e))?;

        Ok(response.inner().clone())
    }

    pub async fn get_tx_hash_by_version(&self, version: u64) -> Result<String> {
        let response = self
            .rest_client
            .get_transaction_by_version_bcs(version)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to get aptos transaction by version {}: {}",
                    version,
                    e
                )
            })?;

        match response.into_inner() {
            TransactionData::OnChain(txn) => Ok(txn.info.transaction_hash().to_hex_literal()),
            other => Err(anyhow!(
                "Transaction at version {} is not on-chain (got {:?})",
                version,
                other
            )),
        }
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
            TransactionData::OnChain(txn) => txn.events,
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

    /// Parse a single event to check if it's a bridge event using BCS directly
    fn parse_bridge_event(
        &self,
        event: &ContractEvent,
        bridge_contract_address: &str,
    ) -> Result<Option<BridgeEvent>> {
        let event_type_tag = event.type_tag();
        let event_type_str = event_type_tag.to_canonical_string();

        // Parse and normalize contract addresses using Aptos SDK
        let expected_addr = AccountAddress::from_str(bridge_contract_address)
            .map_err(|e| anyhow!("Invalid bridge contract address: {}", e))?;

        // Extract contract address from event type
        let event_addr_str = if let Some(pos) = event_type_str.find("::") {
            &event_type_str[..pos]
        } else {
            return Ok(None);
        };

        let event_addr = AccountAddress::from_str(event_addr_str)
            .map_err(|e| anyhow!("Invalid event contract address: {}", e))?;

        // Compare normalized addresses
        if event_addr != expected_addr {
            return Ok(None);
        }

        let event_data = event.event_data();

        // Parse BCS event data directly based on event type
        let bridge_event = if event_type_str.ends_with("::bridge::Mint") {
            let mint_bcs: MintEventBCS = bcs::from_bytes(event_data).map_err(|e| {
                anyhow!(
                    "Failed to deserialize mint event data: {} (type: {})",
                    e,
                    event_type_str
                )
            })?;
            BridgeEvent::Mint(mint_bcs.into())
        } else if event_type_str.ends_with("::bridge::Burn") {
            let burn_bcs: BurnEventBCS = bcs::from_bytes(event_data).map_err(|e| {
                anyhow!(
                    "Failed to deserialize burn event data: {} (type: {})",
                    e,
                    event_type_str
                )
            })?;
            BridgeEvent::Burn(burn_bcs.into())
        } else if event_type_str.ends_with("::bridge::WithdrawByLP") {
            let withdraw_bcs: WithdrawByLPEventBCS = bcs::from_bytes(event_data).map_err(|e| {
                anyhow!(
                    "Failed to deserialize withdraw event data: {} (type: {})",
                    e,
                    event_type_str
                )
            })?;
            BridgeEvent::WithdrawByLP(withdraw_bcs.into())
        } else {
            return Ok(None);
        };

        Ok(Some(bridge_event))
    }
}
