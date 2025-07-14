//! Burn operation example
//!
//! This example shows how to use the Aptos Bridge SDK to burn tokens.

use aptos_bridge_sdk::{
    utils::{format_btc_amount, validate_btc_address},
    BridgeClient, QueryClient,
};
use aptos_sdk::types::chain_id::ChainId;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration from environment variables
    let node_url = env::var("APTOS_NODE_URL")
        .unwrap_or_else(|_| "https://fullnode.devnet.aptoslabs.com/v1".to_string());
    let private_key =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable is required");
    let bridge_contract_address = env::var("BRIDGE_CONTRACT_ADDRESS")
        .expect("BRIDGE_CONTRACT_ADDRESS environment variable is required");
    let faucet_url = env::var("FAUCET_URL")
        .unwrap_or_else(|_| "https://faucet.devnet.aptoslabs.com".to_string());

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
    println!("Amount: {}", format_btc_amount(amount));
    println!("Fee Rate: {}", fee_rate);
    println!("Operator ID: {}", operator_id);
    println!();

    // Create Bridge client
    let mut bridge_client =
        BridgeClient::new(&node_url, &private_key, &bridge_contract_address).await?;

    // Create query client
    let query_client = QueryClient::new(&node_url, &bridge_contract_address)?;

    // Query and print bridge configuration
    println!("\n📋 Querying bridge configuration...");
    query_client.print_bridge_config().await?;

    // Fund account with APT (testnet only)
    println!("\n💰 Funding account with APT...");

    // Validate burn parameters
    println!("\n🔍 Validating burn parameters...");
    match query_client
        .validate_burn_params(&btc_address, fee_rate, amount, operator_id)
        .await
    {
        Ok(_) => println!("✅ Burn parameters are valid"),
        Err(e) => {
            println!("❌ Burn parameter validation failed: {}", e);
            println!("💡 Please check your parameters and bridge configuration");

            // Try interactive configuration
            println!("\n🔧 Would you like to configure parameters interactively? (y/n)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                let (new_btc_address, new_fee_rate, new_amount, new_operator_id) =
                    interactive_config()?;

                // Execute burn with new parameters
                println!("\n🔥 Executing burn operation with new parameters...");
                let tx_hash = bridge_client
                    .burn(new_btc_address, new_fee_rate, new_amount, new_operator_id)
                    .await?;

                println!("✅ Burn transaction submitted!");
                println!("Transaction hash: {}", tx_hash);

                // Wait for transaction confirmation
                println!("\n⏳ Waiting for transaction confirmation...");
                match bridge_client.wait_for_transaction(&tx_hash, 60).await {
                    Ok(status) => {
                        println!("✅ Transaction confirmed!");
                        println!("Final status: {:?}", status);
                    }
                    Err(e) => {
                        println!("❌ Transaction confirmation failed: {}", e);
                        println!(
                            "💡 You can check the transaction status later using the hash: {}",
                            tx_hash
                        );
                    }
                }
            }

            return Ok(());
        }
    }

    // User confirmation
    println!(
        "\n⚠️  You are about to burn {} to BTC address: {}",
        format_btc_amount(amount),
        btc_address
    );
    println!("Are you sure you want to proceed? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        println!("❌ Operation cancelled by user");
        return Ok(());
    }

    // Execute burn operation
    println!("\n🔥 Executing burn operation...");
    let tx_hash = bridge_client
        .burn(btc_address, fee_rate, amount, operator_id)
        .await?;

    println!("✅ Burn transaction submitted!");
    println!("Transaction hash: {}", tx_hash);

    // Wait for transaction confirmation
    println!("\n⏳ Waiting for transaction confirmation...");
    match bridge_client.wait_for_transaction(&tx_hash, 60).await {
        Ok(status) => {
            println!("✅ Transaction confirmed!");
            println!("Final status: {:?}", status);
        }
        Err(e) => {
            println!("❌ Transaction confirmation failed: {}", e);
            println!(
                "💡 You can check the transaction status later using the hash: {}",
                tx_hash
            );
        }
    }

    println!("\n🎉 Burn operation completed!");
    print_usage();

    Ok(())
}

fn print_usage() {
    println!("\n📖 Usage:");
    println!("Set the following environment variables:");
    println!("  APTOS_NODE_URL=https://fullnode.devnet.aptoslabs.com/v1");
    println!("  PRIVATE_KEY=your_private_key_here");
    println!("  BRIDGE_CONTRACT_ADDRESS=contract_address_here");
    println!("  BTC_ADDRESS=bitcoin_address_here");
    println!("  AMOUNT=amount_in_satoshi (e.g., 50000000 for 0.5 BTC)");
    println!("  FEE_RATE=fee_rate_in_sat_per_vbyte (e.g., 100)");
    println!("  OPERATOR_ID=operator_id (e.g., 1)");
    println!("  FAUCET_URL=https://faucet.devnet.aptoslabs.com (optional)");
    println!("\nExample:");
    println!("  export PRIVATE_KEY=0x1234567890abcdef...");
    println!("  export BRIDGE_CONTRACT_ADDRESS=0x123...");
    println!("  export BTC_ADDRESS=bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4");
    println!("  export AMOUNT=50000000");
    println!("  export FEE_RATE=100");
    println!("  export OPERATOR_ID=1");
    println!("  cargo run --example burn");
}

fn interactive_config() -> Result<(String, u64, u64, u64), Box<dyn std::error::Error>> {
    println!("\n🔧 Interactive Configuration");

    // Get BTC address
    println!("Enter BTC address:");
    let mut btc_address = String::new();
    std::io::stdin().read_line(&mut btc_address)?;
    let btc_address = btc_address.trim().to_string();

    // Validate BTC address
    validate_btc_address(&btc_address)?;

    // Get amount
    println!("Enter amount in satoshi (e.g., 50000000 for 0.5 BTC):");
    let mut amount_str = String::new();
    std::io::stdin().read_line(&mut amount_str)?;
    let amount = amount_str.trim().parse::<u64>()?;

    // Get fee rate
    println!("Enter fee rate in sat/vbyte (e.g., 100):");
    let mut fee_rate_str = String::new();
    std::io::stdin().read_line(&mut fee_rate_str)?;
    let fee_rate = fee_rate_str.trim().parse::<u64>()?;

    // Get operator ID
    println!("Enter operator ID (e.g., 1):");
    let mut operator_id_str = String::new();
    std::io::stdin().read_line(&mut operator_id_str)?;
    let operator_id = operator_id_str.trim().parse::<u64>()?;

    println!("\n📋 Configuration Summary:");
    println!("BTC Address: {}", btc_address);
    println!("Amount: {}", format_btc_amount(amount));
    println!("Fee Rate: {}", fee_rate);
    println!("Operator ID: {}", operator_id);

    Ok((btc_address, fee_rate, amount, operator_id))
}
