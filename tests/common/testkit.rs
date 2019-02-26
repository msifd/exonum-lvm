use exonum::{
    api::node::public::explorer::{TransactionQuery, TransactionResponse},
    crypto::{self, Hash, PublicKey, SecretKey},
    messages::{self, RawTransaction, Signed},
};

use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};

// Import data types used in tests from the crate where the service is defined.
use exonum_lvm::{
    currency::{
        service as currency_service,
        api::{WalletInfo, WalletQuery},
        transactions::CreateWallet,
        wallet::Wallet,
    },
    lvm::{
        service as lvm_service,
        api::{ContractInfo, ContractQuery},
        contract::Contract,
        transactions::{CreateContract, CallContract},
    },
};

/// Wrapper for the cryptocurrency service API allowing to easily use it
/// (compared to `TestKitApi` calls).
pub struct CryptocurrencyApi {
    pub inner: TestKitApi,
}

impl CryptocurrencyApi {
    /// Generates a wallet creation transaction with a random key pair, sends it over HTTP,
    /// and checks the synchronous result (i.e., the hash of the transaction returned
    /// within the response).
    /// Note that the transaction is not immediately added to the blockchain, but rather is put
    /// to the pool of unconfirmed transactions.
    pub fn create_wallet(&self, name: &str) -> (Signed<RawTransaction>, SecretKey) {
        let (pubkey, key) = crypto::gen_keypair();
        // Create a pre-signed transaction
        let tx = CreateWallet::sign(name, &pubkey, &key);

        let data = messages::to_hex_string(&tx);
        let tx_info: TransactionResponse = self
            .inner
            .public(ApiKind::Explorer)
            .query(&json!({ "tx_body": data }))
            .post("v1/transactions")
            .unwrap();
        assert_eq!(tx_info.tx_hash, tx.hash());
        (tx, key)
    }

    pub fn get_wallet(&self, pub_key: PublicKey) -> Option<Wallet> {
        let wallet_info = self
            .inner
            .public(ApiKind::Service(currency_service::SERVICE_NAME))
            .query(&WalletQuery { pub_key })
            .get::<WalletInfo>("v1/wallets/info")
            .unwrap();

        let to_wallet = wallet_info.wallet_proof.to_wallet.check().unwrap();
        let wallet = to_wallet
            .all_entries()
            .find(|(ref k, _)| **k == pub_key)
            .and_then(|tuple| tuple.1)
            .cloned();
        wallet
    }

    /// Sends a transfer transaction over HTTP and checks the synchronous result.
    pub fn transfer(&self, tx: &Signed<RawTransaction>) {
        let data = messages::to_hex_string(&tx);
        let tx_info: TransactionResponse = self
            .inner
            .public(ApiKind::Explorer)
            .query(&json!({ "tx_body": data }))
            .post("v1/transactions")
            .unwrap();
        assert_eq!(tx_info.tx_hash, tx.hash());
    }

    /// Asserts that a wallet with the specified public key is not known to the blockchain.
    pub fn assert_no_wallet(&self, pub_key: PublicKey) {
        let wallet_info: WalletInfo = self
            .inner
            .public(ApiKind::Service(currency_service::SERVICE_NAME))
            .query(&WalletQuery { pub_key })
            .get("v1/wallets/info")
            .unwrap();

        let to_wallet = wallet_info.wallet_proof.to_wallet.check().unwrap();
        assert!(to_wallet.missing_keys().find(|v| **v == pub_key).is_some())
    }

    /// Asserts that the transaction with the given hash has a specified status.
    pub fn assert_tx_status(&self, tx_hash: Hash, expected_status: &serde_json::Value) {
        let info: serde_json::Value = self
            .inner
            .public(ApiKind::Explorer)
            .query(&TransactionQuery::new(tx_hash))
            .get("v1/transactions")
            .unwrap();

        if let serde_json::Value::Object(mut info) = info {
            let tx_status = info.remove("status").unwrap();
            assert_eq!(tx_status, *expected_status);
        } else {
            panic!("Invalid transaction info format, object expected");
        }
    }

    pub fn create_contract(&self, code: &str) -> (Signed<RawTransaction>, PublicKey) {
        let (pubkey, key) = crypto::gen_keypair();
        let (contract_pk, _) = crypto::gen_keypair();
        // Create a pre-signed transaction
        let tx = CreateContract::sign(&contract_pk, code, &pubkey, &key);

        let data = messages::to_hex_string(&tx);
        let tx_info: TransactionResponse = self
            .inner
            .public(ApiKind::Explorer)
            .query(&json!({ "tx_body": data }))
            .post("v1/transactions")
            .unwrap();
        assert_eq!(tx_info.tx_hash, tx.hash());
        (tx, contract_pk)
    }

    pub fn get_contract(&self, pub_key: PublicKey) -> Option<Contract> {
        let contract_info = self
            .inner
            .public(ApiKind::Service(lvm_service::SERVICE_NAME))
            .query(&ContractQuery { pub_key })
            .get::<ContractInfo>("v1/contracts/info")
            .unwrap();

        let contract_proof = contract_info.contract_proof.check().unwrap();
        let contract = contract_proof
            .all_entries()
            .find(|(ref k, _)| **k == pub_key)
            .and_then(|tuple| tuple.1)
            .cloned();
        contract
    }

    pub fn call_contract(&self, contract_pk: &PublicKey, fn_name: &str, args: Vec<&str>) -> Signed<RawTransaction> {
        let (pubkey, key) = crypto::gen_keypair();

        let args = args.iter().map(|s| s.to_string()).collect();
        let tx = CallContract::sign(&contract_pk, fn_name, &args, &pubkey, &key);

        let data = messages::to_hex_string(&tx);
        let tx_info: TransactionResponse = self
            .inner
            .public(ApiKind::Explorer)
            .query(&json!({ "tx_body": data }))
            .post("v1/transactions")
            .unwrap();
        assert_eq!(tx_info.tx_hash, tx.hash());
        tx
    }
}

/// Creates a testkit together with the API wrapper defined above.
pub fn create_testkit() -> (TestKit, CryptocurrencyApi) {
    let testkit = TestKitBuilder::validator()
        .with_service(currency_service::Service)
        .with_service(lvm_service::Service)
        .create();
    let api = CryptocurrencyApi {
        inner: testkit.api(),
    };
    (testkit, api)
}
