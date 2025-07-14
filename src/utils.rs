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

/// Convert AccountAddress to hex string
pub fn account_address_to_hex(addr: &AccountAddress) -> String {
    format!("0x{}", addr.to_hex())
}

/// Convert byte array to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert hex string to byte array
pub fn hex_to_bytes(hex_str: &str) -> BridgeResult<Vec<u8>> {
    let hex_str = if hex_str.starts_with("0x") {
        &hex_str[2..]
    } else {
        hex_str
    };

    hex::decode(hex_str).map_err(|e| BridgeError::Other(format!("Invalid hex string: {}", e)))
}

/// Format BTC amount (satoshi to BTC)
pub fn format_btc_amount(satoshi: u64) -> String {
    let btc = satoshi as f64 / 100_000_000.0;
    format!("{:.8} BTC", btc)
}

/// Parse BTC amount (BTC to satoshi)
pub fn parse_btc_amount(btc_str: &str) -> BridgeResult<u64> {
    let btc_str = btc_str.trim().to_lowercase();
    let btc_str = if btc_str.ends_with(" btc") {
        &btc_str[..btc_str.len() - 4]
    } else {
        &btc_str
    };

    let btc: f64 = btc_str
        .parse()
        .map_err(|_| BridgeError::Other("Invalid BTC amount format".to_string()))?;

    if btc < 0.0 {
        return Err(BridgeError::Other(
            "BTC amount cannot be negative".to_string(),
        ));
    }

    Ok((btc * 100_000_000.0) as u64)
}

/// Retry operation with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay: std::time::Duration,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;
    let mut last_error = None;

    for attempt in 0..max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt == max_retries - 1 {
                    break;
                }

                println!(
                    "Attempt {} failed: {}. Retrying in {:?}...",
                    attempt + 1,
                    last_error.as_ref().unwrap(),
                    delay
                );
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    Err(last_error.unwrap())
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

/// Calculate transaction hash
pub fn calculate_tx_hash(tx_data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(tx_data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Convert timestamp to readable string
pub fn timestamp_to_string(timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
    match duration.duration_since(UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            let hours = secs / 3600;
            let minutes = (secs % 3600) / 60;
            let seconds = secs % 60;
            format!("{}h {}m {}s ago", hours, minutes, seconds)
        }
        Err(_) => "Unknown time".to_string(),
    }
}

/// Validate fee rate
pub fn validate_fee_rate(fee_rate: u64, max_fee_rate: u64) -> BridgeResult<()> {
    if fee_rate == 0 {
        return Err(BridgeError::Other("Fee rate cannot be zero".to_string()));
    }

    if fee_rate > max_fee_rate {
        return Err(BridgeError::Other(format!(
            "Fee rate {} exceeds maximum {}",
            fee_rate, max_fee_rate
        )));
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
    fn test_format_btc_amount() {
        assert_eq!(format_btc_amount(100000000), "1.00000000 BTC");
        assert_eq!(format_btc_amount(50000000), "0.50000000 BTC");
    }

    #[test]
    fn test_parse_btc_amount() {
        assert_eq!(parse_btc_amount("1.0").unwrap(), 100000000);
        assert_eq!(parse_btc_amount("0.5 btc").unwrap(), 50000000);
        assert!(parse_btc_amount("invalid").is_err());
    }

    #[test]
    fn test_validate_btc_address() {
        assert!(validate_btc_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa").is_ok());
        assert!(validate_btc_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy").is_ok());
        assert!(validate_btc_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4").is_ok());
        assert!(validate_btc_address("").is_err());
        assert!(validate_btc_address("invalid").is_err());
    }

    #[test]
    fn test_validate_fee_rate() {
        assert!(validate_fee_rate(100, 1000).is_ok());
        assert!(validate_fee_rate(0, 1000).is_err());
        assert!(validate_fee_rate(2000, 1000).is_err());
    }
}
