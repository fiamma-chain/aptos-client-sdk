//! Burn operation example
//!
//! This example shows how to use the Aptos Bridge SDK to burn tokens.

use anyhow::Result;
use aptos_client_sdk::BridgeClient;
use std::{env, time::Duration};
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let private_key =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable is required");

    let aptos_api_key = env::var("APTOS_API_KEY").ok();
    let bridge_contract_address =
        "0xc70be23fa7b086eb766776ca78e0d0633b5c0d1a58fa1b6e1f2207f481452e1c";
    let btc_light_client = "0x4f6417cea8184f3fbf73f63c26f6923da7c73ccb27feefacf5c31c4abcafda5e";

    let bridge_client = BridgeClient::new(
        &node_url,
        aptos_api_key.as_deref(),
        &private_key,
        &bridge_contract_address,
        &btc_light_client,
    )?;
    // Burn operation parameters
    let btc_address = "bcrt1phcnl4zcl2fu047pv4wx6y058v8u0n02at6lthvm7pcf2wrvjm5tqatn90k";
    let amount = 500000;
    let fee_rate = 5;
    let operator_id = 1;

    // Execute burn operation
    let tx_hash = bridge_client
        .burn(btc_address.to_string(), fee_rate, amount, operator_id)
        .await?;

    time::sleep(Duration::from_secs(5)).await;
    let tx = bridge_client.get_transaction_by_hash(&tx_hash).await?;
    if tx.success() {
        println!("Burn transaction successful, hash: {}", tx_hash);
    } else {
        println!("Burn transaction failed, error: {}", tx.vm_status());
    }

    Ok(())
}
