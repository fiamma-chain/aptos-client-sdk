//! Query events example
//!
//! This example shows how to query bridge events from a user transaction hash.

use anyhow::Result;
use aptos_bridge_sdk::QueryClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize query client
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let query_client = QueryClient::new(node_url)?;

    let bridge_contract_address =
        "0x2e5df32d3db81510b01dc0ec2fd6220b43b29b1e2a98b48a013a774f10726e5b";

    let tx_hash = "0xc0d462ffb1e03b6d72a15a82055f05f573a089acb77aff1e612d7476f3e1075d";

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
