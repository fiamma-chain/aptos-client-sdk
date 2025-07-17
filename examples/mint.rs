//! Mint operation example
//!
//! This example shows how to use the Aptos Bridge SDK to mint tokens.

use anyhow::Result;
use aptos_client_sdk::{
    types::{Peg, ScriptType, TxProof},
    BridgeClient,
};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    // Get configuration from environment variables
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

    // Create example peg
    let peg = create_example_peg()?;

    // Display peg information
    println!("Peg: {} satoshi to {}", peg.value, peg.to);

    // Execute mint operation
    let tx_hash = bridge_client.mint(peg).await?;

    println!("Mint transaction hash: {}", tx_hash);

    Ok(())
}

fn create_example_peg() -> Result<Peg> {
    let peg = Peg {
        to: "0x2823126c1fd6124b0496b89dcb1de2ae0a71011baadf058c6a12ee22d0024cbe".to_string(),
        value: 500000,
        // For Local testing, we don't need to provide the block number and inclusion proof
        block_num: 0,
        inclusion_proof: TxProof {
            block_header: vec![],
            tx_id: vec![0x4],
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
