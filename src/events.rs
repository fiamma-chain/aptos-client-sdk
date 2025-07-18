//! Event listener implementation
//!
//! Provides functionality to listen to Aptos Bridge contract events.

use crate::types::{parse_burn_event, parse_mint_event, BurnEventRaw, MintEventRaw};
use crate::{BridgeEvent, BurnEvent, MintEvent};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// GraphQL structures
#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: Option<Value>,
}

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<GraphQLData>,
    errors: Option<Vec<Value>>,
}

#[derive(Deserialize)]
struct GraphQLData {
    bridge_mint_events: Vec<MintEventRaw>,
    bridge_burn_events: Vec<BurnEventRaw>,
}

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_mint(&self, event: MintEvent) -> Result<()>;
    async fn handle_burn(&self, event: BurnEvent) -> Result<()>;
}

/// Event monitor
pub struct EventMonitor {
    graphql_url: String,
    graphql_api_key: String,
    handler: Box<dyn EventHandler>,
    last_processed_version: u64,
    query_client: crate::QueryClient,
}

impl EventMonitor {
    /// Create new event monitor
    pub fn new(
        graphql_url: &str,
        graphql_api_key: &str,
        node_url: &str,
        aptos_api_key: Option<&str>,
        handler: Box<dyn EventHandler>,
        last_processed_version: u64,
    ) -> Result<Self> {
        let query_client = crate::QueryClient::new(node_url, aptos_api_key)?;

        Ok(Self {
            graphql_url: graphql_url.to_string(),
            graphql_api_key: graphql_api_key.to_string(),
            handler,
            last_processed_version,
            query_client,
        })
    }

    /// Process events from given start version
    pub async fn process(&self) -> Result<Vec<BridgeEvent>> {
        let events = self.fetch_events(self.last_processed_version).await?;
        self.handle_events(&events).await?;
        Ok(events)
    }

    /// Fetch events from GraphQL
    async fn fetch_events(&self, start_version: u64) -> Result<Vec<BridgeEvent>> {
        let data = self.query_graphql(start_version).await?;

        let mut events = Vec::new();
        events.extend(self.process_mint_events(data.bridge_mint_events).await?);
        events.extend(self.process_burn_events(data.bridge_burn_events).await?);

        // Sort by version
        events.sort_by_key(|event| match event {
            BridgeEvent::Mint(e) => e.version.unwrap_or(0),
            BridgeEvent::Burn(e) => e.version.unwrap_or(0),
        });

        Ok(events)
    }

    /// Execute GraphQL query
    async fn query_graphql(&self, start_version: u64) -> Result<GraphQLData> {
        let query = r#"
            query GetBridgeEvents($startVersion: numeric!) {
                bridge_burn_events(where: {version: {_gt: $startVersion}}, order_by: {version: asc}) {
                    amount, btc_address, fee_rate, from_address, operator_id, timestamp, version
                }
                bridge_mint_events(where: {version: {_gt: $startVersion}}, order_by: {version: asc}) {
                    amount, btc_block_num, btc_tx_id, timestamp, to_address, version
                }
            }
        "#;

        let variables = serde_json::json!({ "startVersion": start_version });
        let request = GraphQLRequest {
            query: query.to_string(),
            variables: Some(variables),
        };

        let response = reqwest::Client::new()
            .post(&self.graphql_url)
            .header("Authorization", format!("Bearer {}", self.graphql_api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to send GraphQL request to {}: {}",
                    self.graphql_url,
                    e
                )
            })?
            .json::<GraphQLResponse>()
            .await
            .map_err(|e| anyhow!("Failed to parse GraphQL response: {}", e))?;

        if let Some(errors) = response.errors {
            return Err(anyhow::anyhow!("GraphQL errors: {:?}", errors));
        }

        let data = response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in GraphQL response"))?;

        Ok(data)
    }

    /// Process mint events
    async fn process_mint_events(&self, raw_events: Vec<MintEventRaw>) -> Result<Vec<BridgeEvent>> {
        let mut events = Vec::new();
        for raw in raw_events {
            let event = self.create_mint_event(raw).await?;
            events.push(event);
        }
        Ok(events)
    }

    /// Process burn events
    async fn process_burn_events(&self, raw_events: Vec<BurnEventRaw>) -> Result<Vec<BridgeEvent>> {
        let mut events = Vec::new();
        for raw in raw_events {
            let event = self.create_burn_event(raw).await?;
            events.push(event);
        }
        Ok(events)
    }

    /// Create mint event from raw data
    async fn create_mint_event(&self, raw: MintEventRaw) -> Result<BridgeEvent> {
        let mut event = parse_mint_event(&serde_json::to_value(&raw)?)?;

        if let Some(version) = event.version {
            match self.query_client.get_tx_hash_by_version(version).await {
                Ok(tx_hash) => {
                    event.transaction_hash = Some(tx_hash);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to get transaction hash for version {}: {}",
                        version, e
                    );
                }
            }
        }

        Ok(BridgeEvent::Mint(event))
    }

    /// Create burn event from raw data
    async fn create_burn_event(&self, raw: BurnEventRaw) -> Result<BridgeEvent> {
        let mut event = parse_burn_event(&serde_json::to_value(&raw)?)?;

        if let Some(version) = event.version {
            match self.query_client.get_tx_hash_by_version(version).await {
                Ok(tx_hash) => {
                    event.transaction_hash = Some(tx_hash);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to get transaction hash for version {}: {}",
                        version, e
                    );
                }
            }
        }

        Ok(BridgeEvent::Burn(event))
    }

    /// Handle all events
    async fn handle_events(&self, events: &[BridgeEvent]) -> Result<()> {
        for event in events {
            match event {
                BridgeEvent::Mint(mint_event) => {
                    self.handler.handle_mint(mint_event.clone()).await?
                }
                BridgeEvent::Burn(burn_event) => {
                    self.handler.handle_burn(burn_event.clone()).await?
                }
            }
        }
        Ok(())
    }
}
