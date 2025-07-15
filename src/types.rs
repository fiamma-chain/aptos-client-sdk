//! Core data type definitions
//!
//! This module defines all data types required for interacting with Aptos Bridge contracts.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Custom error type
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("BCS serialization error: {0}")]
    Bcs(#[from] bcs::Error),

    #[error("Aptos SDK error: {0}")]
    Aptos(String),

    #[error("Invalid private key format")]
    InvalidPrivateKey,

    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Fetch events error: {0}")]
    FetchEventsError(String),

    #[error("Event parsing failed: {0}")]
    EventParseFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Other error: {0}")]
    Other(String),
}

/// Custom Result type
pub type BridgeResult<T> = std::result::Result<T, BridgeError>;

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
    /// P2PKH (Pay to Public Key Hash)
    P2PKH = 0,
    /// P2SH (Pay to Script Hash)
    P2SH = 1,
    /// P2WPKH (Pay to Witness Public Key Hash)
    P2WPKH = 2,
    /// P2WSH (Pay to Witness Script Hash)
    P2WSH = 3,
    /// P2TR (Pay to Taproot)
    P2TR = 4,
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

impl fmt::Display for BridgeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeEvent::Mint(event) => {
                write!(
                    f,
                    "Mint: {} satoshi to {} at (version: {}, event sequence_number: {})",
                    event.event.amount, event.event.to, event.version, event.sequence_number
                )
            }
            BridgeEvent::Burn(event) => {
                write!(
                    f,
                    "Burn: {} satoshi from {} to {} at (version: {}, event sequence_number: {})",
                    event.event.amount,
                    event.event.from,
                    event.event.btc_address,
                    event.version,
                    event.sequence_number
                )
            }
        }
    }
}

/// BCS-serializable Peg structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegForBcs {
    pub to: aptos_sdk::types::account_address::AccountAddress,
    pub value: u64,
    pub block_num: u64,
    pub inclusion_proof: TxProofForBcs,
    pub tx_out_ix: u64,
    pub dest_script_hash: Vec<u8>,
    pub script_type: u8,
}

/// BCS-serializable TxProof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxProofForBcs {
    pub block_header: Vec<u8>,
    pub tx_id: Vec<u8>,
    pub tx_index: u64,
    pub merkle_proof: Vec<u8>,
    pub raw_tx: Vec<u8>,
}

impl TryFrom<&Peg> for PegForBcs {
    type Error = BridgeError;

    fn try_from(peg: &Peg) -> BridgeResult<Self> {
        let to = aptos_sdk::types::account_address::AccountAddress::from_str_strict(&peg.to)
            .map_err(|_| BridgeError::InvalidAddress(peg.to.clone()))?;

        let inclusion_proof = TxProofForBcs {
            block_header: peg.inclusion_proof.block_header.clone(),
            tx_id: peg.inclusion_proof.tx_id.clone(),
            tx_index: peg.inclusion_proof.tx_index,
            merkle_proof: peg.inclusion_proof.merkle_proof.clone(),
            raw_tx: peg.inclusion_proof.raw_tx.clone(),
        };

        let script_type = match peg.script_type {
            ScriptType::P2PKH => 0,
            ScriptType::P2SH => 1,
            ScriptType::P2WPKH => 2,
            ScriptType::P2WSH => 3,
            ScriptType::P2TR => 4,
        };

        Ok(PegForBcs {
            to,
            value: peg.value,
            block_num: peg.block_num,
            inclusion_proof,
            tx_out_ix: peg.tx_out_ix,
            dest_script_hash: peg.dest_script_hash.clone(),
            script_type,
        })
    }
}

/// Constants module
pub mod constants {
    pub const EXPIRATION_TIMESTAMP_SECS: u64 = 60;
}
