# Aptos Bridge SDK

一个用于与 Aptos 区块链上的 Bitcoin Bridge 合约交互的 Rust SDK。

## 概述

这个 SDK 提供了与基于 BitVM 技术的 Bitcoin Bridge 合约交互的完整功能，包括：

- 🔄 **Mint 操作**: 基于 BTC 存款证明铸造代币
- 🔥 **Burn 操作**: 燃烧代币并提取到 BTC 地址
- 👂 **事件监听**: 监听 mint 和 burn 事件
- 📊 **配置查询**: 查询桥接配置和状态
- 🛠️ **工具函数**: 地址验证、数量格式化等

## 安装

### 1. 添加依赖

在你的 `Cargo.toml` 文件中添加以下依赖：

```toml
[dependencies]
aptos-bridge-sdk = { path = "path/to/aptos-bridge-sdk" }
aptos-sdk = { git = "https://github.com/aptos-labs/aptos-core", branch = "devnet" }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"

[patch.crates-io]
merlin = { git = "https://github.com/aptos-labs/merlin" }
x25519-dalek = { git = "https://github.com/aptos-labs/x25519-dalek", branch = "zeroize_v1" }
```

### 2. 创建 `.cargo/config.toml`

在项目根目录创建 `.cargo/config.toml` 文件：

```toml
[build]
rustflags = ["--cfg", "tokio_unstable"]
```

## 快速开始

### 基本使用

```rust
use aptos_bridge_sdk::{BridgeClient, QueryClient};
use aptos_sdk::types::chain_id::ChainId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建桥接客户端
    let mut bridge_client = BridgeClient::new(
        "https://fullnode.devnet.aptoslabs.com/v1",
        "your_private_key",
        "bridge_contract_address",
        ChainId::new(2), // devnet
    ).await?;
    
    // 创建查询客户端
    let query_client = QueryClient::new(
        "https://fullnode.devnet.aptoslabs.com/v1",
        "bridge_contract_address"
    )?;
    
    // 查询桥接配置
    let config = query_client.get_bridge_config().await?;
    println!("Bridge config: {:?}", config);
    
    Ok(())
}
```

### Mint 操作

```rust
use aptos_bridge_sdk::types::{Peg, TxProof, ScriptType};

// 创建 Peg 数据
let peg = Peg {
    to: "0x1234...".to_string(),
    value: 100_000_000, // 1 BTC in satoshi
    block_num: 800_000,
    inclusion_proof: TxProof {
        block_header: vec![/* block header data */],
        tx_id: vec![/* transaction id */],
        tx_index: 0,
        merkle_proof: vec![/* merkle proof */],
        raw_tx: vec![/* raw transaction */],
    },
    tx_out_ix: 0,
    dest_script_hash: vec![/* script hash */],
    script_type: ScriptType::P2PKH,
};

// 执行 mint
let tx_hash = bridge_client.mint(vec![peg]).await?;
println!("Mint transaction: {}", tx_hash);
```

### Burn 操作

```rust
// 执行 burn
let tx_hash = bridge_client.burn(
    "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(), // BTC 地址
    100,          // 费率
    50_000_000,   // 0.5 BTC in satoshi
    1,            // 操作员 ID
).await?;
println!("Burn transaction: {}", tx_hash);
```

### 事件监听

```rust
use aptos_bridge_sdk::{EventHandler, EventMonitor};
use async_trait::async_trait;

struct MyEventHandler;

#[async_trait]
impl EventHandler for MyEventHandler {
    async fn handle_mint(&self, event: MintEvent) -> Result<(), BridgeError> {
        println!("Mint event: {:?}", event);
        Ok(())
    }
    
    async fn handle_burn(&self, event: BurnEvent) -> Result<(), BridgeError> {
        println!("Burn event: {:?}", event);
        Ok(())
    }
}

// 创建事件监听器
let mut event_monitor = EventMonitor::new(
    "https://fullnode.devnet.aptoslabs.com/v1",
    "bridge_contract_address",
    Box::new(MyEventHandler),
    0, // 起始版本
)?;

// 启动监听
event_monitor.start().await?;
```

## 示例

项目包含以下示例：

### 1. Mint 示例

```bash
export PRIVATE_KEY=0x1234567890abcdef...
export BRIDGE_CONTRACT_ADDRESS=0xabc123...
cargo run --example mint
```

### 2. Burn 示例

```bash
export PRIVATE_KEY=0x1234567890abcdef...
export BRIDGE_CONTRACT_ADDRESS=0xabc123...
export BTC_ADDRESS=bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
export AMOUNT=50000000
cargo run --example burn
```

### 3. 事件监听示例

```bash
export BRIDGE_CONTRACT_ADDRESS=0xabc123...
export START_VERSION=1000000
cargo run --example event_listener
```

### 4. 查询示例

```bash
export BRIDGE_CONTRACT_ADDRESS=0xabc123...
cargo run --example query
```

## API 文档

### BridgeClient

主要的桥接客户端，提供 mint 和 burn 功能。

#### 方法

- `new(node_url, private_key, bridge_contract, chain_id)` - 创建新的客户端
- `mint(pegs)` - 铸造代币
- `burn(btc_address, fee_rate, amount, operator_id)` - 燃烧代币
- `get_transaction_status(tx_hash)` - 查询交易状态
- `wait_for_transaction(tx_hash, timeout)` - 等待交易确认

### QueryClient

查询客户端，提供配置和状态查询功能。

#### 方法

- `new(node_url, bridge_contract)` - 创建新的查询客户端
- `get_bridge_config()` - 获取完整的桥接配置
- `get_owner()` - 获取合约所有者
- `get_min_confirmations()` - 获取最小确认数
- `is_burn_paused()` - 检查燃烧是否暂停
- `validate_mint_params(amount, peg_count)` - 验证 mint 参数
- `validate_burn_params(amount, fee_rate)` - 验证 burn 参数

### EventMonitor

事件监听器，用于监听桥接事件。

#### 方法

- `new(node_url, contract_address, handler, start_version)` - 创建事件监听器
- `start()` - 启动监听器
- `process_events()` - 处理事件（一次性）
- `set_batch_size(size)` - 设置批处理大小
- `set_poll_interval(interval)` - 设置轮询间隔

## 配置

### 环境变量

- `APTOS_NODE_URL` - Aptos 节点 URL（默认：devnet）
- `PRIVATE_KEY` - 私钥（必需）
- `BRIDGE_CONTRACT_ADDRESS` - 桥接合约地址（必需）
- `FAUCET_URL` - 水龙头 URL（测试网用）

### 网络配置

#### Devnet

```rust
let chain_id = ChainId::new(2);
let node_url = "https://fullnode.devnet.aptoslabs.com/v1";
let faucet_url = "https://faucet.devnet.aptoslabs.com";
```

#### Testnet

```rust
let chain_id = ChainId::new(2);
let node_url = "https://fullnode.testnet.aptoslabs.com/v1";
let faucet_url = "https://faucet.testnet.aptoslabs.com";
```

#### Mainnet

```rust
let chain_id = ChainId::new(1);
let node_url = "https://fullnode.mainnet.aptoslabs.com/v1";
// 主网没有水龙头
```

## 数据类型

### Peg

```rust
pub struct Peg {
    pub to: String,                    // 接收地址
    pub value: u64,                    // BTC 数量（satoshi）
    pub block_num: u64,                // 区块高度
    pub inclusion_proof: TxProof,      // 包含证明
    pub tx_out_ix: u64,                // 输出索引
    pub dest_script_hash: Vec<u8>,     // 目标脚本哈希
    pub script_type: ScriptType,       // 脚本类型
}
```

### TxProof

```rust
pub struct TxProof {
    pub block_header: Vec<u8>,         // 区块头
    pub tx_id: Vec<u8>,                // 交易 ID
    pub tx_index: u64,                 // 交易索引
    pub merkle_proof: Vec<Vec<u8>>,    // Merkle 证明
    pub raw_tx: Vec<u8>,               // 原始交易
}
```

### BridgeConfig

```rust
pub struct BridgeConfig {
    pub owner: String,                 // 合约所有者
    pub min_confirmations: u64,        // 最小确认数
    pub max_pegs_per_mint: u64,        // 每次 mint 的最大 peg 数
    pub max_btc_per_mint: u64,         // 每次 mint 的最大 BTC 数量
    pub min_btc_per_mint: u64,         // 每次 mint 的最小 BTC 数量
    pub max_btc_per_burn: u64,         // 每次 burn 的最大 BTC 数量
    pub min_btc_per_burn: u64,         // 每次 burn 的最小 BTC 数量
    pub burn_paused: bool,             // burn 是否暂停
    pub max_fee_rate: u64,             // 最大费率
}
```

## 错误处理

SDK 使用自定义的错误类型 `BridgeError`：

```rust
pub enum BridgeError {
    Network(reqwest::Error),           // 网络错误
    Json(serde_json::Error),           // JSON 解析错误
    Bcs(bcs::Error),                   // BCS 序列化错误
    Aptos(String),                     // Aptos SDK 错误
    InvalidPrivateKey,                 // 无效私钥
    InvalidAddress(String),            // 无效地址
    TransactionFailed(String),         // 交易失败
    EventParseFailed(String),          // 事件解析失败
    Config(String),                    // 配置错误
    Other(String),                     // 其他错误
}
```

## 工具函数

### 地址和密钥验证

```rust
use aptos_bridge_sdk::utils::*;

// 验证私钥
validate_private_key("0x1234...")?;

// 验证 BTC 地址
validate_btc_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4")?;

// 解析账户地址
let addr = parse_account_address("0x1234...")?;
```

### 数量格式化

```rust
// 格式化 BTC 数量
let formatted = format_btc_amount(100_000_000); // "1.00000000 BTC"

// 解析 BTC 数量
let satoshi = parse_btc_amount("1.5 BTC")?; // 150_000_000
```

## 测试

运行测试：

```bash
cargo test
```

运行特定测试：

```bash
cargo test --test integration_tests
```

## 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

## 支持

如果您遇到问题或有疑问，请：

1. 查看 [示例代码](examples/)
2. 阅读 [API 文档](https://docs.rs/aptos-bridge-sdk)
3. 在 [GitHub Issues](https://github.com/your-org/aptos-bridge-sdk/issues) 中提出问题

## 更新日志

查看 [CHANGELOG.md](CHANGELOG.md) 了解版本更新信息。
