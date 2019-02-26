use exonum::{
    crypto::PublicKey,
};

use std::collections::HashMap;

use super::proto;

#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::Contract", serde_pb_convert)]
pub struct Contract {
    pub pub_key: PublicKey,
    pub code: String,
    pub state: HashMap<String, String>,
}

impl Contract {
    pub fn new(pub_key: &PublicKey, code: &str) -> Self {
        Self {
            pub_key: *pub_key,
            code: code.to_string(),
            state: HashMap::new(),
        }
    }
}
