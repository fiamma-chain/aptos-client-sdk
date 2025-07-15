//! Bridge client implementation
//!
//! Provides core functionality for interacting with Aptos Bridge contracts.

use crate::types::{constants::*, BridgeError, BridgeResult, Peg, PegForBcs};
use crate::utils::{parse_account_address, validate_btc_address};
use crate::QueryClient;

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
    ) -> BridgeResult<Self> {
        // Parse contract address
        let bridge_contract_address = parse_account_address(bridge_contract_address)?;

        let btc_light_client = parse_account_address(btc_light_client)?;

        // Create REST client
        let rest_client = Client::new(
            Url::parse(node_url)
                .map_err(|e| BridgeError::Config(format!("Invalid Aptos node URL: {}", e)))?,
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
            query_client,
            account,
            bridge_contract_address,
            btc_light_client,
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
    pub async fn get_min_confirmations(&self) -> BridgeResult<u64> {
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
            .map_err(|e| BridgeError::Aptos(e.to_string()))?;

        // Parse the response
        let result = response
            .inner()
            .get(0)
            .ok_or_else(|| BridgeError::Other("No response from view function".to_string()))?;

        // Deserialize the u64 result
        let min_confirmations: u64 = serde_json::from_value(result.clone())
            .map_err(|e| BridgeError::Other(format!("Failed to parse min_confirmations: {}", e)))?;

        Ok(min_confirmations)
    }

    /// Get latest block height
    pub async fn get_latest_block_height(&self) -> BridgeResult<u64> {
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
            .map_err(|e| BridgeError::Aptos(e.to_string()))?;

        // Parse the response
        let result = response
            .inner()
            .get(0)
            .ok_or_else(|| BridgeError::Other("No response from view function".to_string()))?;

        // Deserialize the u64 result
        let latest_block_height: u64 = serde_json::from_value(result.clone()).map_err(|e| {
            BridgeError::Other(format!("Failed to parse latest_block_height: {}", e))
        })?;

        Ok(latest_block_height)
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

impl std::ops::Deref for BridgeClient {
    type Target = QueryClient;

    fn deref(&self) -> &Self::Target {
        &self.query_client
    }
}
