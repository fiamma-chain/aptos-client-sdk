//! Query events example
//!
//! This example shows how to query bridge events from a user transaction hash.

use anyhow::Result;
use aptos_client_sdk::QueryClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    // Initialize query client
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let aptos_api_key = env::var("APTOS_API_KEY").ok();
    let query_client = QueryClient::new(node_url, aptos_api_key.as_deref())?;

    let bridge_contract_address =
        "0xc70be23fa7b086eb766776ca78e0d0633b5c0d1a58fa1b6e1f2207f481452e1c";

    let tx_hash = "0x162357b8a8044fede477cbd17b2ba61cfb15608152f7a9cdf2787b8e1754b942";

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
