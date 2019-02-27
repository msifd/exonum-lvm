use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};

use crate::currency::schema::Schema as CurrencySchema;

use crate::lvm::{proto, runner::Runner, schema::Schema as LvmSchema, service::LVM_SERVICE_ID};

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    #[fail(display = "Contract already exists")]
    ContractAlreadyExists = 0,
    #[fail(display = "Contract not exists")]
    ContractNotExists = 1,
    #[fail(display = "Contract execution error")]
    ContractExecutionError = 2,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::CreateContract")]
pub struct CreateContract {
    pub pub_key: PublicKey,
    pub code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::CallContract")]
pub struct CallContract {
    pub pub_key: PublicKey,
    pub fn_name: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum LvmTransactions {
    CreateContract(CreateContract),
    CallContract(CallContract),
}

impl CreateContract {
    #[doc(hidden)]
    pub fn sign(
        pub_key: &PublicKey,
        code: &str,
        pk: &PublicKey,
        sk: &SecretKey,
    ) -> Signed<RawTransaction> {
        Message::sign_transaction(
            Self {
                pub_key: *pub_key,
                code: code.to_string(),
            },
            LVM_SERVICE_ID,
            *pk,
            sk,
        )
    }
}

impl CallContract {
    #[doc(hidden)]
    pub fn sign(
        pub_key: &PublicKey,
        fn_name: &str,
        args: &Vec<String>,
        pk: &PublicKey,
        sk: &SecretKey,
    ) -> Signed<RawTransaction> {
        Message::sign_transaction(
            Self {
                pub_key: *pub_key,
                fn_name: fn_name.to_string(),
                args: args.clone(),
            },
            LVM_SERVICE_ID,
            *pk,
            sk,
        )
    }
}

impl Transaction for CreateContract {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        {
            let mut schema = LvmSchema::new(context.fork());
            match schema.contract(&self.pub_key) {
                None => {
                    schema.create_contract(&self.pub_key, &self.code);
                }
                Some(_) => Err(Error::ContractAlreadyExists)?,
            }
        }

        {
            let hash = context.tx_hash();
            let mut schema = CurrencySchema::new(context.fork());
            if schema.wallet(&self.pub_key).is_none() {
                let name = format!("contract-{}", &self.pub_key);
                schema.create_wallet(&self.pub_key, &name, &hash);
            } else {
                Err(Error::ContractAlreadyExists)?
            }
        }

        Ok(())
    }
}

impl Transaction for CallContract {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let contract = {
            let schema = LvmSchema::new(context.fork());
            match schema.contract(&self.pub_key) {
                Some(c) => c,
                None => Err(Error::ContractNotExists)?,
            }
        };

        let contract_wallet = {
            let schema = CurrencySchema::new(context.fork());
            match schema.wallet(&self.pub_key) {
                Some(w) => w,
                None => Err(Error::ContractNotExists)?,
            }
        };

        let runner = Runner {
            contract,
            contract_wallet,
            context: &mut context,
        };

        match runner.exec(&self.fn_name, self.args.clone()) {
            Ok(contract) => {
                let mut schema = LvmSchema::new(context.fork());
                schema.contracts_mut().put(&self.pub_key, contract);
                Ok(())
            }
            Err(desc) => Err(ExecutionError::with_description(
                Error::ContractExecutionError as u8,
                desc,
            )),
        }
    }
}
