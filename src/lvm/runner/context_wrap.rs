use exonum::{blockchain::TransactionContext, crypto::PublicKey};

use rlua::Context;

use std::sync::Mutex;

use crate::{
    currency::{schema::Schema as CurrencySchema, wallet::Wallet},
    lvm::contract::Contract,
};

use super::{lua_api::CurrencyApi, runner::Runner};

static mut WRAP: Option<Mutex<RunnerCtxWrap>> = None;

pub struct RunnerCtxWrap {
    _contract: *mut Contract,
    contract_wallet: *mut Wallet,
    context: *mut TransactionContext<'static>,
}

unsafe impl Send for RunnerCtxWrap {}

impl RunnerCtxWrap {
    pub fn init(runner: &mut Runner) {
        unsafe fn extend_lifetime<'a, 'ctx>(
            r: &mut TransactionContext<'ctx>,
        ) -> &'a mut TransactionContext<'static> {
            std::mem::transmute::<_, _>(r)
        }

        unsafe {
            WRAP = Some(Mutex::new(RunnerCtxWrap {
                _contract: &mut runner.contract,
                contract_wallet: &mut runner.contract_wallet,
                context: extend_lifetime(runner.context),
            }));
        };
    }

    pub fn reset() {
        unsafe { WRAP = None };
    }

    pub fn register_functions(lua_ctx: &Context) -> rlua::Result<()> {
        let globals = lua_ctx.globals();

        let transfer_fn = lua_ctx.create_function(|_, (to, amount): (String, u64)| {
            RunnerCtxWrap::transfer(&to, amount);
            Ok(())
        })?;
        globals.raw_set("transfer", transfer_fn)?;

        Ok(())
    }
}

impl CurrencyApi for RunnerCtxWrap {
    fn transfer(receiver: &String, amount: u64) {
        if let Some(wrap) = unsafe { &WRAP } {
            let mut wrap = wrap.lock().unwrap();

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
