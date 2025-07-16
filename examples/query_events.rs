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

    // Example transaction hash (replace with actual transaction hash)
    let tx_hash = "0x56c12a374facc5c6276d2eb454830b1adfd605a157550fd881712e8f81c2e992";

    println!("Checking transaction type for: {}", tx_hash);

    // First check if it's a user transaction
    match query_client.is_user_transaction(tx_hash).await {
        Ok(is_user_tx) => {
            if !is_user_tx {
                println!("❌ Transaction is not a user transaction. Only user transactions are supported.");
                return Ok(());
            }
            println!("✅ Transaction is a user transaction");
        }
        Err(e) => {
            println!("❌ Error checking transaction type: {}", e);
            return Ok(());
        }
    }

    // Get bridge events from transaction
    println!("\nQuerying bridge events...");
    match query_client
        .get_bridge_events_by_tx_hash(tx_hash, bridge_contract_address)
        .await
    {
        Ok(bridge_events) => {
            if bridge_events.is_empty() {
                println!("No bridge events found in this transaction.");
            } else {
                println!("Found {} bridge events:", bridge_events.len());
                for (i, event) in bridge_events.iter().enumerate() {
                    println!("  Event {}: {}", i + 1, event);
                }
            }
        }
        Err(e) => {
            println!("❌ Error querying bridge events: {}", e);
        }
    }

    // Get all events from transaction
    println!("\nQuerying all events...");
    match query_client.get_all_events_by_tx_hash(tx_hash).await {
        Ok(all_events) => {
            if all_events.is_empty() {
                println!("No events found in this transaction.");
            } else {
                println!("Found {} total events:", all_events.len());
                for (i, event) in all_events.iter().enumerate() {
                    println!("  Event {}: {}", i + 1, event.typ);

                    // Pretty print the event data
                    if let Ok(formatted_data) = serde_json::to_string_pretty(&event.data) {
                        println!("    Data: {}", formatted_data);
                    }
                    println!("    Sequence: {}", event.sequence_number);
                    println!();
                }
            }
        }
        Err(e) => {
            println!("❌ Error querying all events: {}", e);
        }
    }

    Ok(())
}
