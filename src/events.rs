//! Event listener implementation
//!
//! Provides functionality to listen to Aptos Bridge contract events.

use crate::types::{BurnEvent, MintEvent};
use crate::utils::parse_account_address;
use crate::{BurnEventWithVersion, MintEventWithVersion};

use anyhow::{Context, Result};
use aptos_sdk::{rest_client::Client, types::account_address::AccountAddress};
use async_trait::async_trait;
use url::Url;

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle Mint event
    async fn handle_mint(
        &self,
        mint_version: u64,
        mint_sequence_number: u64,
        event: MintEvent,
    ) -> Result<()>;

    /// Handle Burn event
    async fn handle_burn(
        &self,
        burn_version: u64,
        burn_sequence_number: u64,
        event: BurnEvent,
    ) -> Result<()>;
}

/// Event monitor
pub struct EventMonitor {
    rest_client: Client,
    contract_address: AccountAddress,
    handler: Box<dyn EventHandler>,
    mint_start: u64,
    burn_start: u64,
}

impl EventMonitor {
    /// Create new event monitor
    pub fn new(
        node_url: &str,
        contract_address: &str,
        handler: Box<dyn EventHandler>,
        mint_start: u64,
        burn_start: u64,
    ) -> Result<Self> {
        let contract_address = parse_account_address(contract_address)?;

        let rest_client = Client::new(
            Url::parse(node_url).with_context(|| format!("Invalid node URL: {}", node_url))?,
        );

        Ok(Self {
            rest_client,
            contract_address,
            handler,
            mint_start,
            burn_start,
        })
    }

    /// Process events with handler
    pub async fn process(&self) -> Result<()> {
        self.fetch_and_process_mint_events(self.mint_start).await?;

        self.fetch_and_process_burn_events(self.burn_start).await?;

        Ok(())
    }

    /// Fetch mint events
    async fn fetch_and_process_mint_events(&self, start: u64) -> Result<()> {
        let mint_events = self.fetch_mint_events(start).await?;

        for event in mint_events {
            self.handler
                .handle_mint(event.version, event.sequence_number, event.event)
                .await?;
        }

        Ok(())
    }

    /// Fetch mint events
    async fn fetch_mint_events(&self, start: u64) -> Result<Vec<MintEventWithVersion>> {
        // For #[event] structs, the struct_tag is the full event type path
        let struct_tag = format!("{}::bridge::Mint", self.contract_address.to_hex_literal());
        let field_name = "events";
        let response = self
            .rest_client
            .get_account_events(
                self.contract_address,
                &struct_tag,
                field_name,
                Some(start),
                None,
            )
            .await
            .context("Failed to fetch mint events from Aptos node")?;

        let mut events = Vec::new();
        for event in response.into_inner() {
            match self.parse_mint_event(&event.data) {
                Ok(mint_event) => events.push(MintEventWithVersion {
                    version: event.version.into(),
                    sequence_number: event.sequence_number.into(),
                    event: mint_event,
                }),
                Err(e) => return Err(e),
            }
        }

        Ok(events)
    }

    /// Fetch and process burn events
    async fn fetch_and_process_burn_events(&self, start: u64) -> Result<()> {
        let burn_events = self.fetch_burn_events(start).await?;

        for event in burn_events {
            self.handler
                .handle_burn(event.version, event.sequence_number, event.event)
                .await?;
        }

        Ok(())
    }

    /// Fetch burn events
    async fn fetch_burn_events(&self, start: u64) -> Result<Vec<BurnEventWithVersion>> {
        let struct_tag = format!("{}::bridge::Burn", self.contract_address.to_hex_literal());
        let field_name = "events";

        let response = self
            .rest_client
            .get_account_events(
                self.contract_address,
                &struct_tag,
                field_name,
                Some(start),
                None,
            )
            .await
            .context("Failed to fetch burn events from Aptos node")?;

        let mut events = Vec::new();
        for event in response.into_inner() {
            match self.parse_burn_event(&event.data) {
                Ok(burn_event) => events.push(BurnEventWithVersion {
                    version: event.version.into(),
                    sequence_number: event.sequence_number.into(),
                    event: burn_event,
                }),
                Err(e) => return Err(e),
            }
        }

        Ok(events)
    }

    /// Parse mint event using serde_json
    fn parse_mint_event(&self, data: &serde_json::Value) -> Result<MintEvent> {
        serde_json::from_value(data.clone()).context("Failed to parse mint event data")
    }

    /// Parse burn event using serde_json
    fn parse_burn_event(&self, data: &serde_json::Value) -> Result<BurnEvent> {
        serde_json::from_value(data.clone()).context("Failed to parse burn event data")
    }
}
