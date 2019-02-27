use exonum::{blockchain::TransactionContext};

use rlua::{Function, Lua, MultiValue, StdLib};

use std::collections::HashMap;

use crate::{currency::wallet::Wallet, lvm::contract::Contract};

use super::{context_wrap::RunnerCtxWrap};

pub type State = HashMap<String, String>;

#[derive(Debug)]
pub struct Runner<'a, 'ctx> {
    pub contract: Contract,
    pub contract_wallet: Wallet,
    pub context: &'a mut TransactionContext<'ctx>,
}

impl Runner<'_, '_> {
    pub fn exec(mut self, fn_name: &str, args: Vec<String>) -> Result<Contract, String> {
        let lvm_lua_subset = StdLib::BASE
            | StdLib::TABLE
            | StdLib::STRING
            | StdLib::UTF8
            | StdLib::MATH
            | StdLib::PACKAGE;
        let lua = Lua::new_with(lvm_lua_subset);

        RunnerCtxWrap::init(&mut self);

        let result: rlua::Result<_> = lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();

            let state_table = lua_ctx.pack(self.contract.state.clone())?;
            globals.raw_set("state", state_table)?;

            RunnerCtxWrap::register_functions(&lua_ctx)?;

            lua_ctx.load(&self.contract.code).exec()?;

            let func: Function = globals.get(fn_name)?;
            let args: rlua::Result<Vec<_>> = args.into_iter().map(|v| lua_ctx.pack(v)).collect();
            let args = MultiValue::from_vec(args?);
            func.call::<_, ()>(args)?;

            let state_table = globals.raw_get("state")?;
            self.contract.state = lua_ctx.unpack(state_table)?;
            Ok(())
        });

        RunnerCtxWrap::reset();

        match result {
            Ok(()) => Ok(self.contract.clone()),
            Err(e) => Err(format!("{}", e)),
        }
    }
}
