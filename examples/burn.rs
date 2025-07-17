//! Burn operation example
//!
//! This example shows how to use the Aptos Bridge SDK to burn tokens.

use anyhow::Result;
use aptos_client_sdk::BridgeClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let private_key =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable is required");

    let aptos_api_key =
        env::var("APTOS_API_KEY").expect("APTOS_API_KEY environment variable is required");
    let bridge_contract_address =
        "0x6b891d58da6e4fd7bb2ab229917833c47cb34d8d60cf75e93d717bda43eee387";
    let btc_light_client = "0x67dd32fe9ee2e6d7c6016d51d912f5c7cf02032e9fe94b9c2db1b2762196952d";

    let mut bridge_client = BridgeClient::new(
        &node_url,
        &aptos_api_key,
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
