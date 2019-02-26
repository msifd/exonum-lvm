use exonum::{
    api::ServiceApiBuilder,
    blockchain::{self, Transaction, TransactionSet},
    crypto::Hash,
    helpers::fabric::{self, Context},
    messages::RawTransaction,
    storage::Snapshot,
};

use super::{
    Schema,
    api::PublicApi,
    transactions::WalletTransactions,
};

/// Unique service ID.
pub const CRYPTOCURRENCY_SERVICE_ID: u16 = 128;
/// Name of the service.
pub const SERVICE_NAME: &str = "cryptocurrency";
/// Initial balance of the wallet.
pub const INITIAL_BALANCE: u64 = 100;

/// Exonum `Service` implementation.
#[derive(Default, Debug)]
pub struct Service;

impl blockchain::Service for Service {
    fn service_id(&self) -> u16 {
        CRYPTOCURRENCY_SERVICE_ID
    }

    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn state_hash(&self, view: &dyn Snapshot) -> Vec<Hash> {
        let schema = Schema::new(view);
        schema.state_hash()
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<dyn Transaction>, failure::Error> {
        WalletTransactions::tx_from_raw(raw).map(Into::into)
    }

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        PublicApi::wire(builder);
    }
}

/// A configuration service creator for the `NodeBuilder`.
#[derive(Debug)]
pub struct ServiceFactory;

impl fabric::ServiceFactory for ServiceFactory {
    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    fn make_service(&mut self, _: &Context) -> Box<dyn blockchain::Service> {
        Box::new(Service)
    }
}
