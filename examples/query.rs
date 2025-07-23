//! Query events example
//!
//! This example shows how to query bridge events from a user transaction hash.

use anyhow::Result;
use aptos_client_sdk::{BridgeClient, QueryClient};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    // Initialize query client
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let aptos_api_key = env::var("APTOS_API_KEY").ok();
    let query_client = QueryClient::new(node_url, aptos_api_key.as_deref())?;

    let bridge_contract_address =
        "0xeed4b8e27b6bd68e902e0e20633814d0d6d1a1c096763507fcaf058854a5b9b4";

    let tx_hash = "0x068e942ec3312ba5dd735392c5cc7091561a535a736ead923149cea057de7912";

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

    // Test BridgeClient methods
    println!("\n--- Testing BridgeClient methods ---");
    test_bridge_client_methods().await?;

    Ok(())
}

/// Test BridgeClient methods: get_latest_block_height and get_min_confirmations
async fn test_bridge_client_methods() -> Result<()> {
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let aptos_api_key = env::var("APTOS_API_KEY").ok();

    // We need a private key to create BridgeClient, but we won't use it for these read-only operations
    let private_key = env::var("PRIVATE_KEY").unwrap_or_else(|_| {
        // Use a dummy private key for read-only operations
        "0x1".to_string()
    });

    let bridge_contract_address =
        "0xeed4b8e27b6bd68e902e0e20633814d0d6d1a1c096763507fcaf058854a5b9b4";
    let btc_light_client = "0x749e2800973809a39eb72ed6e38f154151cef1213b2e72e031ad86875bbc051a";

    let bridge_client = BridgeClient::new(
        node_url,
        aptos_api_key.as_deref(),
        &private_key,
        bridge_contract_address,
        Some(&btc_light_client),
    )?;

    // Test get_min_confirmations
    match bridge_client.get_min_confirmations().await {
        Ok(min_confirmations) => {
            println!("✅ Minimum confirmations required: {}", min_confirmations);
        }
        Err(e) => {
            println!("❌ Error getting minimum confirmations: {}", e);
        }
    }

    // Test get_latest_block_height
    match bridge_client.get_latest_block_height().await {
        Ok(latest_height) => {
            println!("✅ Latest block height: {}", latest_height);
        }
        Err(e) => {
            println!("❌ Error getting latest block height: {}", e);
        }
    }

    Ok(())
}
