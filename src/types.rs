//! Core data type definitions
//!
//! This module defines all data types required for interacting with Aptos Bridge contracts.

use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
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
    pub merkle_proof: Vec<Vec<u8>>,
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
            .map_err(|e| anyhow!("Invalid address format '{}': {}", self.to, e))?;

        // Convert script type to u8
        let script_type_u8 = match self.script_type {
            ScriptType::P2SH => 0u8,
            ScriptType::P2WSH => 1u8,
            ScriptType::P2TR => 2u8,
        };

        // Serialize each parameter according to contract requirements
        let args = vec![
            bcs::to_bytes(&to_address)
                .map_err(|e| anyhow!("Failed to serialize to address: {}", e))?,
            bcs::to_bytes(&self.value).map_err(|e| anyhow!("Failed to serialize value: {}", e))?,
            bcs::to_bytes(&self.block_num)
                .map_err(|e| anyhow!("Failed to serialize block_num: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.tx_index)
                .map_err(|e| anyhow!("Failed to serialize tx_index: {}", e))?,
            bcs::to_bytes(&self.tx_out_ix)
                .map_err(|e| anyhow!("Failed to serialize tx_out_ix: {}", e))?,
            bcs::to_bytes(&script_type_u8)
                .map_err(|e| anyhow!("Failed to serialize script_type: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.block_header)
                .map_err(|e| anyhow!("Failed to serialize block_header: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.tx_id)
                .map_err(|e| anyhow!("Failed to serialize tx_id: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.merkle_proof)
                .map_err(|e| anyhow!("Failed to serialize tx_merkle_proof: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.raw_tx)
                .map_err(|e| anyhow!("Failed to serialize raw_tx: {}", e))?,
            bcs::to_bytes(&self.dest_script_hash)
                .map_err(|e| anyhow!("Failed to serialize dest_script_hash: {}", e))?,
        ];

        Ok(args)
    }
}

/// Mint event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintEvent {
    /// Recipient address
    pub to_address: String,
    /// Minted amount
    pub amount: u64,
    /// BTC transaction ID
    pub btc_tx_id: String,
    /// BTC block height
    pub btc_block_num: u64,
    /// Timestamp
    pub timestamp: Option<u64>,
    /// Version
    pub version: Option<u64>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
}

/// Burn event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnEvent {
    /// Sender address
    pub from_address: String,
    /// BTC address
    pub btc_address: String,
    /// Fee rate
    pub fee_rate: u64,
    /// Burned amount
    pub amount: u64,
    /// Operator ID
    pub operator_id: u64,
    /// Timestamp
    pub timestamp: Option<u64>,
    /// Version
    pub version: Option<u64>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
}

/// Raw mint event data (supports both GraphQL and transaction event sources)
///
/// This structure can handle event data from two sources:
/// - GraphQL queries: includes `timestamp` and `version` fields
/// - Transaction events: may not include these fields (will be None)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MintEventRaw {
    pub to_address: String,
    pub amount: String,
    pub btc_tx_id: String,
    pub btc_block_num: String,
    /// Optional timestamp (present in GraphQL, may be absent in transaction events)
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Optional version (present in GraphQL, may be absent in transaction events)
    #[serde(default)]
    pub version: Option<String>,
    /// Optional transaction hash (present in transaction events, may be absent in GraphQL)
    #[serde(default)]
    pub transaction_hash: Option<String>,
}

/// Raw burn event data (supports both GraphQL and transaction event sources)
///
/// This structure can handle event data from two sources:
/// - GraphQL queries: includes `timestamp` and `version` fields  
/// - Transaction events: may not include these fields (will be None)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BurnEventRaw {
    pub from_address: String,
    pub btc_address: String,
    pub fee_rate: String,
    pub amount: String,
    pub operator_id: String,
    /// Optional timestamp (present in GraphQL, may be absent in transaction events)
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Optional version (present in GraphQL, may be absent in transaction events)
    #[serde(default)]
    pub version: Option<String>,
    /// Optional transaction hash (present in transaction events, may be absent in GraphQL)
    #[serde(default)]
    pub transaction_hash: Option<String>,
}

impl From<MintEventRaw> for MintEvent {
    fn from(raw: MintEventRaw) -> Self {
        Self {
            to_address: raw.to_address,
            amount: raw.amount.parse().unwrap_or(0),
            btc_tx_id: raw.btc_tx_id,
            btc_block_num: raw.btc_block_num.parse().unwrap_or(0),
            timestamp: raw.timestamp.and_then(|t| parse_timestamp(&t)),
            version: raw.version.and_then(|v| v.parse().ok()),
            transaction_hash: raw.transaction_hash,
        }
    }
}

impl From<BurnEventRaw> for BurnEvent {
    fn from(raw: BurnEventRaw) -> Self {
        Self {
            from_address: raw.from_address,
            btc_address: raw.btc_address,
            fee_rate: raw.fee_rate.parse().unwrap_or(0),
            amount: raw.amount.parse().unwrap_or(0),
            operator_id: raw.operator_id.parse().unwrap_or(0),
            timestamp: raw.timestamp.and_then(|t| parse_timestamp(&t)),
            version: raw.version.and_then(|v| v.parse().ok()),
            transaction_hash: raw.transaction_hash,
        }
    }
}

/// Bridge event enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEvent {
    /// Mint event
    Mint(MintEvent),
    /// Burn event
    Burn(BurnEvent),
}

/// Parse mint event using serde_json
pub fn parse_mint_event(data: &serde_json::Value) -> Result<MintEvent> {
    let raw_event: MintEventRaw = serde_json::from_value(data.clone())
        .map_err(|e| anyhow!("Failed to parse mint event data: {}", e))?;
    Ok(raw_event.into())
}

/// Parse burn event using serde_json
pub fn parse_burn_event(data: &serde_json::Value) -> Result<BurnEvent> {
    let raw_event: BurnEventRaw = serde_json::from_value(data.clone())
        .map_err(|e| anyhow!("Failed to parse burn event data: {}", e))?;
    Ok(raw_event.into())
}

/// Constants module
pub mod constants {
    pub const EXPIRATION_TIMESTAMP_SECS: u64 = 60;
}

/// Parse ISO 8601 timestamp string to Unix timestamp (u64)
/// Assumes timestamp without timezone info is in UTC
fn parse_timestamp(timestamp_str: &str) -> Option<u64> {
    // First try to parse as NaiveDateTime (no timezone), then treat as UTC
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S") {
        return Some(naive_dt.and_utc().timestamp() as u64);
    }
    None
}
