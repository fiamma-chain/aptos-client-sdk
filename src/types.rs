//! Core data type definitions
//!
//! This module defines all data types required for interacting with Aptos Bridge contracts.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Bitcoin transaction proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxProof {
    /// Block header data
    pub block_header: Vec<u8>,
    /// Transaction ID
    pub tx_id: Vec<u8>,
    /// Transaction index in block
    pub tx_index: u64,
    /// Merkle proof path
    pub merkle_proof: Vec<u8>,
    /// Raw transaction data
    pub raw_tx: Vec<u8>,
}

/// Bitcoin script type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptType {
    P2SH = 0,  // Pay to Script Hash
    P2WSH = 1, // Pay to Witness Script Hash
    P2TR = 2,  // Pay to Taproot
}

/// Peg structure for mint operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peg {
    /// Recipient address
    pub to: String,
    /// BTC amount (satoshi)
    pub value: u64,
    /// Block height
    pub block_num: u64,
    /// Inclusion proof
    pub inclusion_proof: TxProof,
    /// Transaction output index
    pub tx_out_ix: u64,
    /// Destination script hash
    pub dest_script_hash: Vec<u8>,
    /// Script type
    pub script_type: ScriptType,
}

impl Peg {
    /// Serialize peg data to BCS format for contract calls
    pub fn serialize_to_args(&self) -> Result<Vec<Vec<u8>>> {
        // Convert address string to AccountAddress
        let to_address = aptos_sdk::types::account_address::AccountAddress::from_str(&self.to)
            .with_context(|| format!("Invalid address format: {}", self.to))?;

        // Convert script type to u8
        let script_type_u8 = match self.script_type {
            ScriptType::P2SH => 0u8,
            ScriptType::P2WSH => 1u8,
            ScriptType::P2TR => 2u8,
        };

        // Serialize each parameter according to contract requirements
        let args = vec![
            bcs::to_bytes(&to_address).context("Failed to serialize to address")?,
            bcs::to_bytes(&self.value).context("Failed to serialize value")?,
            bcs::to_bytes(&self.block_num).context("Failed to serialize block_num")?,
            bcs::to_bytes(&self.inclusion_proof.tx_index)
                .context("Failed to serialize tx_index")?,
            bcs::to_bytes(&self.tx_out_ix).context("Failed to serialize tx_out_ix")?,
            bcs::to_bytes(&script_type_u8).context("Failed to serialize script_type")?,
            bcs::to_bytes(&self.inclusion_proof.block_header)
                .context("Failed to serialize block_header")?,
            bcs::to_bytes(&self.inclusion_proof.tx_id).context("Failed to serialize tx_id")?,
            bcs::to_bytes(&self.inclusion_proof.merkle_proof)
                .context("Failed to serialize tx_merkle_proof")?,
            bcs::to_bytes(&self.inclusion_proof.raw_tx).context("Failed to serialize raw_tx")?,
            bcs::to_bytes(&self.dest_script_hash)
                .context("Failed to serialize dest_script_hash")?,
        ];

        Ok(args)
    }
}

/// Mint event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintEvent {
    /// Recipient address
    pub to: String,
    /// Minted amount
    pub amount: u64,
    /// BTC transaction ID
    pub tx_id: Vec<u8>,
    /// Block height
    pub block_num: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintEventWithVersion {
    pub version: u64,
    pub sequence_number: u64,
    pub event: MintEvent,
}

/// Burn event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnEvent {
    /// Sender address
    pub from: String,
    /// BTC address
    pub btc_address: String,
    /// Fee rate
    pub fee_rate: u64,
    /// Burned amount
    pub amount: u64,
    /// Operator ID
    pub operator_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnEventWithVersion {
    pub version: u64,
    pub sequence_number: u64,
    pub event: BurnEvent,
}

/// Bridge event enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEvent {
    /// Mint event
    Mint(MintEventWithVersion),
    /// Burn event
    Burn(BurnEventWithVersion),
}

/// Constants module
pub mod constants {
    pub const EXPIRATION_TIMESTAMP_SECS: u64 = 60;
}
