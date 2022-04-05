use near_sdk::{serde_json::json, json_types::U128};
use near_sdk_sim::{init_simulator, UserAccount, DEFAULT_GAS, STORAGE_AMOUNT, to_yocto};
use staking_contract_tutorial::AccountJson;
use near_sdk_sim::transaction::{ExecutionStatus};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes!{
    FT_CONTRACT_WASM_FILE => "token-test/vbi-ft.wasm",
    STAKING_CONTRACT_WASM_FILE => "out/staking-contract.wasm"
}

const FT_CONTRACT_ID: &str = "ft_contract";
const FT_TOTAL_SUPPLY: &str = "100000000000000000000000000000"; // 1M token
const STAKING_CONTRACT_ID: &str = "staking_contract";
const STAKING_FT_AMOUNT: &str = "50000000000000000000000000000";
const ALICE_DEPOSIT_AMOUNT: &str = "10000000000000000000000000000";


pub fn init() -> (UserAccount, UserAccount, UserAccount, UserAccount) {
    let root = init_simulator(None);
    let alice = root.create_user("alice".to_string(), to_yocto("100"));

    // Deploy and init FT token
    let ft_contract = root.deploy_and_init(
        &FT_CONTRACT_WASM_FILE,
        FT_CONTRACT_ID.to_string(),
        "new_default_meta", 
        &json!({
            "owner_id": alice.account_id(),
            "total_supply": FT_TOTAL_SUPPLY
        }).to_string().as_bytes(), 
        STORAGE_AMOUNT,
        DEFAULT_GAS
    );

    // Deploy and init Staking contract
    let staking_contract = root.deploy_and_init(
        &STAKING_CONTRACT_WASM_FILE, 
        STAKING_CONTRACT_ID.to_string(), 
        "new_default_config", 
        &json!({
            "owner_id": alice.account_id(),
            "ft_contract_id": ft_contract.account_id()
        }).to_string().as_bytes(), 
        STORAGE_AMOUNT,
        DEFAULT_GAS
    );

    // Storage deposit ft contract
    root.call(
        ft_contract.account_id(), 
        "storage_deposit", 
        &json!({
            "account_id": staking_contract.account_id()
        }).to_string().as_bytes(), 
        DEFAULT_GAS, 
        to_yocto("0.01")
    );

    alice.call(
        ft_contract.account_id(), 
        "ft_transfer", 
        &json!({
            "receiver_id": staking_contract.account_id(),
            "amount": STAKING_FT_AMOUNT
        }).to_string().as_bytes(),
        DEFAULT_GAS, 
        1
    );

    (root, alice, ft_contract, staking_contract)
}

#[test]
pub fn test_deposit_and_stake() {
    let (root, alice, ft_contract, staking_contract) = init();

    // Storage deposit
    alice.call(
        staking_contract.account_id(), 
        "storage_deposit", 
        &json!({}).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    // Deposit token
    alice.call(
        ft_contract.account_id(), 
        "ft_transfer_call", 
        &json!({
            "receiver_id": staking_contract.account_id(),
            "amount": ALICE_DEPOSIT_AMOUNT,
            "msg": ""
        }).to_string().as_bytes(), 
        DEFAULT_GAS, 
        1
    );

    let account_json: AccountJson = root.view(
        staking_contract.account_id(), 
        "get_account_info", 
        &json!({
            "account_id": alice.account_id()
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(account_json.account_id, alice.account_id());
    assert_eq!(account_json.stake_balance, U128(10000000000000000000000000000));
    assert!(account_json.reward.0 > 0);
    assert_eq!(account_json.unstake_balance.0, 0);
}

#[test]
pub fn test_deposit_and_stake_without_storage() {
    let (root, alice, ft_contract, staking_contract) = init();

    // Storage deposit
    // alice.call(
    //     staking_contract.account_id(), 
    //     "storage_deposit", 
    //     &json!({}).to_string().as_bytes(), 
    //     DEFAULT_GAS,
    //     to_yocto("0.01") 
    // );

    // Deposit token
    let outcome = alice.call(
        ft_contract.account_id(), 
        "ft_transfer_call", 
        &json!({
            "receiver_id": staking_contract.account_id(),
            "amount": ALICE_DEPOSIT_AMOUNT,
            "msg": ""
        }).to_string().as_bytes(), 
        DEFAULT_GAS, 
        1
    );

    assert_eq!(outcome.promise_errors().len(), 1);

    // assert error type
    if let ExecutionStatus::Failure(error) = &outcome.promise_errors().remove(0).unwrap().outcome().status {
        println!("Excute error: {}", error.to_string());
        assert!(error.to_string().contains("ERR_ACCOUNT_NOT_FOUND"));
    } else {
        unreachable!()
    }
}