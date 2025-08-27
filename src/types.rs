//! Core data type definitions
//!
//! This module defines all data types required for interacting with Aptos Bridge contracts.

use anyhow::{anyhow, Result};
use aptos_sdk::types::account_address::AccountAddress;
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
}

impl Peg {
    /// Serialize peg data to BCS format for contract calls
    pub fn serialize_to_args(&self) -> Result<Vec<Vec<u8>>> {
        // Convert address string to AccountAddress
        let to_address = aptos_sdk::types::account_address::AccountAddress::from_str(&self.to)
            .map_err(|e| anyhow!("Invalid address format '{}': {}", self.to, e))?;

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

/// WithdrawByLP event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawByLPEvent {
    /// Sender address
    pub from_address: String,
    /// Withdraw ID
    pub withdraw_id: u64,
    /// BTC address
    pub btc_address: String,
    /// Fee rate
    pub fee_rate: u64,
    /// Withdrawn amount
    pub amount: u64,
    /// LP ID
    pub lp_id: u64,
    /// Minimum receive amount
    pub receive_min_amount: u64,
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

/// Raw WithdrawByLP event data (supports both GraphQL and transaction event sources)
///
/// This structure can handle event data from two sources:
/// - GraphQL queries: includes `timestamp` and `version` fields  
/// - Transaction events: may not include these fields (will be None)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WithdrawByLPEventRaw {
    pub from_address: String,
    pub withdraw_id: String,
    pub btc_address: String,
    pub fee_rate: String,
    pub amount: String,
    pub lp_id: String,
    pub receive_min_amount: String,
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

impl From<WithdrawByLPEventRaw> for WithdrawByLPEvent {
    fn from(raw: WithdrawByLPEventRaw) -> Self {
        Self {
            from_address: raw.from_address,
            withdraw_id: raw.withdraw_id.parse().unwrap_or(0),
            btc_address: raw.btc_address,
            fee_rate: raw.fee_rate.parse().unwrap_or(0),
            amount: raw.amount.parse().unwrap_or(0),
            lp_id: raw.lp_id.parse().unwrap_or(0),
            receive_min_amount: raw.receive_min_amount.parse().unwrap_or(0),
            timestamp: raw.timestamp.and_then(|t| parse_timestamp(&t)),
            version: raw.version.and_then(|v| v.parse().ok()),
            transaction_hash: raw.transaction_hash,
        }
    }
}

/// BCS-compatible Mint event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MintEventBCS {
    pub to_address: [u8; 32], // AccountAddress as fixed-size array
    pub amount: u64,
    pub btc_tx_id: Vec<u8>,
    pub btc_block_num: u64,
}

/// BCS-compatible Burn event structure  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BurnEventBCS {
    pub from_address: [u8; 32], // AccountAddress as fixed-size array
    pub btc_address: String,
    pub fee_rate: u64,
    pub amount: u64,
    pub operator_id: u64,
}

/// BCS-compatible WithdrawByLP event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WithdrawByLPEventBCS {
    pub from_address: [u8; 32], // AccountAddress as fixed-size array
    pub withdraw_id: u64,
    pub btc_address: String, // BCS will handle variable-length encoding
    pub fee_rate: u64,
    pub amount: u64,
    pub lp_id: u64,
    pub receive_min_amount: u64,
}

impl From<MintEventBCS> for MintEvent {
    fn from(bcs: MintEventBCS) -> Self {
        // Convert AccountAddress bytes to hex string with fallback
        let to_address = AccountAddress::from_bytes(&bcs.to_address)
            .map(|addr| addr.to_hex_literal())
            .unwrap_or_else(|_| format!("0x{}", hex::encode(&bcs.to_address)));

        // Convert btc_tx_id bytes to hex string
        let btc_tx_id = hex::encode(&bcs.btc_tx_id);

        MintEvent {
            to_address,
            amount: bcs.amount,
            btc_tx_id,
            btc_block_num: bcs.btc_block_num,
            timestamp: None, // Not available in BCS events
            version: None,
            transaction_hash: None,
        }
    }
}

impl From<BurnEventBCS> for BurnEvent {
    fn from(bcs: BurnEventBCS) -> Self {
        // Convert AccountAddress bytes to hex string with fallback
        let from_address = AccountAddress::from_bytes(&bcs.from_address)
            .map(|addr| addr.to_hex_literal())
            .unwrap_or_else(|_| format!("0x{}", hex::encode(&bcs.from_address)));

        BurnEvent {
            from_address,
            btc_address: bcs.btc_address,
            fee_rate: bcs.fee_rate,
            amount: bcs.amount,
            operator_id: bcs.operator_id,
            timestamp: None, // Not available in BCS events
            version: None,
            transaction_hash: None,
        }
    }
}

impl From<WithdrawByLPEventBCS> for WithdrawByLPEvent {
    fn from(bcs: WithdrawByLPEventBCS) -> Self {
        // Convert AccountAddress bytes to hex string with fallback
        let from_address = AccountAddress::from_bytes(&bcs.from_address)
            .map(|addr| addr.to_hex_literal())
            .unwrap_or_else(|_| format!("0x{}", hex::encode(&bcs.from_address)));

        WithdrawByLPEvent {
            from_address,
            withdraw_id: bcs.withdraw_id,
            btc_address: bcs.btc_address,
            fee_rate: bcs.fee_rate,
            amount: bcs.amount,
            lp_id: bcs.lp_id,
            receive_min_amount: bcs.receive_min_amount,
            timestamp: None, // Not available in BCS events
            version: None,
            transaction_hash: None,
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
    /// WithdrawByLP event
    WithdrawByLP(WithdrawByLPEvent),
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

/// Parse WithdrawByLP event using serde_json
pub fn parse_withdraw_by_lp_event(data: &serde_json::Value) -> Result<WithdrawByLPEvent> {
    let raw_event: WithdrawByLPEventRaw = serde_json::from_value(data.clone())
        .map_err(|e| anyhow!("Failed to parse withdraw by LP event data: {}", e))?;
    Ok(raw_event.into())
}

/// LP Status enumeration (matches Move contract LPStatus enum)
/// 0 = UNREGISTERED, 1 = ACTIVE, 2 = SUSPENDED, 3 = TERMINATED
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
#[serde(tag = "__variant__")]
pub enum LPStatus {
    UNREGISTERED = 0,
    ACTIVE = 1,
    SUSPENDED = 2,
    TERMINATED = 3,
}

impl LPStatus {
    /// Parse LP status data from view function response
    pub fn from_view_response(result: &serde_json::Value) -> Result<Self> {
        // Now serde handles the __variant__ format automatically
        let status = serde_json::from_value::<LPStatus>(result.clone())
            .map_err(|e| anyhow!("Failed to parse get_lp_status response: {}", e))?;
        Ok(status)
    }

    /// Convert variant name string to LPStatus enum
    pub fn from_variant_name(variant: &str) -> Result<Self> {
        match variant {
            "UNREGISTERED" => Ok(LPStatus::UNREGISTERED),
            "ACTIVE" => Ok(LPStatus::ACTIVE),
            "SUSPENDED" => Ok(LPStatus::SUSPENDED),
            "TERMINATED" => Ok(LPStatus::TERMINATED),
            _ => Err(anyhow!("Invalid LP status variant: '{}'. Valid variants are: UNREGISTERED, ACTIVE, SUSPENDED, TERMINATED", variant)),
        }
    }
}

/// LP information structure  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPInfo {
    pub lp_id: u64,
    pub bitcoin_addr: String,
    pub lp_addr: String,
    pub lp_fee: u64,
    pub status: LPStatus,
}

/// LP withdraw information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPWithdraw {
    pub id: u64,
    pub withdraw_amount: u64,
    pub receiver_addr: String,
    pub receiver_script_hash: String,
    pub receive_min_amount: u64,
    pub fee_rate: u64,
    pub timestamp: u64,
    pub lp_id: u64,
}

/// LP withdraw raw information structure (Move contract returns string values)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPWithdrawRaw {
    pub id: String,
    pub withdraw_amount: String,
    pub receiver_addr: String,
    pub receiver_script_hash: String,
    pub receive_min_amount: String,
    pub fee_rate: String,
    pub timestamp: String,
    pub lp_id: String,
}

impl From<LPWithdrawRaw> for LPWithdraw {
    fn from(raw: LPWithdrawRaw) -> Self {
        Self {
            id: raw.id.parse().unwrap_or(0),
            withdraw_amount: raw.withdraw_amount.parse().unwrap_or(0),
            receiver_addr: raw.receiver_addr,
            receiver_script_hash: raw.receiver_script_hash,
            receive_min_amount: raw.receive_min_amount.parse().unwrap_or(0),
            fee_rate: raw.fee_rate.parse().unwrap_or(0),
            timestamp: raw.timestamp.parse().unwrap_or(0),
            lp_id: raw.lp_id.parse().unwrap_or(0),
        }
    }
}

impl LPWithdraw {
    /// Parse LP withdraw data from view function response
    pub fn from_view_response(result: &serde_json::Value) -> Result<Self> {
        // Parse as LPWithdrawRaw since Move contract returns string values for numbers
        let raw = serde_json::from_value::<LPWithdrawRaw>(result.clone())
            .map_err(|e| anyhow!("Failed to parse get_lp_withdraw response: {}", e))?;
        Ok(raw.into())
    }
}

/// Request parameters for withdraw_by_lp function
#[derive(Debug, Clone)]
pub struct WithdrawByLPParams {
    pub withdraw_id: u64,
    pub btc_address: String,
    pub receiver_script_hash: Vec<u8>,
    pub receive_min_amount: u64,
    pub lp_id: u64,
    pub amount: u64,
    pub fee_rate: u64,
}

impl WithdrawByLPParams {
    /// Serialize request parameters to BCS format for contract calls
    pub fn serialize_to_args(&self) -> Result<Vec<Vec<u8>>> {
        let args = vec![
            bcs::to_bytes(&self.withdraw_id)
                .map_err(|e| anyhow!("Failed to serialize withdraw_id: {}", e))?,
            bcs::to_bytes(&self.btc_address)
                .map_err(|e| anyhow!("Failed to serialize btc_address: {}", e))?,
            bcs::to_bytes(&self.receiver_script_hash)
                .map_err(|e| anyhow!("Failed to serialize receiver_script_hash: {}", e))?,
            bcs::to_bytes(&self.receive_min_amount)
                .map_err(|e| anyhow!("Failed to serialize receive_min_amount: {}", e))?,
            bcs::to_bytes(&self.lp_id).map_err(|e| anyhow!("Failed to serialize lp_id: {}", e))?,
            bcs::to_bytes(&self.amount)
                .map_err(|e| anyhow!("Failed to serialize amount: {}", e))?,
            bcs::to_bytes(&self.fee_rate)
                .map_err(|e| anyhow!("Failed to serialize fee_rate: {}", e))?,
        ];
        Ok(args)
    }
}

/// Request parameters for claim_lp_withdraw function
#[derive(Debug, Clone)]
pub struct ClaimLPWithdrawParams {
    pub withdraw_id: u64,
    pub block_num: u64,
    pub tx_out_ix: u64,
    pub amount_sats: u64,
    pub inclusion_proof: TxProof,
}

impl ClaimLPWithdrawParams {
    /// Serialize request parameters to BCS format for contract calls
    pub fn serialize_to_args(&self) -> Result<Vec<Vec<u8>>> {
        let args = vec![
            bcs::to_bytes(&self.withdraw_id)
                .map_err(|e| anyhow!("Failed to serialize withdraw_id: {}", e))?,
            bcs::to_bytes(&self.block_num)
                .map_err(|e| anyhow!("Failed to serialize block_num: {}", e))?,
            bcs::to_bytes(&self.tx_out_ix)
                .map_err(|e| anyhow!("Failed to serialize tx_out_ix: {}", e))?,
            bcs::to_bytes(&self.amount_sats)
                .map_err(|e| anyhow!("Failed to serialize amount_sats: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.block_header)
                .map_err(|e| anyhow!("Failed to serialize block_header: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.tx_id)
                .map_err(|e| anyhow!("Failed to serialize tx_id: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.tx_index)
                .map_err(|e| anyhow!("Failed to serialize tx_index: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.merkle_proof)
                .map_err(|e| anyhow!("Failed to serialize tx_merkle_proof: {}", e))?,
            bcs::to_bytes(&self.inclusion_proof.raw_tx)
                .map_err(|e| anyhow!("Failed to serialize raw_tx: {}", e))?,
        ];
        Ok(args)
    }
}

/// Request parameters for register_lp function
#[derive(Debug, Clone)]
pub struct RegisterLPParams {
    pub lp_id: u64,
    pub bitcoin_addr: String,
    pub lp_addr: String,
    pub lp_fee: u64,
}

impl RegisterLPParams {
    /// Serialize request parameters to BCS format for contract calls
    pub fn serialize_to_args(&self) -> Result<Vec<Vec<u8>>> {
        // Convert lp_addr string to AccountAddress
        let lp_addr = aptos_sdk::types::account_address::AccountAddress::from_str(&self.lp_addr)
            .map_err(|e| anyhow!("Invalid LP address format '{}': {}", self.lp_addr, e))?;

        let args = vec![
            bcs::to_bytes(&self.lp_id).map_err(|e| anyhow!("Failed to serialize lp_id: {}", e))?,
            bcs::to_bytes(&self.bitcoin_addr)
                .map_err(|e| anyhow!("Failed to serialize bitcoin_addr: {}", e))?,
            bcs::to_bytes(&lp_addr).map_err(|e| anyhow!("Failed to serialize lp_addr: {}", e))?,
            bcs::to_bytes(&self.lp_fee)
                .map_err(|e| anyhow!("Failed to serialize lp_fee: {}", e))?,
        ];
        Ok(args)
    }
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
