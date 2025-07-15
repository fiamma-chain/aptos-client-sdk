//! Burn operation example
//!
//! This example shows how to use the Aptos Bridge SDK to burn tokens.

use anyhow::Result;
use aptos_bridge_sdk::BridgeClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Get configuration from environment variables
    let node_url = env::var("APTOS_NODE_URL")
        .unwrap_or_else(|_| "https://fullnode.devnet.aptoslabs.com/v1".to_string());
    let private_key =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable is required");
    let bridge_contract_address = env::var("BRIDGE_CONTRACT_ADDRESS")
        .expect("BRIDGE_CONTRACT_ADDRESS environment variable is required");
    let btc_light_client =
        env::var("BTC_LIGHT_CLIENT").expect("BTC_LIGHT_CLIENT environment variable is required");

    // Burn operation parameters
    let btc_address = env::var("BTC_ADDRESS")
        .unwrap_or_else(|_| "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string());
    let amount = env::var("AMOUNT")
        .unwrap_or_else(|_| "50000000".to_string()) // 0.5 BTC
        .parse::<u64>()
        .expect("Invalid amount format");
    let fee_rate = env::var("FEE_RATE")
        .unwrap_or_else(|_| "100".to_string())
        .parse::<u64>()
        .expect("Invalid fee rate format");
    let operator_id = env::var("OPERATOR_ID")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()
        .expect("Invalid operator ID format");

    println!("🔥 Aptos Bridge Burn Example");
    println!("Node URL: {}", node_url);
    println!("Bridge Contract: {}", bridge_contract_address);
    println!("BTC Address: {}", btc_address);
    println!("Fee Rate: {}", fee_rate);
    println!("Operator ID: {}", operator_id);
    println!();

    // Create Bridge client
    let mut bridge_client = BridgeClient::new(
        &node_url,
        &private_key,
        &bridge_contract_address,
        &btc_light_client,
    )
    .await?;

    // Execute burn operation
    println!("\n🔥 Executing burn operation...");
    let tx_hash = bridge_client
        .burn(btc_address, fee_rate, amount, operator_id)
        .await?;

    println!("✅ Burn transaction submitted!");
    println!("Transaction hash: {}", tx_hash);

    println!("\n🎉 Burn operation completed!");

    Ok(())
}
