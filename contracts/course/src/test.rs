#![cfg(test)]

use crate::course::{CourseContract, CourseContractClient, Course};

use soroban_sdk::{
    testutils::{Address as _},
    Bytes, Map, Val, IntoVal, Symbol, Vec,
    Address, Env, contract, contractimpl
};


fn mock_institution_fn(env: &Env, is_verified: bool) -> Address {
    let contract_id = env.register_contract(None, InstitutionContractMock);
    env.as_contract(&contract_id, || {
        // Store with ID 1 to match the specific check in the contract
        let key = Bytes::from_slice(env, b"is_verified_1");
        env.storage().persistent().set(&key, &is_verified);
    });
    contract_id
}

#[contract]
struct InstitutionContractMock;

#[contractimpl]
impl InstitutionContractMock {
    pub fn get_institution(env: Env, id: u64) -> Map<Bytes, Val> {
        let mut result = Map::new(&env);
        // Get verification status for specific ID
        let key = Bytes::from_slice(&env, format!("is_verified_{}", id).as_bytes());
        let is_verified = env.storage().persistent().get(&key).unwrap_or(false);
        result.set(Bytes::from_slice(&env, b"verified"), is_verified.into_val(&env));
        result
    }

    pub fn is_verified(env: Env, id: u64) -> bool {
        // Match the ID check from CourseContract.create_course
        let key = Bytes::from_slice(&env, format!("is_verified_{}", id).as_bytes());
        env.storage().persistent().get(&key).unwrap_or(false)
    }
}

#[test]
fn test_list_courses() {
    let env = Env::default();
    
    let contract_id = env.register_contract(None, CourseContract);
    
    let client = CourseContractClient::new(&env, &contract_id);
    
    let courses = client.list_courses();
    assert_eq!(courses.len(), 0);
}

#[test]
fn test_create_course() {
    let env = Env::default();
    let course_contract = env.register_contract(None, CourseContract);
    
    let institution_contract = mock_institution_fn(&env, true);
    
    let institution = Address::generate(&env);
    
    let title = Bytes::from_slice(&env, b"Rust Programming 101");
    let price = 100;
    let metadata = Bytes::from_slice(&env, b"ipfs://QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco");
    let certificate_id = 1;
    
    env.mock_all_auths();
    
    let course_id_val: Val = env.invoke_contract(
        &course_contract,
        &Symbol::new(&env, "create_course"),
        (
            title.clone(),
            institution.clone(),
            price,
            metadata.clone(),
            certificate_id,
            institution_contract,
        ).into_val(&env),
    );
    let course_id: u64 = course_id_val.into_val(&env);
    
    let course_val: Val = env.invoke_contract(
        &course_contract,
        &Symbol::new(&env, "get_course"),
        (course_id,).into_val(&env),
    );
    let course: crate::course::Course = course_val.into_val(&env);
    
    assert_eq!(course.title, title);
    assert_eq!(course.institution, institution);
    assert_eq!(course.price, price);
    assert_eq!(course.metadata, metadata);
    assert_eq!(course.certificate_id, certificate_id);
}

