//! Query operation example
//!
//! This example shows how to use the Aptos Bridge SDK to query bridge configuration and status.

use aptos_bridge_sdk::{utils::format_btc_amount, QueryClient};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let query_client = QueryClient::new(&node_url, &bridge_contract_address)?;

    println!("✅ Query client created");
    println!(
        "Contract Address: {}",
        query_client.get_contract_address_hex()
    );
    println!();

    // Query and display all configuration
    println!("📋 Querying bridge configuration...");
    match query_client.get_bridge_config().await {
        Ok(config) => {
            println!("✅ Bridge configuration retrieved successfully");
            println!();

            // Display detailed configuration
            println!("🌉 Bridge Configuration Details:");
            println!("┌─────────────────────────────────────────────────────────────┐");
            println!("│ Owner Information                                           │");
            println!("├─────────────────────────────────────────────────────────────┤");
            println!("│ Owner Address: {:<40} │", config.owner);
            println!("├─────────────────────────────────────────────────────────────┤");
            println!("│ Security Parameters                                         │");
            println!("├─────────────────────────────────────────────────────────────┤");
            println!("│ Min Confirmations: {:<40} │", config.min_confirmations);
            println!("│ Max Fee Rate: {:<44} │", config.max_fee_rate);
            println!("├─────────────────────────────────────────────────────────────┤");
            println!("│ Mint Limits                                                 │");
            println!("├─────────────────────────────────────────────────────────────┤");
            println!("│ Max Pegs per Mint: {:<36} │", config.max_pegs_per_mint);
            println!(
                "│ Max BTC per Mint: {:<37} │",
                format_btc_amount(config.max_btc_per_mint)
            );
            println!(
                "│ Min BTC per Mint: {:<37} │",
                format_btc_amount(config.min_btc_per_mint)
            );
            println!("├─────────────────────────────────────────────────────────────┤");
            println!("│ Burn Limits                                                 │");
            println!("├─────────────────────────────────────────────────────────────┤");
            println!(
                "│ Max BTC per Burn: {:<36} │",
                format_btc_amount(config.max_btc_per_burn)
            );
            println!(
                "│ Min BTC per Burn: {:<36} │",
                format_btc_amount(config.min_btc_per_burn)
            );
            println!(
                "│ Burn Paused: {:<40} │",
                if config.burn_paused { "Yes" } else { "No" }
            );
            println!("└─────────────────────────────────────────────────────────────┘");
            println!();

            // Display configuration warnings
            if config.burn_paused {
                println!("⚠️  WARNING: Burn functionality is currently paused!");
            }

            if config.max_fee_rate > 1000 {
                println!(
                    "⚠️  WARNING: Maximum fee rate is quite high: {}",
                    config.max_fee_rate
                );
            }

            // Check if we should enter interactive mode
            println!("🔧 Would you like to enter interactive query mode? (y/n)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                interactive_query(&query_client).await?;
            }
        }
        Err(e) => {
            println!("❌ Failed to retrieve bridge configuration: {}", e);
            println!("💡 Please check your node URL and contract address");
            return Ok(());
        }
    }

    println!("🎉 Query operations completed!");
    print_usage();

    Ok(())
}

async fn interactive_query(query_client: &QueryClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 Interactive Query Mode");
    println!("Available commands:");
    println!("  1. limits - Show mint/burn limits");
    println!("  2. status - Show bridge status");
    println!("  3. validate - Validate parameters");
    println!("  4. proof - Check proof usage");
    println!("  5. exit - Exit interactive mode");
    println!();

    loop {
        println!("Enter command (1-5):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" | "limits" => {
                print_limits(query_client).await?;
            }
            "2" | "status" => {
                print_status(query_client).await?;
            }
            "3" | "validate" => {
                interactive_validate(query_client).await?;
            }
            "4" | "proof" => {
                interactive_proof_check(query_client).await?;
            }
            "5" | "exit" => {
                println!("👋 Exiting interactive mode");
                break;
            }
            _ => {
                println!("❌ Invalid command. Please enter 1-5.");
            }
        }

        println!();
    }

    Ok(())
}

async fn print_limits(query_client: &QueryClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Mint/Burn Limits:");

    match query_client.get_bridge_config().await {
        Ok(config) => {
            println!("  Mint Limits:");
            println!("    Max Pegs per Transaction: {}", config.max_pegs_per_mint);
            println!(
                "    Max BTC per Transaction: {}",
                format_btc_amount(config.max_btc_per_mint)
            );
            println!(
                "    Min BTC per Transaction: {}",
                format_btc_amount(config.min_btc_per_mint)
            );
            println!("  Burn Limits:");
            println!(
                "    Max BTC per Transaction: {}",
                format_btc_amount(config.max_btc_per_burn)
            );
            println!(
                "    Min BTC per Transaction: {}",
                format_btc_amount(config.min_btc_per_burn)
            );
        }
        Err(e) => {
            println!("❌ Failed to get limits: {}", e);
        }
    }

    Ok(())
}

async fn print_status(query_client: &QueryClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Bridge Status:");

    match query_client.get_bridge_config().await {
        Ok(config) => {
            println!("  Owner: {}", config.owner);
            println!("  Min Confirmations: {}", config.min_confirmations);
            println!("  Max Fee Rate: {}", config.max_fee_rate);
            println!(
                "  Burn Status: {}",
                if config.burn_paused {
                    "Paused"
                } else {
                    "Active"
                }
            );
        }
        Err(e) => {
            println!("❌ Failed to get status: {}", e);
        }
    }

    Ok(())
}

async fn interactive_validate(
    query_client: &QueryClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Parameter Validation");
    println!("Choose validation type:");
    println!("  1. Mint parameters");
    println!("  2. Burn parameters");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => {
            println!("Enter amount in satoshi:");
            let mut amount_str = String::new();
            std::io::stdin().read_line(&mut amount_str)?;
            let amount = amount_str.trim().parse::<u64>()?;

            println!("Enter number of pegs:");
            let mut pegs_str = String::new();
            std::io::stdin().read_line(&mut pegs_str)?;
            let pegs_count = pegs_str.trim().parse::<usize>()?;

            // Create dummy pegs for validation
            let pegs = vec![
                aptos_bridge_sdk::types::Peg {
                    to: "0x1".to_string(),
                    value: amount,
                    block_num: 0,
                    inclusion_proof: aptos_bridge_sdk::types::TxProof {
                        block_header: vec![],
                        tx_id: vec![],
                        tx_index: 0,
                        merkle_proof: vec![],
                        raw_tx: vec![],
                    },
                    tx_out_ix: 0,
                    dest_script_hash: vec![],
                    script_type: aptos_bridge_sdk::types::ScriptType::P2PKH,
                };
                pegs_count
            ];

            match query_client.validate_mint_params(&pegs).await {
                Ok(_) => println!("✅ Mint parameters are valid"),
                Err(e) => println!("❌ Mint parameters are invalid: {}", e),
            }
        }
        "2" => {
            println!("Enter BTC address:");
            let mut btc_address = String::new();
            std::io::stdin().read_line(&mut btc_address)?;
            let btc_address = btc_address.trim();

            println!("Enter amount in satoshi:");
            let mut amount_str = String::new();
            std::io::stdin().read_line(&mut amount_str)?;
            let amount = amount_str.trim().parse::<u64>()?;

            println!("Enter fee rate:");
            let mut fee_rate_str = String::new();
            std::io::stdin().read_line(&mut fee_rate_str)?;
            let fee_rate = fee_rate_str.trim().parse::<u64>()?;

            println!("Enter operator ID:");
            let mut operator_id_str = String::new();
            std::io::stdin().read_line(&mut operator_id_str)?;
            let operator_id = operator_id_str.trim().parse::<u64>()?;

            match query_client
                .validate_burn_params(btc_address, fee_rate, amount, operator_id)
                .await
            {
                Ok(_) => println!("✅ Burn parameters are valid"),
                Err(e) => println!("❌ Burn parameters are invalid: {}", e),
            }
        }
        _ => {
            println!("❌ Invalid selection");
        }
    }

    Ok(())
}

async fn interactive_proof_check(
    query_client: &QueryClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Proof Usage Check");
    println!("Enter proof hash (hex):");

    let mut proof_hash = String::new();
    std::io::stdin().read_line(&mut proof_hash)?;
    let proof_hash = proof_hash.trim();

    match query_client.is_proof_used(proof_hash).await {
        Ok(used) => {
            if used {
                println!("❌ Proof has already been used");
            } else {
                println!("✅ Proof is available for use");
            }
        }
        Err(e) => {
            println!("❌ Failed to check proof: {}", e);
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
