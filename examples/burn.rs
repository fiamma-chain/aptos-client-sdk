//! Burn operation example
//!
//! This example shows how to use the Aptos Bridge SDK to burn tokens.

use anyhow::Result;
use aptos_bridge_sdk::BridgeClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    // Get configuration from environment variables
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let private_key =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable is required");
    let bridge_contract_address = "0x1234567890123456789012345678901234567890";
    let btc_light_client = "0x1234567890123456789012345678901234567890";

    let mut bridge_client = BridgeClient::new(
        &node_url,
        &private_key,
        &bridge_contract_address,
        &btc_light_client,
    )
    .await?;
    // Burn operation parameters
    let btc_address = "bcrt1phcnl4zcl2fu047pv4wx6y058v8u0n02at6lthvm7pcf2wrvjm5tqatn90k";
    let amount = 500000;
    let fee_rate = 5;
    let operator_id = 1;

    // Execute burn operation
    let tx_hash = bridge_client
        .burn(btc_address.to_string(), fee_rate, amount, operator_id)
        .await?;

    println!("Burn transaction hash: {}", tx_hash);

    Ok(())
}
