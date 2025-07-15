//! Mint operation example
//!
//! This example shows how to use the Aptos Bridge SDK to mint tokens.

use anyhow::Result;
use aptos_bridge_sdk::{
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
    let bridge_contract_address =
        "0x348fb76b8668c1c4e5e0d0e9fe13b926dfeb309ec720947f4050ddc6c974d459";
    let btc_light_client = "0x105deccf9cb2725b9312ed0cb532490448a261e86f21df67ade4d3dc4221e41a";

    let mut bridge_client = BridgeClient::new(
        &node_url,
        &private_key,
        &bridge_contract_address,
        &btc_light_client,
    )
    .await?;

    // Create example pegs
    let pegs = create_example_pegs()?;

    // Display peg information
    for (i, peg) in pegs.iter().enumerate() {
        println!("Peg {}: {} satoshi to {}", i + 1, peg.value, peg.to);
    }

    // Execute mint operation
    let tx_hash = bridge_client.mint(pegs).await?;

    println!("Mint transaction hash: {}", tx_hash);

    Ok(())
}

fn create_example_pegs() -> Result<Vec<Peg>> {
    let pegs = vec![Peg {
        to: "0x2823126c1fd6124b0496b89dcb1de2ae0a71011baadf058c6a12ee22d0024cbe".to_string(),
        value: 500000,
        // For Local testing, we don't need to provide the block number and inclusion proof
        block_num: 0,
        inclusion_proof: TxProof {
            block_header: vec![],
            tx_id: vec![],
            tx_index: 0,
            merkle_proof: vec![],
            raw_tx: vec![],
        },
        tx_out_ix: 0,
        dest_script_hash: vec![],
        script_type: ScriptType::P2WSH,
    }];

    Ok(pegs)
}
