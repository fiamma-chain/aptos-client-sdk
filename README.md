# Fiamma Bitvm Bridge Aptos Client SDK

A Rust SDK for interacting with Fiamma Bitvm Bridge contracts, enabling cross-chain operations between Aptos and Bitcoin.

## Features

- **Bridge Operations**: Mint and burn tokens across different blockchains
- **Event Monitoring**: Listen to bridge events in real-time
- **Query Client**: Query bridge state and transaction information
- **Type Safety**: Strongly typed interfaces for all bridge operations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aptos-client-sdk = { git = "https://github.com/your-repo/aptos-client-sdk" }
```

## Quick Start

### Basic Setup

```rust
use aptos_client_sdk::BridgeClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
    let private_key = "your_private_key";
    let bridge_contract_address = "0x6b891d58da6e4fd7bb2ab229917833c47cb34d8d60cf75e93d717bda43eee387";
    let btc_light_client = "0x67dd32fe9ee2e6d7c6016d51d912f5c7cf02032e9fe94b9c2db1b2762196952d";

    let mut bridge_client = BridgeClient::new(
        &node_url,
        private_key,
        bridge_contract_address,
        btc_light_client,
    ).await?;

    Ok(())
}
```

### Minting Tokens

```rust
use aptos_bridge_sdk::{BridgeClient, types::{Peg, ScriptType, TxProof}};

// Initialize bridge client
let mut bridge_client = BridgeClient::new(...).await?;

// Create peg and transaction proof
let peg = Peg { /* peg configuration */ };
let tx_proof = TxProof { /* transaction proof */ };

// Mint tokens
bridge_client.mint(peg, tx_proof).await?;
```

### Burning Tokens

```rust
// Burn tokens
let amount = 1000000; // Amount in smallest unit
bridge_client.burn(amount, "destination_address").await?;
```

### Event Monitoring

```rust
use aptos_bridge_sdk::{EventMonitor, EventHandler};

let event_monitor = EventMonitor::new(&node_url, &bridge_contract_address).await?;

// Listen for bridge events
event_monitor.start_listening(|event| {
    println!("Received bridge event: {:?}", event);
}).await?;
```

## Examples

The `examples/` directory contains complete working examples:

- **`mint.rs`**: Demonstrates how to mint tokens through the bridge
- **`burn.rs`**: Shows how to burn tokens for cross-chain transfer
- **`event_listener.rs`**: Example of monitoring bridge events
- **`query.rs`**: Querying bridge state and transaction information

To run an example:

```bash
# Set up environment variables
export PRIVATE_KEY="your_private_key_here"

# Run the mint example
cargo run --example mint

# Run the burn example
cargo run --example burn

# Run the event listener
cargo run --example event_listener

# Run the query example
cargo run --example query
```

## Environment Variables

Make sure to set the following environment variables:

- `PRIVATE_KEY`: Your Aptos account private key

You can also create a `.env` file in the project root:

```env
PRIVATE_KEY=your_private_key_here
```

## API Reference

### Core Types

- **`BridgeClient`**: Main client for bridge operations
- **`QueryClient`**: Client for querying bridge state
- **`EventMonitor`**: Real-time event monitoring
- **`BridgeEvent`**: Bridge event data structure
- **`Peg`**: Peg configuration for cross-chain operations
- **`TxProof`**: Transaction proof for verification

### Main Functions

- `BridgeClient::new()`: Initialize a new bridge client
- `BridgeClient::mint()`: Mint tokens on Aptos
- `BridgeClient::burn()`: Burn tokens for cross-chain transfer
- `QueryClient::get_bridge_state()`: Query current bridge state
- `EventMonitor::start_listening()`: Start monitoring bridge events

## Requirements

- Rust 1.70+
- Tokio runtime for async operations
- Access to an Aptos node (testnet or mainnet)

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For questions and support, please open an issue on GitHub. 