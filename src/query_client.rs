//! Query client implementation
//!
//! Provides functionality to query Aptos Bridge contract configuration and status.

use anyhow::{Context, Result};
use aptos_sdk::rest_client::{Client, Transaction};
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
        let tx_hash = tx_hash
            .parse()
            .with_context(|| format!("Invalid transaction hash: {}", tx_hash))?;

        let response = self
            .rest_client
            .get_transaction_by_hash(tx_hash)
            .await
            .context("Failed to get transaction from Aptos node")?;

        Ok(response.inner().clone())
    }
}
