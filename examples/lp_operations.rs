//! LP Operations Example
//!
//! Demonstrates how to use LP-related functionality in the Aptos Bridge Client SDK.

use anyhow::Result;
use aptos_client_sdk::{
    BridgeClient, ClaimLPWithdrawParams, RegisterLPParams, TxProof, WithdrawByLPParams,
};
use std::{env, time::Duration};
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    let node_url = env::var("APTOS_NODE_URL")
        .unwrap_or_else(|_| "https://fullnode.testnet.aptoslabs.com/v1".to_string());

    let private_key =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable must be set");
    let aptos_api_key = env::var("APTOS_API_KEY").ok();
    let bridge_contract_address =
        "0xeed4b8e27b6bd68e902e0e20633814d0d6d1a1c096763507fcaf058854a5b9b4";
    let btc_light_client = "0x749e2800973809a39eb72ed6e38f154151cef1213b2e72e031ad86875bbc051a";

    // Create bridge client
    let client = BridgeClient::new(
        &node_url,
        aptos_api_key.as_deref(),
        &private_key,
        &bridge_contract_address,
        &btc_light_client,
    )?;

    // Example 1: Register a new LP
    let register_params = RegisterLPParams {
        lp_id: 1,
        bitcoin_addr: "bcrt1phcnl4zcl2fu047pv4wx6y058v8u0n02at6lthvm7pcf2wrvjm5tqatn90k"
            .to_string(),
        lp_addr: "0x2823126c1fd6124b0496b89dcb1de2ae0a71011baadf058c6a12ee22d0024cbe".to_string(),
        lp_fee: 1000, // 0.1% fee (basis points)
    };

    let tx_hash = client.register_lp(register_params).await?;
    time::sleep(Duration::from_secs(5)).await;

    let tx = client.get_transaction_by_hash(&tx_hash).await?;
    if tx.success() {
        println!("LP registered successfully, hash: {}", tx_hash);
    } else {
        println!("LP registered failed, error: {}", tx.vm_status());
    }

    // Example 2: Check LP status
    match client.get_lp_status(1).await {
        Ok(status) => println!("LP status: {:?}", status),
        Err(e) => println!("Failed to get LP status: {}", e),
    }

    // Example 3: Withdraw through LP
    let withdraw_params = WithdrawByLPParams {
        withdraw_id: 12350,
        btc_address: "bcrt1phcnl4zcl2fu047pv4wx6y058v8u0n02at6lthvm7pcf2wrvjm5tqatn90k".to_string(),
        receiver_script_hash: hex::decode("a914b7fcce0647b5e26b4a14b6b3b6f8b5e8e8e8e8e8e887")?,
        receive_min_amount: 450000, // 0.0045 BTC minimum
        lp_id: 1,
        amount: 500000, // 0.005 BTC
        fee_rate: 10,   // 10 sat/vB
    };

    let tx_hash = client.withdraw_by_lp(withdraw_params).await?;
    time::sleep(Duration::from_secs(5)).await;

    let tx = client.get_transaction_by_hash(&tx_hash).await?;
    if tx.success() {
        println!("Withdraw by lp transaction successful, hash: {}", tx_hash);
    } else {
        println!(
            "Withdraw by lp transaction failed, error: {}",
            tx.vm_status()
        );
    }

    // Example 4: Get LP withdraw information
    match client.get_lp_withdraw(12350).await {
        Ok(withdraw_info) => {
            println!("Withdraw info:");
            println!("  ID: {}", withdraw_info.id);
            println!("  Amount: {}", withdraw_info.withdraw_amount);
            println!("  Receiver: {}", withdraw_info.receiver_addr);
            println!("  LP ID: {}", withdraw_info.lp_id);
            println!(
                "  Receiver script hash: {}",
                withdraw_info.receiver_script_hash
            );
            println!("  Receive min amount: {}", withdraw_info.receive_min_amount);
            println!("  Fee rate: {}", withdraw_info.fee_rate);
            println!("  Timestamp: {}", withdraw_info.timestamp);
        }
        Err(e) => println!("Failed to get withdraw info: {}", e),
    }

    // Example 5: Claim LP withdraw (owner only)
    // This would typically be called by the bridge operator after confirming the BTC transaction
    let claim_params = ClaimLPWithdrawParams {
        withdraw_id: 12350,
        block_num: 800000,
        tx_out_ix: 0,
        amount_sats: 460000, // Amount actually received (after fees)
        inclusion_proof: TxProof {
            block_header: vec![0; 80], // Placeholder block header
            tx_id: vec![0; 32],        // Placeholder transaction ID
            tx_index: 0,
            merkle_proof: vec![vec![0; 32]], // Placeholder merkle proof
            raw_tx: vec![0; 250],            // Placeholder raw transaction
        },
    };

    let tx_hash = client.claim_lp_withdraw(claim_params).await?;
    time::sleep(Duration::from_secs(5)).await;

    let tx = client.get_transaction_by_hash(&tx_hash).await?;
    if tx.success() {
        println!("LP withdraw claimed, hash: {}", tx_hash);
    } else {
        println!("LP withdraw claimed failed, error: {}", tx.vm_status());
    }

    Ok(())
}
