#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

use crate::contract::{InstitutionContract, InstitutionContractClient};

// use crate::{InstitutionContract, InstitutionContractClient};

#[test]
fn test_register_institution() {
    let env = Env::default();
    let contract_id = env.register(InstitutionContract, ());
    let client = InstitutionContractClient::new(&env, &contract_id);

    let wallet = Address::generate(&env);
    let name = Bytes::from_slice(&env, b"Test Institution");
    let metadata = Bytes::from_slice(&env, b"Metadata");

    let id = client.register_institution(&name, &wallet, &metadata);
    let institution = client.get_institution(&id);

    assert_eq!(institution.name, name);
    assert_eq!(institution.wallet, wallet);
    assert!(!institution.verified);
}

#[test]
fn test_verify_institution() {
    let env = Env::default();
    let contract_id = env.register(InstitutionContract, ());
    let client = InstitutionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    let name = Bytes::from_slice(&env, b"Institution");
    let metadata = Bytes::from_slice(&env, b"Metadata");

    client.set_admin(&admin);
    let id = client.register_institution(&name, &wallet, &metadata);
    env.mock_all_auths();

    client.verify_institution(&id, &admin);

    let institution = client.get_institution(&id);
    assert!(institution.verified);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_non_admin_cannot_verify_institution() {
    let env = Env::default();
    let contract_id = env.register(InstitutionContract, ());
    let client = InstitutionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    let name = Bytes::from_slice(&env, b"Institution");
    let metadata = Bytes::from_slice(&env, b"Metadata");

    client.set_admin(&admin);
    env.mock_all_auths();

    let id = client.register_institution(&name, &wallet, &metadata);

    client.verify_institution(&id, &non_admin);
}

#[test]
fn test_list_institutions() {
    let env = Env::default();
    let contract_id = env.register(InstitutionContract, ());
    let client = InstitutionContractClient::new(&env, &contract_id);

    let wallet1 = Address::generate(&env);
    let wallet2 = Address::generate(&env);
    let name1 = Bytes::from_slice(&env, b"Institution 1");
    let name2 = Bytes::from_slice(&env, b"Institution 2");
    let metadata = Bytes::from_slice(&env, b"Metadata");

    client.register_institution(&name1, &wallet1, &metadata);
    client.register_institution(&name2, &wallet2, &metadata);

    let institutions = client.list_institutions();
    assert_eq!(institutions.len(), 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_remove_institution() {
    let env = Env::default();
    let contract_id = env.register(InstitutionContract, ());
    let client = InstitutionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    let name = Bytes::from_slice(&env, b"Institution");
    let metadata = Bytes::from_slice(&env, b"Metadata");

    client.set_admin(&admin);
    let id = client.register_institution(&name, &wallet, &metadata);
    env.mock_all_auths();

    client.remove_institution(&id, &admin);

    client.get_institution(&id);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_remove_institution() {
    let env = Env::default();
    let contract_id = env.register(InstitutionContract, ());
    let client = InstitutionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let wallet = Address::generate(&env);
    let name = Bytes::from_slice(&env, b"Institution");
    let metadata = Bytes::from_slice(&env, b"Metadata");

    client.set_admin(&admin);
    let id = client.register_institution(&name, &wallet, &metadata);
    client.remove_institution(&id, &non_admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_set_admin_twice_should_fail() {
    let env = Env::default();
    let contract_id = env.register(InstitutionContract, ());
    let client = InstitutionContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    client.set_admin(&admin1);
    client.set_admin(&admin2);
}
