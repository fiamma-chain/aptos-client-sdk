//! Core data type definitions
//!
//! This module defines all data types required for interacting with Aptos Bridge contracts.

use anyhow::Result;
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

    #[error("Event parsing failed: {0}")]
    EventParseFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Other error: {0}")]
    Other(String),
}

/// Custom Result type
pub type BridgeResult<T> = std::result::Result<T, BridgeError>;

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction submitted but not confirmed
    Pending,
    /// Transaction confirmed successfully
    Success,
    /// Transaction failed
    Failed { reason: String },
    /// Transaction rejected
    Rejected { reason: String },
}

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
    /// Transaction hash
    pub transaction_hash: String,
    /// Block timestamp
    pub block_timestamp: u64,
    /// Event sequence number
    pub sequence_number: u64,
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
    /// Transaction hash
    pub transaction_hash: String,
    /// Block timestamp
    pub block_timestamp: u64,
    /// Event sequence number
    pub sequence_number: u64,
}

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Contract owner address
    pub owner: String,
    /// Minimum confirmations
    pub min_confirmations: u64,
    /// Maximum pegs per mint
    pub max_pegs_per_mint: u64,
    /// Maximum BTC per mint
    pub max_btc_per_mint: u64,
    /// Minimum BTC per mint
    pub min_btc_per_mint: u64,
    /// Maximum BTC per burn
    pub max_btc_per_burn: u64,
    /// Minimum BTC per burn
    pub min_btc_per_burn: u64,
    /// Whether burn is paused
    pub burn_paused: bool,
    /// Maximum fee rate
    pub max_fee_rate: u64,
}

/// Bridge event enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEvent {
    /// Mint event
    Mint(MintEvent),
    /// Burn event
    Burn(BurnEvent),
}

impl fmt::Display for BridgeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeEvent::Mint(event) => {
                write!(f, "Mint: {} satoshi to {}", event.amount, event.to)
            }
            BridgeEvent::Burn(event) => {
                write!(
                    f,
                    "Burn: {} satoshi from {} to {}",
                    event.amount, event.from, event.btc_address
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
    pub merkle_proof: Vec<Vec<u8>>,
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
    /// Mint event type
    pub const MINT_EVENT_TYPE: &str = "0x1::bridge::Mint";
    /// Burn event type
    pub const BURN_EVENT_TYPE: &str = "0x1::bridge::Burn";

    /// Mint function name
    pub const MINT_FUNCTION: &str = "mint";
    /// Burn function name
    pub const BURN_FUNCTION: &str = "burn";

    pub const EXPIRATION_TIMESTAMP_SECS: u64 = 60;

    /// Configuration query function names
    pub const GET_OWNER_FUNCTION: &str = "get_owner";
    pub const MIN_CONFIRMATIONS_FUNCTION: &str = "min_confirmations";
    pub const MAX_PEGS_PER_MINT_FUNCTION: &str = "max_pegs_per_mint";
    pub const MAX_BTC_PER_MINT_FUNCTION: &str = "max_btc_per_mint";
    pub const MIN_BTC_PER_MINT_FUNCTION: &str = "min_btc_per_mint";
    pub const MAX_BTC_PER_BURN_FUNCTION: &str = "max_btc_per_burn";
    pub const MIN_BTC_PER_BURN_FUNCTION: &str = "min_btc_per_burn";
    pub const BURN_PAUSED_FUNCTION: &str = "burn_paused";
    pub const MAX_FEE_RATE_FUNCTION: &str = "max_fee_rate";
    pub const GET_MINTED_FUNCTION: &str = "get_minted";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peg_conversion() {
        let peg = Peg {
            to: "0x1".to_string(),
            value: 100000000,
            block_num: 123456,
            inclusion_proof: TxProof {
                block_header: vec![1, 2, 3],
                tx_id: vec![4, 5, 6],
                tx_index: 0,
                merkle_proof: vec![vec![7, 8, 9]],
                raw_tx: vec![10, 11, 12],
            },
            tx_out_ix: 0,
            dest_script_hash: vec![13, 14, 15],
            script_type: ScriptType::P2PKH,
        };

        let peg_bcs = PegForBcs::try_from(&peg).unwrap();
        assert_eq!(peg_bcs.value, 100000000);
        assert_eq!(peg_bcs.block_num, 123456);
        assert_eq!(peg_bcs.script_type, 0);
    }

    #[test]
    fn test_event_display() {
        let mint_event = MintEvent {
            to: "0x1".to_string(),
            amount: 100000000,
            tx_id: vec![1, 2, 3],
            block_num: 123456,
            transaction_hash: "hash".to_string(),
            block_timestamp: 1234567890,
            sequence_number: 1,
        };

        let event = BridgeEvent::Mint(mint_event);
        assert!(event.to_string().contains("Mint"));
        assert!(event.to_string().contains("100000000"));
    }
}
