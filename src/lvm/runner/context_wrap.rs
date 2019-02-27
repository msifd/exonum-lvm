use exonum::{blockchain::TransactionContext, crypto::PublicKey};

use crate::{
    currency::{schema::Schema as CurrencySchema, wallet::Wallet},
    lvm::contract::Contract,
};

use super::{lua_api::CurrencyApi, runner::Runner};

static mut WRAP: Option<RunnerCtxWrap> = None;

pub struct RunnerCtxWrap {
    _contract: *mut Contract,
    contract_wallet: *mut Wallet,
    context: *mut TransactionContext<'static>,
}

unsafe impl Send for RunnerCtxWrap {}
unsafe impl Sync for RunnerCtxWrap {}

impl RunnerCtxWrap {
    pub fn init(runner: &mut Runner) {
        unsafe fn extend_lifetime<'r, 'ctx>(
            r: &'r mut TransactionContext<'ctx>,
        ) -> &'r mut TransactionContext<'static> {
            std::mem::transmute::<_, _>(r)
        }

        unsafe {
            WRAP = Some(RunnerCtxWrap {
                _contract: &mut runner.contract,
                contract_wallet: &mut runner.contract_wallet,
                context: extend_lifetime(runner.context),
            });
        };
    }

    pub fn reset() {
        unsafe { WRAP = None };
    }
}

impl CurrencyApi for RunnerCtxWrap {
    fn transfer(receiver: &String, amount: u64) {
        if let Some(wrap) = unsafe { &WRAP } {
            let sender = unsafe { &*wrap.contract_wallet };
            let context = unsafe { &mut *wrap.context };

            let tx_hash = context.tx_hash();
            let mut schema = CurrencySchema::new(context.fork());

            // TODO: handle errors
            let receiver = hex::decode(&receiver).unwrap();
            let receiver = PublicKey::from_slice(&receiver).unwrap();
            let receiver = schema.wallet(&receiver).unwrap();

            schema.decrease_wallet_balance(sender.clone(), amount, &tx_hash);
            schema.increase_wallet_balance(receiver, amount, &tx_hash);
        }
    }
}
