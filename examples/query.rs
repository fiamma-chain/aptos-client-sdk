//! Query operation example
//!
//! This example shows how to use the Aptos Bridge SDK to query bridge configuration and status.

use anyhow::Result;
use aptos_bridge_sdk::QueryClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Get configuration from environment variables
    let node_url = env::var("APTOS_NODE_URL")
        .unwrap_or_else(|_| "https://fullnode.devnet.aptoslabs.com/v1".to_string());
    let bridge_contract_address = env::var("BRIDGE_CONTRACT_ADDRESS")
        .expect("BRIDGE_CONTRACT_ADDRESS environment variable is required");

    println!("🔍 Aptos Bridge Query Example");
    println!("Node URL: {}", node_url);
    println!("Bridge Contract: {}", bridge_contract_address);
    println!();

    // Create query client
    let query_client = QueryClient::new(&node_url)?;

    println!("✅ Query client created");
    println!();

    // Query transaction example
    println!("📋 Querying example transaction...");

    // Example: query a specific transaction (you can replace this with an actual hash)
    let example_tx_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    match query_client.get_transaction_by_hash(example_tx_hash).await {
        Ok(transaction) => {
            println!("✅ Transaction found:");
            if let Ok(info) = transaction.transaction_info() {
                println!("  Hash: {}", info.hash);
            }
            println!("  Success: {}", transaction.success());
            if let Some(version) = transaction.version() {
                println!("  Version: {}", version);
            }
        }
        Err(e) => {
            println!("❌ Failed to retrieve transaction: {}", e);
            println!("💡 This is expected if the example hash doesn't exist");
        }
    }

    // Check if we should enter interactive mode
    println!("\n🔧 Would you like to enter interactive query mode? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        interactive_query(&query_client).await?;
    }

    println!("🎉 Query operations completed!");
    print_usage();

    Ok(())
}

async fn interactive_query(query_client: &QueryClient) -> Result<()> {
    println!("\n🔧 Interactive Query Mode");
    println!("Available commands:");
    println!("  1. transaction - Query transaction by hash");
    println!("  2. exit - Exit interactive mode");
    println!();

    loop {
        println!("Enter command (1-2):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" | "transaction" => {
                query_transaction(query_client).await?;
            }
            "2" | "exit" => {
                println!("👋 Exiting interactive mode");
                break;
            }
            _ => {
                println!("❌ Invalid command. Please enter 1-2.");
            }
        }

        println!();
    }

    Ok(())
}

async fn query_transaction(query_client: &QueryClient) -> Result<()> {
    println!("🔍 Transaction Query");
    println!("Enter transaction hash:");

    let mut tx_hash = String::new();
    std::io::stdin().read_line(&mut tx_hash)?;
    let tx_hash = tx_hash.trim();

    match query_client.get_transaction_by_hash(tx_hash).await {
        Ok(transaction) => {
            println!("✅ Transaction Details:");
            if let Ok(info) = transaction.transaction_info() {
                println!("  Hash: {}", info.hash);
                println!("  Gas Used: {}", info.gas_used);
            }
            println!("  Success: {}", transaction.success());
            if let Some(version) = transaction.version() {
                println!("  Version: {}", version);
            }
            println!("  VM Status: {}", transaction.vm_status());
        }
        Err(e) => {
            println!("❌ Failed to get transaction: {}", e);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("\n📖 Usage:");
    println!("Set the following environment variables:");
    println!("  APTOS_NODE_URL=https://fullnode.devnet.aptoslabs.com/v1");
    println!("  BRIDGE_CONTRACT_ADDRESS=contract_address_here");
    println!("\nExample:");
    println!("  export BRIDGE_CONTRACT_ADDRESS=0x123...");
    println!("  cargo run --example query");
}
