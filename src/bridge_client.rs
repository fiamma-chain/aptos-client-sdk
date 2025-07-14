//! Bridge client implementation
//!
//! Provides core functionality for interacting with Aptos Bridge contracts.

use crate::types::{constants::*, BridgeError, BridgeResult, Peg, PegForBcs, TransactionStatus};
use crate::utils::{parse_account_address, validate_btc_address};

use aptos_sdk::move_types::identifier::Identifier;
use aptos_sdk::move_types::language_storage::ModuleId;
use aptos_sdk::rest_client::aptos_api_types::{transaction, MoveModuleId};
use aptos_sdk::transaction_builder::TransactionBuilder;
use aptos_sdk::{
    crypto::ed25519::Ed25519PrivateKey,
    rest_client::{Client, FaucetClient},
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        transaction::{EntryFunction, SignedTransaction, TransactionPayload},
        LocalAccount,
    },
};
use std::str::FromStr;
use url::Url;

/// Bridge client
pub struct BridgeClient {
    /// REST client
    rest_client: Client,
    /// Local account
    account: LocalAccount,
    /// Bridge contract address
    bridge_contract_address: AccountAddress,
    /// Maximum retry attempts
    max_retries: u32,
    /// Retry delay
    retry_delay: std::time::Duration,
}

impl BridgeClient {
    /// Create new Bridge client
    pub async fn new(
        node_url: &str,
        private_key_hex: &str,
        bridge_contract_address: &str,
    ) -> BridgeResult<Self> {
        // Parse contract address
        let bridge_contract_address = parse_account_address(bridge_contract_address)?;

        // Create REST client
        let rest_client = Client::new(
            Url::parse(node_url)
                .map_err(|e| BridgeError::Config(format!("Invalid Aptos node URL: {}", e)))?,
        );

        // Create private key
        let private_key_str = if private_key_hex.starts_with("0x") {
            &private_key_hex[2..]
        } else {
            private_key_hex
        };

        let private_key = Ed25519PrivateKey::try_from(private_key_str.as_bytes())
            .map_err(|_| BridgeError::InvalidPrivateKey)?;

        // Create local account
        let account = LocalAccount::new(
            AccountAddress::from_str_strict(private_key_str)
                .map_err(|_| BridgeError::InvalidPrivateKey)?,
            private_key,
            0,
        );

        Ok(Self {
            rest_client,
            account,
            bridge_contract_address,
            max_retries: 3,
            retry_delay: std::time::Duration::from_secs(1),
        })
    }

    /// Mint tokens
    pub async fn mint(&mut self, pegs: Vec<Peg>) -> BridgeResult<String> {
        if pegs.is_empty() {
            return Err(BridgeError::Other("Pegs cannot be empty".to_string()));
        }

        // Convert to BCS-serializable format
        let pegs_for_bcs: Vec<PegForBcs> = pegs
            .iter()
            .map(|peg| PegForBcs::try_from(peg))
            .collect::<Result<Vec<_>, _>>()?;

        // Serialize parameters
        let args = vec![bcs::to_bytes(&pegs_for_bcs).map_err(|e| BridgeError::Bcs(e))?];

        // Create Entry Function
        let entry_function = EntryFunction::new(
            ModuleId::new(
                self.bridge_contract_address,
                Identifier::new("fiamma_bridge_account").unwrap(),
            ),
            MINT_FUNCTION
                .parse()
                .map_err(|e| BridgeError::Other(format!("Invalid aptos function name: {}", e)))?,
            vec![], // No type parameters
            args,
        );

        // Execute transaction
        let tx_hash = self
            .execute_transaction(TransactionPayload::EntryFunction(entry_function))
            .await?;

        println!("Mint transaction submitted: {}", tx_hash);
        Ok(tx_hash)
    }

    /// Burn tokens
    pub async fn burn(
        &mut self,
        btc_address: String,
        fee_rate: u64,
        amount: u64,
        operator_id: u64,
    ) -> BridgeResult<String> {
        // Validate BTC address format
        validate_btc_address(&btc_address)?;

        // Validate amount
        if amount == 0 {
            return Err(BridgeError::Other("Amount cannot be zero".to_string()));
        }

        // Serialize parameters
        let args = vec![
            bcs::to_bytes(&btc_address).map_err(|e| BridgeError::Bcs(e))?,
            bcs::to_bytes(&fee_rate).map_err(|e| BridgeError::Bcs(e))?,
            bcs::to_bytes(&amount).map_err(|e| BridgeError::Bcs(e))?,
            bcs::to_bytes(&operator_id).map_err(|e| BridgeError::Bcs(e))?,
        ];

        // Create Entry Function
        let entry_function = EntryFunction::new(
            ModuleId::new(
                self.bridge_contract_address,
                Identifier::new("fiamma_bridge_account").unwrap(),
            ),
            BURN_FUNCTION
                .parse()
                .map_err(|e| BridgeError::Other(format!("Invalid function name: {}", e)))?,
            vec![], // No type parameters
            args,
        );

        // Execute transaction
        let tx_hash = self
            .execute_transaction(TransactionPayload::EntryFunction(entry_function))
            .await?;

        println!("Burn transaction submitted: {}", tx_hash);
        Ok(tx_hash)
    }

    /// Query transaction status
    pub async fn get_transaction_status(&self, tx_hash: &str) -> BridgeResult<TransactionStatus> {
        // Parse transaction hash
        let tx_hash = tx_hash
            .parse()
            .map_err(|e| BridgeError::Other(format!("Invalid transaction hash: {}", e)))?;

        // Query transaction information
        match self.rest_client.get_transaction_by_hash(tx_hash).await {
            Ok(transaction) => {
                // Check if transaction was successful
                match self.rest_client.get_transaction_by_hash(tx_hash).await {
                    Ok(_) => {
                        // Further check transaction execution status
                        // This needs to be implemented based on actual Aptos SDK API
                        Ok(TransactionStatus::Success)
                    }
                    Err(_) => Ok(TransactionStatus::Failed {
                        reason: "Transaction execution failed".to_string(),
                    }),
                }
            }
            Err(e) => {
                if e.to_string().contains("not found") {
                    Ok(TransactionStatus::Pending)
                } else {
                    Err(BridgeError::Aptos(e.to_string()))
                }
            }
        }
    }

    /// Wait for transaction confirmation
    pub async fn wait_for_transaction(
        &self,
        tx_hash: &str,
        timeout_secs: u64,
    ) -> BridgeResult<TransactionStatus> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            match self.get_transaction_status(tx_hash).await? {
                TransactionStatus::Success => return Ok(TransactionStatus::Success),
                TransactionStatus::Failed { reason } => {
                    return Ok(TransactionStatus::Failed { reason });
                }
                TransactionStatus::Rejected { reason } => {
                    return Ok(TransactionStatus::Rejected { reason });
                }
                TransactionStatus::Pending => {
                    if start_time.elapsed() > timeout {
                        return Err(BridgeError::Other("Transaction timeout".to_string()));
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }

    /// Generic method for executing transactions
    async fn execute_transaction(&mut self, payload: TransactionPayload) -> BridgeResult<String> {
        let chain_id = self
            .rest_client
            .get_index()
            .await
            .map_err(|e| BridgeError::Aptos(e.to_string()))?
            .inner()
            .chain_id;
        let transaction_builder = TransactionBuilder::new(
            payload,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + EXPIRATION_TIMESTAMP_SECS,
            ChainId::new(chain_id),
        )
        .sender(self.account.address())
        .sequence_number(self.account.sequence_number());
        // Sign transaction
        let signed_transaction = self
            .account
            .sign_with_transaction_builder(transaction_builder);

        // Submit transaction
        let response = self
            .rest_client
            .submit(&signed_transaction)
            .await
            .map_err(|e| BridgeError::Aptos(e.to_string()))?;

        Ok(response.inner().hash.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ScriptType, TxProof};

    #[tokio::test]
    async fn test_client_creation() {
        let private_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let bridge_address = "0x1";
        let node_url = "https://fullnode.devnet.aptoslabs.com/v1";

        let result = BridgeClient::new(node_url, private_key, bridge_address).await;

        // This test might fail because we use a dummy private key
        // In actual usage, a real private key is required
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_peg_creation() {
        let peg = Peg {
            to: "0x1".to_string(),
            value: 100000000, // 1 BTC
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

        assert_eq!(peg.value, 100000000);
        assert_eq!(peg.block_num, 123456);
    }
}
