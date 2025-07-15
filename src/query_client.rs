//! Query client implementation
//!
//! Provides functionality to query Aptos Bridge contract configuration and status.

use crate::types::{BridgeError, BridgeResult};

use aptos_sdk::rest_client::{Client, Transaction};
use url::Url;

/// Query client
pub struct QueryClient {
    /// REST client
    rest_client: Client,
}

impl QueryClient {
    /// Create new query client
    pub fn new(node_url: &str) -> BridgeResult<Self> {
        let rest_client = Client::new(
            Url::parse(node_url)
                .map_err(|e| BridgeError::Other(format!("Invalid node URL: {}", e)))?,
        );

        Ok(Self { rest_client })
    }

    /// Query transaction status
    pub async fn get_transaction_by_hash(&self, tx_hash: &str) -> BridgeResult<Transaction> {
        // Parse transaction hash
        let tx_hash = tx_hash
            .parse()
            .map_err(|e| BridgeError::Other(format!("Invalid transaction hash: {}", e)))?;

        let response = self
            .rest_client
            .get_transaction_by_hash(tx_hash)
            .await
            .map_err(|e| BridgeError::Aptos(e.to_string()))?;

        Ok(response.inner().clone())
    }
}
