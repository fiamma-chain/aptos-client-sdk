pub mod bridge_client;
pub mod events;
pub mod query_client;
pub mod types;
pub mod utils;

// Re-export commonly used types and functions
pub use bridge_client::BridgeClient;
pub use events::{EventHandler, EventMonitor};
pub use query_client::QueryClient;

// Re-export main data types (excluding error types)
pub use types::{BridgeEvent, BurnEvent, MintEvent, Peg, ScriptType, TxProof};
