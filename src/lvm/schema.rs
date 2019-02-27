use exonum::{
    crypto::{Hash, PublicKey},
    storage::{Fork, ProofMapIndex, Snapshot},
};

use super::contract::Contract;

#[derive(Debug)]
pub struct Schema<T> {
    view: T,
}

impl<T> AsMut<T> for Schema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.view
    }
}

impl<T> Schema<T>
where
    T: AsRef<dyn Snapshot>,
{
    pub fn new(view: T) -> Self {
        Schema { view }
    }

    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.contracts().merkle_root()]
    }

    pub fn contracts(&self) -> ProofMapIndex<&T, PublicKey, Contract> {
        ProofMapIndex::new("lvm.contracts", &self.view)
    }

    pub fn contract(&self, pub_key: &PublicKey) -> Option<Contract> {
        self.contracts().get(pub_key)
    }
}

impl Schema<&mut Fork> {
    pub fn contracts_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Contract> {
        ProofMapIndex::new("lvm.contracts", &mut self.view)
    }

    pub fn create_contract(&mut self, pub_key: &PublicKey, code: &str) {
        let contract = Contract::new(pub_key, code);
        self.contracts_mut().put(pub_key, contract);
    }
}
