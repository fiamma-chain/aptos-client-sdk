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

/// Parse LP withdraw data using serde_json
pub fn parse_lp_withdraw(data: &serde_json::Value) -> Result<LPWithdraw> {
    let raw_data: LPWithdrawRaw = serde_json::from_value(data.clone())
        .map_err(|e| anyhow!("Failed to parse LP withdraw data: {}", e))?;
    Ok(raw_data.into())
}

/// LP Status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPStatus {
    pub value: u8,
}

/// LP Status raw structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPStatusRaw {
    #[serde(deserialize_with = "deserialize_number_as_string")]
    pub value: String,
}

/// Custom deserializer to handle both numeric and string values
fn deserialize_number_as_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringOrNumberVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number")
        }

        fn visit_str<E>(self, value: &str) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_u64<E>(self, value: u64) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_i64<E>(self, value: i64) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_f64<E>(self, value: f64) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

impl From<LPStatusRaw> for LPStatus {
    fn from(raw: LPStatusRaw) -> Self {
        Self {
            value: raw.value.parse().unwrap_or(0),
        }
    }
}

impl LPStatus {
    /// Parse LP status data from view function response
    pub fn from_view_response(result: &serde_json::Value) -> Result<Self> {
        // Try to parse as direct number
        if let Some(num) = result.as_u64() {
            return Ok(LPStatus { value: num as u8 });
        }

        // Try to parse as object first (most likely format)
        if let Ok(raw) = serde_json::from_value::<LPStatusRaw>(result.clone()) {
            return Ok(raw.into());
        }

        // Fallback: try to parse as array of strings
        if let Ok(status_data) = serde_json::from_value::<Vec<String>>(result.clone()) {
            let raw = LPStatusRaw {
                value: status_data.get(0).unwrap_or(&"0".to_string()).clone(),
            };
            return Ok(raw.into());
        }

        // Fallback: try to parse as single value
        if let Some(value_str) = result.as_str() {
            let raw = LPStatusRaw {
                value: value_str.to_string(),
            };
            return Ok(raw.into());
        }

        Err(anyhow!(
            "Failed to parse get_lp_status response: unsupported format. Response: {}",
            result
        ))
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
    pub receiver_script_hash: Vec<u8>,
    pub receive_min_amount: u64,
    pub fee_rate: u64,
    pub timestamp: u64,
    pub lp_id: u64,
}

/// LP withdraw raw information structure
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
            receiver_script_hash: hex::decode(raw.receiver_script_hash.trim_start_matches("0x"))
                .unwrap_or_default(),
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
        // Try to parse as object first (most likely format)
        if let Ok(raw) = serde_json::from_value::<LPWithdrawRaw>(result.clone()) {
            return Ok(raw.into());
        }

        // Fallback: try to parse as array of strings
        if let Ok(lp_withdraw_data) = serde_json::from_value::<Vec<String>>(result.clone()) {
            let raw = LPWithdrawRaw {
                id: lp_withdraw_data.get(0).unwrap_or(&"0".to_string()).clone(),
                withdraw_amount: lp_withdraw_data.get(1).unwrap_or(&"0".to_string()).clone(),
                receiver_addr: lp_withdraw_data.get(2).unwrap_or(&"".to_string()).clone(),
                receiver_script_hash: lp_withdraw_data.get(3).unwrap_or(&"".to_string()).clone(),
                receive_min_amount: lp_withdraw_data.get(4).unwrap_or(&"0".to_string()).clone(),
                fee_rate: lp_withdraw_data.get(5).unwrap_or(&"0".to_string()).clone(),
                timestamp: lp_withdraw_data.get(6).unwrap_or(&"0".to_string()).clone(),
                lp_id: lp_withdraw_data.get(7).unwrap_or(&"0".to_string()).clone(),
            };
            return Ok(raw.into());
        }

        Err(anyhow!(
            "Failed to parse get_lp_withdraw response: unsupported format. Response: {}",
            result
        ))
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
