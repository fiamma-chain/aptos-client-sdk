//! # Aptos Bridge SDK
//!
//! This is a Rust SDK for interacting with Bitcoin Bridge contracts on the Aptos blockchain.
//!
//! ## Main Features
//! - Call mint and burn methods
//! - Listen to bridge events
//! - Query bridge configuration and status
//!
//! ## Usage Example
//! ```no_run
//! use aptos_bridge_sdk::{BridgeClient, EventMonitor};
//!
//! // Create bridge client
//! let client = BridgeClient::new(
//!     "https://fullnode.devnet.aptoslabs.com/v1",
//!     "your_private_key",
//!     "bridge_contract_address",
//!     aptos_sdk::types::chain_id::ChainId::new(2), // devnet
//! ).await?;
//!
//! // Call mint method
//! let tx_hash = client.mint(pegs).await?;
//! ```

pub mod bridge_client;
pub mod events;
pub mod query_client;
pub mod types;
pub mod utils;

// Re-export commonly used types and functions
pub use bridge_client::BridgeClient;
pub use events::{EventHandler, EventMonitor};
pub use query_client::QueryClient;
pub use types::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
