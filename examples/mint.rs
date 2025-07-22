//! Mint operation example
//!
//! This example shows how to use the Aptos Bridge SDK to mint tokens.

use anyhow::Result;
use aptos_client_sdk::{
    types::{Peg, ScriptType, TxProof},
    BridgeClient,
};
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
        "0x22aff2ba274c94b5a8687ecde94d68d2123e66381f9b51e97c69d84add86f0b8";
    let btc_light_client = "0x749e2800973809a39eb72ed6e38f154151cef1213b2e72e031ad86875bbc051a";

    let bridge_client = BridgeClient::new(
        &node_url,
        aptos_api_key.as_deref(),
        &private_key,
        &bridge_contract_address,
        &btc_light_client,
    )?;

    // Create example peg
    let peg = create_example_peg()?;

    // Display peg information
    println!("Peg: {} satoshi to {}", peg.value, peg.to);

    // Execute mint operation
    let tx_hash = bridge_client.mint(peg).await?;
    time::sleep(Duration::from_secs(5)).await;
    let tx = bridge_client.get_transaction_by_hash(&tx_hash).await?;

    if tx.success() {
        println!("Mint transaction successful, hash: {}", tx_hash);
    } else {
        println!("Mint transaction failed, error: {}", tx.vm_status());
    }

    Ok(())
}

fn create_example_peg() -> Result<Peg> {
    let tx_id = "0x46724ae173f0c183e974fab2f582701c9d0e0e896a93e3e970d8710f870d28c9";
    let tx_id_bytes = hex::decode(tx_id.trim_start_matches("0x")).unwrap();
    let peg = Peg {
        to: "0x2823126c1fd6124b0496b89dcb1de2ae0a71011baadf058c6a12ee22d0024cbe".to_string(),
        value: 500000,
        // For Local testing, we don't need to provide the block number and inclusion proof
        block_num: 0,
        inclusion_proof: TxProof {
            block_header: vec![0x23],
            tx_id: tx_id_bytes,
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
