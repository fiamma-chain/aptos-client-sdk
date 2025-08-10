//! Mint operation example
//!
//! This example shows how to use the Aptos Bridge SDK to mint tokens.

use anyhow::Result;
use aptos_client_sdk::{
    types::{Peg, ScriptType, TxProof},
    BridgeClient,
};
use aptos_sdk::rest_client::aptos_api_types::TransactionData;
use std::{env, time::Duration};
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    // Get configuration from environment variables
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let private_key =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable is required");
    let aptos_api_key = env::var("APTOS_API_KEY").ok();
    let bridge_contract_address =
        "0xeed4b8e27b6bd68e902e0e20633814d0d6d1a1c096763507fcaf058854a5b9b4";
    let btc_light_client = "0x749e2800973809a39eb72ed6e38f154151cef1213b2e72e031ad86875bbc051a";

    let bridge_client = BridgeClient::new(
        &node_url,
        aptos_api_key.as_deref(),
        &private_key,
        &bridge_contract_address,
        Some(&btc_light_client),
    )?;

    // Create example peg
    let peg = create_example_peg()?;

    // Display peg information
    println!("Peg: {} satoshi to {}", peg.value, peg.to);

    // Save the address before moving peg
    let peg_address = peg.to.clone();

    // Execute mint operation
    let tx_hash = bridge_client.mint(peg).await?;
    let tx = bridge_client.get_transaction_by_hash(&tx_hash).await?;
    match tx {
        TransactionData::OnChain(txn) => {
            let status = txn.info.status();
            if status.is_success() {
                println!("Mint transaction successful, hash: {}", tx_hash);
            } else {
                println!("Mint transaction failed, error: {:?}", status);
            }
        }
        TransactionData::Pending(_) => {
            println!("Mint transaction is still pending: {}", tx_hash);
        }
    }

    time::sleep(Duration::from_secs(5)).await;

    // Query the balance of the peg
    let balance = bridge_client.get_btc_peg_balance(&peg_address).await?;
    println!("BTC peg balance: {} satoshi", balance);

    Ok(())
}

fn create_example_peg() -> Result<Peg> {
    let peg = Peg {
        to: "0x2823126c1fd6124b0496b89dcb1de2ae0a71011baadf058c6a12ee22d0024cbe".to_string(),
        value: 500000,
        // For Local testing, we don't need to provide the block number and inclusion proof
        block_num: 0,
        inclusion_proof: TxProof {
            block_header: vec![0x99],
            tx_id: vec![0x29],
            tx_index: 0,
            merkle_proof: vec![],
            raw_tx: vec![],
        },
        tx_out_ix: 0,
        dest_script_hash: vec![],
        script_type: ScriptType::P2WSH,
    };

    Ok(peg)
}
