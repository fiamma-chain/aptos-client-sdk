//! Utility functions module
//!
//! Provides various helper functions for type conversion, error handling, etc.

use crate::types::{BridgeError, BridgeResult};
use aptos_sdk::types::account_address::AccountAddress;

/// Convert hex string to AccountAddress
pub fn parse_account_address(addr_str: &str) -> BridgeResult<AccountAddress> {
    AccountAddress::from_str_strict(addr_str)
        .map_err(|_| BridgeError::InvalidAddress(addr_str.to_string()))
}

/// Validate BTC address format
pub fn validate_btc_address(address: &str) -> BridgeResult<()> {
    // Simple BTC address format validation
    if address.is_empty() {
        return Err(BridgeError::Other(
            "BTC address cannot be empty".to_string(),
        ));
    }

    // Check for common BTC address prefixes
    if !address.starts_with("1")
        && !address.starts_with("3")
        && !address.starts_with("bc1")
        && !address.starts_with("tb1")
    {
        return Err(BridgeError::Other("Invalid BTC address format".to_string()));
    }

    // Basic length check
    if address.len() < 26 || address.len() > 62 {
        return Err(BridgeError::Other(
            "BTC address length is invalid".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_account_address() {
        let addr = "0x1";
        let result = parse_account_address(addr);
        assert!(result.is_ok());

        let invalid_addr = "invalid";
        let result = parse_account_address(invalid_addr);
        assert!(result.is_err());
    }
}
