use rlua::{Lua, Function, MultiValue};
use std::collections::HashMap;

use super::{
    contract::Contract,
};

pub type State = HashMap<String, String>;

#[derive(Debug)]
pub struct Runner;

impl Runner {
    pub fn exec(mut contract: Contract, fn_name: &str, args: Vec<String>) -> Result<Contract, String> {
        let lua = Lua::new();
        let result: rlua::Result<_> = lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();

            lua_ctx.load(&contract.code).exec()?;

            let state_table = lua_ctx.pack(contract.state.clone())?;
            globals.set("state", state_table)?;

            let func: Function = globals.get(fn_name)?;
            let args: rlua::Result<Vec<_>> = args.into_iter().map(|v| lua_ctx.pack(v)).collect();
            let args = MultiValue::from_vec(args?);
            func.call::<_, ()>(args)?;

            let state_table = globals.raw_get("state")?;
            contract.state = lua_ctx.unpack(state_table)?;
            Ok(contract)
        });

        match result {
            Ok(c) => Ok(c),
            Err(e) => Err(format!("{}", e)),
        }
    }
}

#[test]
fn update_state() {
    use exonum::crypto::PublicKey;

    let contract = Contract::new(&PublicKey::zero(), r#"
        function set_state_at(key, val)
            state[key] = val
        end
    "#);

    let r = Runner::exec(contract, "set_state_at", vec!["key".to_string(), "value".to_string()]);
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap().state,
        [("key".to_string(), "value".to_string())].iter().cloned().collect()
    );
}
