#[macro_use]
extern crate serde_json;

use common::{
    testkit::create_testkit,
};

mod common;

#[test]
fn create_contract() {
    let (mut testkit, api) = create_testkit();

    let code = "some code";
    let (tx, contract_pub) = api.create_contract(code);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let contract = api.get_contract(contract_pub);
    assert!(contract.is_some());
    let contract = contract.unwrap();
    assert_eq!(contract.pub_key, contract_pub);
    assert_eq!(contract.code, code);
}


#[test]
fn call_contract() {
    let (mut testkit, api) = create_testkit();

    let code = r#"
        function nothing()
        end
    "#;
    let (tx, contract_pub) = api.create_contract(code);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let contract = api.get_contract(contract_pub);
    assert!(contract.is_some());

    let tx = api.call_contract(&contract_pub, "nothing", vec![]);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));
}

#[test]
fn contract_changes_state() {
    let (mut testkit, api) = create_testkit();

    let code = r#"
        function greet(what)
            state["hello"] = what
        end
    "#;
    let (tx, contract_pub) = api.create_contract(code);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let contract_before = api.get_contract(contract_pub);
    assert!(contract_before.is_some());
    let contract_before = contract_before.unwrap();
    assert!(contract_before.state.get("hello").is_none());

    let tx = api.call_contract(&contract_pub, "greet", vec!["lvm"]);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let contract_after = api.get_contract(contract_pub).unwrap();
    assert_eq!(contract_after.state.get("hello"), Some(&"lvm".to_string()));
}
