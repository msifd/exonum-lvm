#[macro_use]
extern crate serde_json;

use common::{
    ALICE_NAME,
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

#[test]
fn contract_persist_state() {
    let (mut testkit, api) = create_testkit();

    let code = r#"
        function reset()
            state["counter"] = 0
        end

        function increase()
            state["counter"] = state["counter"] + 1
        end
    "#;
    let (tx, contract_pub) = api.create_contract(code);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let tx = api.call_contract(&contract_pub, "reset", vec![]);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    for _ in 0..2 {
        let tx = api.call_contract(&contract_pub, "increase", vec![]);
        testkit.create_block();
        api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));
    }

    let contract = api.get_contract(contract_pub).unwrap();
    assert_eq!(contract.state.get("counter"), Some(&"2.0".to_string()));
}

#[test]
fn contract_num_types() {
    let (mut testkit, api) = create_testkit();

    let code = r#"
        function set(x)
            state["value"] = x * 2
        end
    "#;
    let (tx, contract_pub) = api.create_contract(code);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let tx = api.call_contract(&contract_pub, "set", vec!["5"]);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let contract = api.get_contract(contract_pub).unwrap();
    assert_eq!(contract.state.get("value"), Some(&"10.0".to_string()));
}

#[test]
fn contract_do_transfer() {
    let (mut testkit, api) = create_testkit();
    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME);

    let code = r#"
        function transfer_half(to, amount)
            transfer(to, amount / 2)
        end
    "#;
    let (tx, contract_pub) = api.create_contract(code);
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let wallet = api.get_wallet(tx_alice.author()).unwrap();
    assert_eq!(wallet.balance, 100);
    let wallet = api.get_wallet(contract_pub).unwrap();
    assert_eq!(wallet.balance, 100);

    let tx = api.call_contract(&contract_pub, "transfer_half", vec![&tx_alice.author().to_hex(), "10"]);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let wallet = api.get_wallet(tx_alice.author()).unwrap();
    assert_eq!(wallet.balance, 105);
    let wallet = api.get_wallet(contract_pub).unwrap();
    assert_eq!(wallet.balance, 95);
}
