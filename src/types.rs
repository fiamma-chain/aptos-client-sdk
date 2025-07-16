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
    pub tx_id: String,
    /// BTC block height
    pub block_num: u64,
    /// Timestamp
    pub timestamp: u64,
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
pub(crate) struct MintEventRaw {
    pub to_address: String,
    pub amount: String,
    pub btc_tx_id: String,
    pub btc_block_num: String,
    pub timestamp: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BurnEventRaw {
    pub from: String,
    pub btc_address: String,
    pub fee_rate: String,
    pub amount: String,
    pub operator_id: String,
    pub timestamp: String,
    pub version: String,
}

impl From<MintEventRaw> for MintEvent {
    fn from(raw: MintEventRaw) -> Self {
        Self {
            to: raw.to_address,
            amount: raw.amount.parse().unwrap_or(0),
            tx_id: raw.btc_tx_id,
            block_num: raw.btc_block_num.parse().unwrap_or(0),
            timestamp: raw.timestamp.parse().unwrap_or(0),
        }
    }
}

impl From<BurnEventRaw> for BurnEvent {
    fn from(raw: BurnEventRaw) -> Self {
        Self {
            from: raw.from,
            btc_address: raw.btc_address,
            fee_rate: raw.fee_rate.parse().unwrap_or(0),
            amount: raw.amount.parse().unwrap_or(0),
            operator_id: raw.operator_id.parse().unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeBurnEvent {
    pub tx_version: u64,
    pub timestamp: u64,
    pub event: BurnEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMintEvent {
    pub tx_version: u64,
    pub timestamp: u64,
    pub event: MintEvent,
}

/// Bridge event enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEvent {
    /// Mint event
    Mint(BridgeMintEvent),
    /// Burn event
    Burn(BridgeBurnEvent),
}

/// Parse mint event using serde_json
pub fn parse_mint_event(data: &serde_json::Value) -> Result<MintEvent> {
    let raw_event: MintEventRaw =
        serde_json::from_value(data.clone()).context("Failed to parse mint event data")?;
    Ok(raw_event.into())
}

/// Parse burn event using serde_json
pub fn parse_burn_event(data: &serde_json::Value) -> Result<BurnEvent> {
    let raw_event: BurnEventRaw =
        serde_json::from_value(data.clone()).context("Failed to parse burn event data")?;
    Ok(raw_event.into())
}

/// Constants module
pub mod constants {
    pub const EXPIRATION_TIMESTAMP_SECS: u64 = 60;
}
