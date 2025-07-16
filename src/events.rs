//! Event listener implementation
//!
//! Provides functionality to listen to Aptos Bridge contract events.

use crate::types::{
    parse_burn_event, parse_mint_event, BridgeBurnEvent, BridgeMintEvent, BurnEventRaw,
    MintEventRaw,
};
use crate::BridgeEvent;

use anyhow::{Context, Result};
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
    async fn handle_mint(&self, event: BridgeMintEvent) -> Result<()>;
    async fn handle_burn(&self, event: BridgeBurnEvent) -> Result<()>;
}

/// Event monitor
pub struct EventMonitor {
    graphql_url: String,
    api_key: String,
    handler: Box<dyn EventHandler>,
    last_processed_version: u64,
}

impl EventMonitor {
    /// Create new event monitor
    pub fn new(
        graphql_url: &str,
        api_key: &str,
        handler: Box<dyn EventHandler>,
        last_processed_version: u64,
    ) -> Self {
        Self {
            graphql_url: graphql_url.to_string(),
            api_key: api_key.to_string(),
            handler,
            last_processed_version,
        }
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
        events.extend(self.process_mint_events(data.bridge_mint_events)?);
        events.extend(self.process_burn_events(data.bridge_burn_events)?);

        // Sort by version
        events.sort_by_key(|event| match event {
            BridgeEvent::Mint(e) => e.tx_version,
            BridgeEvent::Burn(e) => e.tx_version,
        });

        Ok(events)
    }

    /// Execute GraphQL query
    async fn query_graphql(&self, start_version: u64) -> Result<GraphQLData> {
        let query = r#"
            query GetBridgeEvents($startVersion: numeric!) {
                bridge_burn_events(where: {version: {_gt: $startVersion}}, order_by: {version: asc}) {
                    amount, btc_address, fee_rate, from, operator_id, timestamp, version
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send GraphQL request")?
            .json::<GraphQLResponse>()
            .await
            .context("Failed to parse GraphQL response")?;

        if let Some(errors) = response.errors {
            return Err(anyhow::anyhow!("GraphQL errors: {:?}", errors));
        }

        response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in GraphQL response"))
    }

    /// Process mint events
    fn process_mint_events(&self, raw_events: Vec<MintEventRaw>) -> Result<Vec<BridgeEvent>> {
        raw_events
            .into_iter()
            .map(|raw| self.create_mint_event(raw))
            .collect()
    }

    /// Process burn events
    fn process_burn_events(&self, raw_events: Vec<BurnEventRaw>) -> Result<Vec<BridgeEvent>> {
        raw_events
            .into_iter()
            .map(|raw| self.create_burn_event(raw))
            .collect()
    }

    /// Create mint event from raw data
    fn create_mint_event(&self, raw: MintEventRaw) -> Result<BridgeEvent> {
        let version = raw.version.parse().unwrap_or(0);
        let timestamp = raw.timestamp.parse().unwrap_or(0);
        let event = parse_mint_event(&serde_json::to_value(&raw)?)?;

        Ok(BridgeEvent::Mint(BridgeMintEvent {
            tx_version: version,
            timestamp,
            event,
        }))
    }

    /// Create burn event from raw data
    fn create_burn_event(&self, raw: BurnEventRaw) -> Result<BridgeEvent> {
        let version = raw.version.parse().unwrap_or(0);
        let timestamp = raw.timestamp.parse().unwrap_or(0);
        let event = parse_burn_event(&serde_json::to_value(&raw)?)?;

        Ok(BridgeEvent::Burn(BridgeBurnEvent {
            tx_version: version,
            timestamp,
            event,
        }))
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
