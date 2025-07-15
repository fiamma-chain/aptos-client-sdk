//! Utility functions module
//!
//! Provides various helper functions for type conversion, error handling, etc.

use std::str::FromStr;

use anyhow::{bail, Context, Result};
use aptos_sdk::types::account_address::AccountAddress;

/// Convert hex string to AccountAddress
pub fn parse_account_address(addr_str: &str) -> Result<AccountAddress> {
    AccountAddress::from_str(addr_str)
        .with_context(|| format!("Invalid address format: {}", addr_str))
}

/// Validate BTC address format
pub fn validate_btc_address(address: &str) -> Result<()> {
    // Simple BTC address format validation
    if address.is_empty() {
        bail!("BTC address cannot be empty");
    }

    // Check for common BTC address prefixes
    if !address.starts_with("1")
        && !address.starts_with("3")
        && !address.starts_with("bc1")
        && !address.starts_with("tb1")
    {
        bail!("Invalid BTC address format");
    }

    // Basic length check
    if address.len() < 26 || address.len() > 62 {
        bail!("BTC address length is invalid");
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

    #[test]
    fn test_validate_btc_address() {
        // Test valid addresses
        assert!(validate_btc_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa").is_ok());
        assert!(validate_btc_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy").is_ok());
        assert!(validate_btc_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4").is_ok());

        // Test invalid addresses
        assert!(validate_btc_address("").is_err());
        assert!(validate_btc_address("invalid").is_err());
        assert!(validate_btc_address(&"x".repeat(70)).is_err()); // Too long
        assert!(validate_btc_address("1A1z").is_err()); // Too short
    }
}
