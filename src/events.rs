//! Event listener implementation
//!
//! Provides functionality to listen to Aptos Bridge contract events.

use crate::types::{BridgeError, BridgeResult, BurnEvent, MintEvent};
use crate::utils::parse_account_address;
use crate::{BurnEventWithVersion, MintEventWithVersion};

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
    ) -> BridgeResult<()>;

    /// Handle Burn event
    async fn handle_burn(
        &self,
        burn_version: u64,
        burn_sequence_number: u64,
        event: BurnEvent,
    ) -> BridgeResult<()>;
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
    ) -> BridgeResult<Self> {
        let contract_address = parse_account_address(contract_address)?;

        let rest_client = Client::new(
            Url::parse(node_url)
                .map_err(|e| BridgeError::Other(format!("Invalid node URL: {}", e)))?,
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
    pub async fn process(&self) -> BridgeResult<()> {
        self.fetch_and_process_mint_events(self.mint_start).await?;

        self.fetch_and_process_burn_events(self.burn_start).await?;

        Ok(())
    }

    /// Fetch mint events
    async fn fetch_and_process_mint_events(&self, start: u64) -> BridgeResult<()> {
        let mint_events = self.fetch_mint_events(start).await?;

        for event in mint_events {
            self.handler
                .handle_mint(event.version, event.sequence_number, event.event)
                .await?;
        }

        Ok(())
    }

    /// Fetch mint events
    async fn fetch_mint_events(&self, start: u64) -> BridgeResult<Vec<MintEventWithVersion>> {
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
            .map_err(|e| BridgeError::FetchEventsError(e.to_string()))?;

        let mut events = Vec::new();
        for event in response.into_inner() {
            match self.parse_mint_event(&event.data) {
                Ok(mint_event) => events.push(MintEventWithVersion {
                    version: event.version.into(),
                    sequence_number: event.sequence_number.into(),
                    event: mint_event,
                }),
                Err(e) => eprintln!("Failed to parse mint event: {}", e),
            }
        }

        Ok(events)
    }

    /// Fetch and process burn events
    async fn fetch_and_process_burn_events(&self, start: u64) -> BridgeResult<()> {
        let burn_events = self.fetch_burn_events(start).await?;

        for event in burn_events {
            self.handler
                .handle_burn(event.version, event.sequence_number, event.event)
                .await?;
        }

        Ok(())
    }

    /// Fetch burn events
    async fn fetch_burn_events(&self, start: u64) -> BridgeResult<Vec<BurnEventWithVersion>> {
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
            .map_err(|e| BridgeError::FetchEventsError(e.to_string()))?;

        let mut events = Vec::new();
        for event in response.into_inner() {
            match self.parse_burn_event(&event.data) {
                Ok(burn_event) => events.push(BurnEventWithVersion {
                    version: event.version.into(),
                    sequence_number: event.sequence_number.into(),
                    event: burn_event,
                }),
                Err(e) => eprintln!("Failed to parse burn event: {}", e),
            }
        }

        Ok(events)
    }

    /// Parse mint event using serde_json
    fn parse_mint_event(&self, data: &serde_json::Value) -> BridgeResult<MintEvent> {
        serde_json::from_value(data.clone()).map_err(|e| {
            BridgeError::EventParseFailed(format!("Failed to parse mint event: {}", e))
        })
    }

    /// Parse burn event using serde_json
    fn parse_burn_event(&self, data: &serde_json::Value) -> BridgeResult<BurnEvent> {
        serde_json::from_value(data.clone()).map_err(|e| {
            BridgeError::EventParseFailed(format!("Failed to parse burn event: {}", e))
        })
    }
}
