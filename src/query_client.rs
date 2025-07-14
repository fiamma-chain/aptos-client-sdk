//! Query client implementation
//!
//! Provides functionality to query Aptos Bridge contract configuration and status.

use crate::types::{constants::*, BridgeConfig, BridgeError, BridgeResult, Peg};
use crate::utils::parse_account_address;

use aptos_sdk::{rest_client::Client, types::account_address::AccountAddress};
use serde_json::Value;
use std::str::FromStr;
use url::Url;

/// Query client
pub struct QueryClient {
    /// REST client
    rest_client: Client,
    /// Bridge contract address
    bridge_contract_address: AccountAddress,
}

impl QueryClient {
    /// Create new query client
    pub fn new(node_url: &str, bridge_contract_address: &str) -> BridgeResult<Self> {
        let rest_client = Client::new(
            Url::parse(node_url)
                .map_err(|e| BridgeError::Config(format!("Invalid node URL: {}", e)))?,
        );

        let bridge_contract_address = parse_account_address(bridge_contract_address)?;

        Ok(Self {
            rest_client,
            bridge_contract_address,
        })
    }

    /// Get bridge configuration
    pub async fn get_bridge_config(&self) -> BridgeResult<BridgeConfig> {
        // Parallel query of all configuration items
        let (
            owner,
            min_confirmations,
            max_pegs_per_mint,
            max_btc_per_mint,
            min_btc_per_mint,
            max_btc_per_burn,
            min_btc_per_burn,
            burn_paused,
            max_fee_rate,
        ) = tokio::try_join!(
            self.get_owner(),
            self.get_min_confirmations(),
            self.get_max_pegs_per_mint(),
            self.get_max_btc_per_mint(),
            self.get_min_btc_per_mint(),
            self.get_max_btc_per_burn(),
            self.get_min_btc_per_burn(),
            self.is_burn_paused(),
            self.get_max_fee_rate(),
        )?;

        Ok(BridgeConfig {
            owner,
            min_confirmations,
            max_pegs_per_mint,
            max_btc_per_mint,
            min_btc_per_mint,
            max_btc_per_burn,
            min_btc_per_burn,
            burn_paused,
            max_fee_rate,
        })
    }

    /// Get contract owner
    pub async fn get_owner(&self) -> BridgeResult<String> {
        self.call_view_function(GET_OWNER_FUNCTION, vec![]).await
    }

    /// Get minimum confirmations
    pub async fn get_min_confirmations(&self) -> BridgeResult<u64> {
        self.call_view_function(MIN_CONFIRMATIONS_FUNCTION, vec![])
            .await
    }

    /// Get maximum pegs per mint
    pub async fn get_max_pegs_per_mint(&self) -> BridgeResult<u64> {
        self.call_view_function(MAX_PEGS_PER_MINT_FUNCTION, vec![])
            .await
    }

    /// Get maximum BTC per mint
    pub async fn get_max_btc_per_mint(&self) -> BridgeResult<u64> {
        self.call_view_function(MAX_BTC_PER_MINT_FUNCTION, vec![])
            .await
    }

    /// Get minimum BTC per mint
    pub async fn get_min_btc_per_mint(&self) -> BridgeResult<u64> {
        self.call_view_function(MIN_BTC_PER_MINT_FUNCTION, vec![])
            .await
    }

    /// Get maximum BTC per burn
    pub async fn get_max_btc_per_burn(&self) -> BridgeResult<u64> {
        self.call_view_function(MAX_BTC_PER_BURN_FUNCTION, vec![])
            .await
    }

    /// Get minimum BTC per burn
    pub async fn get_min_btc_per_burn(&self) -> BridgeResult<u64> {
        self.call_view_function(MIN_BTC_PER_BURN_FUNCTION, vec![])
            .await
    }

    /// Check if burn is paused
    pub async fn is_burn_paused(&self) -> BridgeResult<bool> {
        self.call_view_function(BURN_PAUSED_FUNCTION, vec![]).await
    }

    /// Get maximum fee rate
    pub async fn get_max_fee_rate(&self) -> BridgeResult<u64> {
        self.call_view_function(MAX_FEE_RATE_FUNCTION, vec![]).await
    }

    /// Get total minted amount
    pub async fn get_minted_amount(&self) -> BridgeResult<u64> {
        self.call_view_function(GET_MINTED_FUNCTION, vec![]).await
    }

    /// Validate mint parameters
    pub async fn validate_mint_params(&self, pegs: &[Peg]) -> BridgeResult<()> {
        let config = self.get_bridge_config().await?;

        // Check number of pegs
        if pegs.len() > config.max_pegs_per_mint as usize {
            return Err(BridgeError::Other(format!(
                "Too many pegs: {} > {}",
                pegs.len(),
                config.max_pegs_per_mint
            )));
        }

        // Check total amount
        let total_amount: u64 = pegs.iter().map(|p| p.value).sum();
        if total_amount > config.max_btc_per_mint {
            return Err(BridgeError::Other(format!(
                "Total amount too large: {} > {}",
                total_amount, config.max_btc_per_mint
            )));
        }

        if total_amount < config.min_btc_per_mint {
            return Err(BridgeError::Other(format!(
                "Total amount too small: {} < {}",
                total_amount, config.min_btc_per_mint
            )));
        }

        Ok(())
    }

    /// Validate burn parameters
    pub async fn validate_burn_params(
        &self,
        btc_address: &str,
        fee_rate: u64,
        amount: u64,
        operator_id: u64,
    ) -> BridgeResult<()> {
        let config = self.get_bridge_config().await?;

        // Check if burn is paused
        if config.burn_paused {
            return Err(BridgeError::Other(
                "Burn functionality is paused".to_string(),
            ));
        }

        // Check amount limits
        if amount > config.max_btc_per_burn {
            return Err(BridgeError::Other(format!(
                "Amount too large: {} > {}",
                amount, config.max_btc_per_burn
            )));
        }

        if amount < config.min_btc_per_burn {
            return Err(BridgeError::Other(format!(
                "Amount too small: {} < {}",
                amount, config.min_btc_per_burn
            )));
        }

        // Check fee rate
        if fee_rate > config.max_fee_rate {
            return Err(BridgeError::Other(format!(
                "Fee rate too high: {} > {}",
                fee_rate, config.max_fee_rate
            )));
        }

        // Validate BTC address
        crate::utils::validate_btc_address(btc_address)?;

        Ok(())
    }

    /// Check if a proof has been used
    pub async fn is_proof_used(&self, proof_hash: &str) -> BridgeResult<bool> {
        // This is a placeholder implementation
        // In a real implementation, you would query the contract state
        Ok(false)
    }

    /// Get suggested fee rate
    pub async fn get_suggested_fee_rate(&self) -> BridgeResult<u64> {
        // This is a placeholder implementation
        // In a real implementation, you might query external APIs or use heuristics
        Ok(100)
    }

    /// Get contract address as hex string
    pub fn get_contract_address_hex(&self) -> String {
        crate::utils::account_address_to_hex(&self.bridge_contract_address)
    }

    /// Print bridge configuration in a formatted way
    pub async fn print_bridge_config(&self) -> BridgeResult<()> {
        let config = self.get_bridge_config().await?;

        println!("Bridge Configuration:");
        println!("  Owner: {}", config.owner);
        println!("  Min Confirmations: {}", config.min_confirmations);
        println!("  Max Pegs per Mint: {}", config.max_pegs_per_mint);
        println!(
            "  Max BTC per Mint: {}",
            crate::utils::format_btc_amount(config.max_btc_per_mint)
        );
        println!(
            "  Min BTC per Mint: {}",
            crate::utils::format_btc_amount(config.min_btc_per_mint)
        );
        println!(
            "  Max BTC per Burn: {}",
            crate::utils::format_btc_amount(config.max_btc_per_burn)
        );
        println!(
            "  Min BTC per Burn: {}",
            crate::utils::format_btc_amount(config.min_btc_per_burn)
        );
        println!("  Burn Paused: {}", config.burn_paused);
        println!("  Max Fee Rate: {}", config.max_fee_rate);

        Ok(())
    }

    /// Generic method to call view functions
    async fn call_view_function<T>(&self, function_name: &str, args: Vec<Value>) -> BridgeResult<T>
    where
        T: serde::de::DeserializeOwned + FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        // This is a placeholder implementation
        // In a real implementation, you would:
        // 1. Create a ViewRequest
        // 2. Call the view function
        // 3. Parse the result

        // For now, return default values based on function name
        match function_name {
            GET_OWNER_FUNCTION => {
                let owner = "0x1".to_string();
                serde_json::from_value(serde_json::Value::String(owner))
                    .map_err(|e| BridgeError::Json(e))
            }
            MIN_CONFIRMATIONS_FUNCTION => {
                serde_json::from_value(serde_json::Value::Number(6.into()))
                    .map_err(|e| BridgeError::Json(e))
            }
            MAX_PEGS_PER_MINT_FUNCTION => {
                serde_json::from_value(serde_json::Value::Number(10.into()))
                    .map_err(|e| BridgeError::Json(e))
            }
            MAX_BTC_PER_MINT_FUNCTION => {
                serde_json::from_value(serde_json::Value::Number(1000000000u64.into()))
                    .map_err(|e| BridgeError::Json(e))
            }
            MIN_BTC_PER_MINT_FUNCTION => {
                serde_json::from_value(serde_json::Value::Number(1000000u64.into()))
                    .map_err(|e| BridgeError::Json(e))
            }
            MAX_BTC_PER_BURN_FUNCTION => {
                serde_json::from_value(serde_json::Value::Number(1000000000u64.into()))
                    .map_err(|e| BridgeError::Json(e))
            }
            MIN_BTC_PER_BURN_FUNCTION => {
                serde_json::from_value(serde_json::Value::Number(1000000u64.into()))
                    .map_err(|e| BridgeError::Json(e))
            }
            BURN_PAUSED_FUNCTION => serde_json::from_value(serde_json::Value::Bool(false))
                .map_err(|e| BridgeError::Json(e)),
            MAX_FEE_RATE_FUNCTION => {
                serde_json::from_value(serde_json::Value::Number(1000u64.into()))
                    .map_err(|e| BridgeError::Json(e))
            }
            GET_MINTED_FUNCTION => serde_json::from_value(serde_json::Value::Number(0u64.into()))
                .map_err(|e| BridgeError::Json(e)),
            _ => Err(BridgeError::Other(format!(
                "Unknown function: {}",
                function_name
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_client_creation() {
        let node_url = "https://fullnode.devnet.aptoslabs.com/v1";
        let contract_address = "0x1";

        let result = QueryClient::new(node_url, contract_address);
        assert!(result.is_ok());
    }
}
