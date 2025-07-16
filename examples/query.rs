//! Query events example
//!
//! This example shows how to query bridge events from a user transaction hash.

use anyhow::Result;
use aptos_client_sdk::QueryClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize query client
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let query_client = QueryClient::new(node_url)?;

    let bridge_contract_address =
        "0x6b891d58da6e4fd7bb2ab229917833c47cb34d8d60cf75e93d717bda43eee387";

    let tx_hash = "0x7c0bf45365fe3fd63f61f78ca01939cfd278e95f57f0d991b28f96d328a2bf33";

    match query_client
        .get_bridge_events_by_hash(tx_hash, bridge_contract_address)
        .await
    {
        Ok(bridge_events) => {
            if bridge_events.is_empty() {
                println!("No bridge events found in this transaction.");
            } else {
                println!("Found {} bridge events:", bridge_events.len());
                for (i, event) in bridge_events.iter().enumerate() {
                    println!("  Event {}: {:#?}", i + 1, event);
                }
            }
        }
        Err(e) => {
            println!("❌ Error querying bridge events: {}", e);
        }
    }

    Ok(())
}
