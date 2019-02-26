use exonum::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    blockchain::{self, BlockProof},
    crypto::PublicKey,
    helpers::Height,
    storage::MapProof,
};

use super::{
    contract::Contract,
    schema::Schema,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ContractQuery {
    pub pub_key: PublicKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractInfo {
    pub block_proof: BlockProof,
    pub contract_proof: MapProof<PublicKey, Contract>,
}

#[derive(Debug, Clone, Copy)]
pub struct PublicApi;

impl PublicApi {
    pub fn contract_info(state: &ServiceApiState, query: ContractQuery) -> api::Result<ContractInfo> {
        let snapshot = state.snapshot();
        let general_schema = blockchain::Schema::new(&snapshot);
        let lvm_schema = Schema::new(&snapshot);

        let max_height = general_schema.block_hashes_by_height().len() - 1;
        let block_proof = general_schema
            .block_and_precommits(Height(max_height))
            .unwrap();

        let contract_proof: MapProof<PublicKey, Contract> = lvm_schema.contracts().get_proof(query.pub_key);

        Ok(ContractInfo {
            block_proof,
            contract_proof,
        })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/contracts/info", Self::contract_info);
    }
}
