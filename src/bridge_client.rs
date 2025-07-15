//! Bridge client implementation
//!
//! Provides core functionality for interacting with Aptos Bridge contracts.

use crate::types::{constants::*, Peg, PegForBcs};
use crate::utils::{parse_account_address, validate_btc_address};
use crate::QueryClient;

use anyhow::{bail, Context, Result};
use aptos_sdk::move_types::identifier::Identifier;
use aptos_sdk::move_types::language_storage::ModuleId;
use aptos_sdk::rest_client::aptos_api_types::{EntryFunctionId, IdentifierWrapper, MoveModuleId};
use aptos_sdk::transaction_builder::TransactionBuilder;
use aptos_sdk::{
    crypto::ed25519::Ed25519PrivateKey,
    rest_client::{aptos_api_types::ViewRequest, Client},
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        transaction::{EntryFunction, TransactionPayload},
        LocalAccount,
    },
};
use url::Url;

/// Bridge client
pub struct BridgeClient {
    /// REST client
    rest_client: Client,
    /// Query client
    query_client: QueryClient,
    /// Local account
    account: LocalAccount,
    /// Bridge contract address
    bridge_contract_address: AccountAddress,
    /// BTC Light client
    btc_light_client: AccountAddress,
}

impl BridgeClient {
    /// Create new Bridge client
    pub async fn new(
        node_url: &str,
        private_key_hex: &str,
        bridge_contract_address: &str,
        btc_light_client: &str,
    ) -> Result<Self> {
        // Parse contract address
        let bridge_contract_address = parse_account_address(bridge_contract_address)?;

        let btc_light_client = parse_account_address(btc_light_client)?;

        // Create REST client
        let rest_client = Client::new(
            Url::parse(node_url)
                .with_context(|| format!("Invalid Aptos node URL: {}", node_url))?,
        );

        // Create query client
        let query_client = QueryClient::new(node_url)?;

        // Create private key
        let private_key_str = if private_key_hex.starts_with("0x") {
            &private_key_hex[2..]
        } else {
            private_key_hex
        };

        let private_key = Ed25519PrivateKey::try_from(private_key_str.as_bytes())
            .context("Invalid private key format")?;

        // Create local account
        let account = LocalAccount::new(
            AccountAddress::from_str_strict(private_key_str)
                .context("Invalid private key format")?,
            private_key,
            0,
        );

        Ok(Self {
            rest_client,
            query_client,
            account,
            bridge_contract_address,
            btc_light_client,
        })
    }

    /// Mint tokens
    pub async fn mint(&mut self, pegs: Vec<Peg>) -> Result<String> {
        if pegs.is_empty() {
            bail!("Pegs cannot be empty");
        }

        // Convert to BCS-serializable format
        let pegs_for_bcs: Vec<PegForBcs> = pegs
            .iter()
            .map(|peg| PegForBcs::try_from(peg))
            .collect::<Result<Vec<_>, _>>()?;

        // Serialize parameters
        let args = vec![bcs::to_bytes(&pegs_for_bcs).context("Failed to serialize pegs for BCS")?];

        // Create Entry Function
        let entry_function = EntryFunction::new(
            ModuleId::new(
                self.bridge_contract_address,
                Identifier::new("fiamma_bridge_account").unwrap(),
            ),
            Identifier::new("mint").unwrap(),
            vec![], // No type parameters
            args,
        );

        // Execute transaction
        let tx_hash = self
            .execute_transaction(TransactionPayload::EntryFunction(entry_function))
            .await?;

        Ok(tx_hash)
    }

    /// Burn tokens
    pub async fn burn(
        &mut self,
        btc_address: String,
        fee_rate: u64,
        amount: u64,
        operator_id: u64,
    ) -> Result<String> {
        // Validate BTC address format
        validate_btc_address(&btc_address)?;

        // Validate amount
        if amount == 0 {
            bail!("Amount cannot be zero");
        }

        // Serialize parameters
        let args = vec![
            bcs::to_bytes(&btc_address).context("Failed to serialize BTC address")?,
            bcs::to_bytes(&fee_rate).context("Failed to serialize fee rate")?,
            bcs::to_bytes(&amount).context("Failed to serialize amount")?,
            bcs::to_bytes(&operator_id).context("Failed to serialize operator ID")?,
        ];

        // Create Entry Function
        let entry_function = EntryFunction::new(
            ModuleId::new(
                self.bridge_contract_address,
                Identifier::new("fiamma_bridge_account").unwrap(),
            ),
            Identifier::new("burn").unwrap(),
            vec![], // No type parameters
            args,
        );

        // Execute transaction
        let tx_hash = self
            .execute_transaction(TransactionPayload::EntryFunction(entry_function))
            .await?;

        Ok(tx_hash)
    }

    /// Get minimum confirmations required for BTC transactions
    pub async fn get_min_confirmations(&self) -> Result<u64> {
        // Construct the view function call
        let view_request = ViewRequest {
            function: EntryFunctionId {
                module: MoveModuleId {
                    address: self.bridge_contract_address.into(),
                    name: IdentifierWrapper(Identifier::new("bridge").unwrap()),
                },
                name: IdentifierWrapper(Identifier::new("get_min_confirmations").unwrap()),
            },
            type_arguments: vec![],
            arguments: vec![],
        };

        // Call the view function
        let response = self
            .rest_client
            .view(&view_request, None)
            .await
            .context("Failed to call get_min_confirmations view function")?;

        // Parse the response
        let result = response
            .inner()
            .get(0)
            .context("No response from view function")?;

        // Deserialize the u64 result
        let min_confirmations: u64 =
            serde_json::from_value(result.clone()).context("Failed to parse min_confirmations")?;

        Ok(min_confirmations)
    }

    /// Get latest block height from BTC light client
    pub async fn get_latest_block_height(&self) -> Result<u64> {
        // Construct the view function call
        let view_request = ViewRequest {
            function: EntryFunctionId {
                module: MoveModuleId {
                    address: self.btc_light_client.into(),
                    name: IdentifierWrapper(Identifier::new("btc_light_client").unwrap()),
                },
                name: IdentifierWrapper(Identifier::new("get_latest_block_height").unwrap()),
            },
            type_arguments: vec![],
            arguments: vec![],
        };

        // Call the view function
        let response = self
            .rest_client
            .view(&view_request, None)
            .await
            .context("Failed to call get_latest_block_height view function")?;

        // Parse the response
        let result = response
            .inner()
            .get(0)
            .context("No response from view function")?;

        // Deserialize the u64 result
        let latest_block_height: u64 = serde_json::from_value(result.clone())
            .context("Failed to parse latest_block_height")?;

        Ok(latest_block_height)
    }

    /// Generic method for executing transactions
    async fn execute_transaction(&mut self, payload: TransactionPayload) -> Result<String> {
        let chain_id = self
            .rest_client
            .get_index()
            .await
            .context("Failed to get chain ID from Aptos node")?
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
            .context("Failed to submit transaction to Aptos node")?;

        Ok(response.inner().hash.to_string())
    }
}

impl std::ops::Deref for BridgeClient {
    type Target = QueryClient;

    fn deref(&self) -> &Self::Target {
        &self.query_client
    }
}
