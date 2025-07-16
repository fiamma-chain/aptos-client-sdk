//! Utility functions module
//!
//! Provides various helper functions for type conversion, error handling, etc.

use std::str::FromStr;

use anyhow::{Context, Result};
use aptos_sdk::types::account_address::AccountAddress;

/// Convert hex string to AccountAddress
pub fn parse_account_address(addr_str: &str) -> Result<AccountAddress> {
    AccountAddress::from_str(addr_str)
        .with_context(|| format!("Invalid address format: {}", addr_str))
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
