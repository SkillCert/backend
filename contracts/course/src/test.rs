#![cfg(test)]

use crate::course::{CourseContract, CourseContractClient, Course, Error};

use soroban_sdk::{
    testutils::{Address as _},
    Bytes, Map, Val, IntoVal, Symbol, 
    Address, Env, contract, contractimpl
};


fn mock_institution_fn(env: &Env, is_verified: bool) -> Address {
    let contract_id = env.register_contract(None, InstitutionContractMock);
    env.as_contract(&contract_id, || {
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
        let key = Bytes::from_slice(&env, format!("is_verified_{}", id).as_bytes());
        let is_verified = env.storage().persistent().get(&key).unwrap_or(false);
        result.set(Bytes::from_slice(&env, b"verified"), is_verified.into_val(&env));
        result
    }

    pub fn is_verified(env: Env, id: u64) -> bool {
        let key = Bytes::from_slice(&env, format!("is_verified_{}", id).as_bytes());
        env.storage().persistent().get(&key).unwrap_or(false)
    }
}

#[contract]
struct CertificateContractMock;

#[contractimpl]
impl CertificateContractMock {
    pub fn issue_certificate(
        _env: Env,
        _student: Address,
        course_id: u64,
        _institution: Address,
        _metadata: Bytes,
    ) -> u64 {
        course_id + 1000
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
    
    let client = CourseContractClient::new(&env, &course_contract);
    
    let course_id = client.create_course(
        &title,
        &institution,
        &price,
        &metadata,
        &certificate_id,
        &institution_contract
    );
    
    let course = client.get_course(&course_id);
    
    assert_eq!(course.title, title);
    assert_eq!(course.institution, institution);
    assert_eq!(course.price, price);
    assert_eq!(course.metadata, metadata);
    assert_eq!(course.certificate_id, certificate_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_enroll_duplicate_panics() {
    let env = Env::default();
    let course_contract = env.register_contract(None, CourseContract);
    
    let institution_contract = mock_institution_fn(&env, true);
    let institution = Address::generate(&env);
    let student = Address::generate(&env);
    
    let title = Bytes::from_slice(&env, b"Blockchain Development");
    let price = 50;
    let metadata = Bytes::from_slice(&env, b"ipfs://QmHash");
    let certificate_id = 1;
    
    env.mock_all_auths();
    
    let client = CourseContractClient::new(&env, &course_contract);
    
    let course_id = client.create_course(
        &title,
        &institution,
        &price,
        &metadata,
        &certificate_id,
        &institution_contract
    );
    
    client.enroll_in_course(&course_id, &student);
    
    client.enroll_in_course(&course_id, &student);
}

#[test]
fn test_complete_course() {
    let env = Env::default();
    let course_contract = env.register_contract(None, CourseContract);
    
    let institution_contract = mock_institution_fn(&env, true);
    let certificate_contract = env.register_contract(None, CertificateContractMock);
    
    let institution = Address::generate(&env);
    let student = Address::generate(&env);
    
    let title = Bytes::from_slice(&env, b"Soroban Smart Contracts");
    let price = 75;
    let metadata = Bytes::from_slice(&env, b"ipfs://QmMetadata");
    let certificate_id = 2;
    
    env.mock_all_auths();
    
    let client = CourseContractClient::new(&env, &course_contract);
    
    let course_id = client.create_course(
        &title,
        &institution,
        &price,
        &metadata,
        &certificate_id,
        &institution_contract
    );
    
    client.enroll_in_course(&course_id, &student);
    
    client.complete_course(&course_id, &student, &certificate_contract);
    
    client.complete_course(&course_id, &student, &certificate_contract);
}

#[test]
fn test_remove_course() {
    let env = Env::default();
    let course_contract = env.register_contract(None, CourseContract);
    
    let institution_contract = mock_institution_fn(&env, true);
    let institution = Address::generate(&env);
    
    let title = Bytes::from_slice(&env, b"Temporary Course");
    let price = 30;
    let metadata = Bytes::from_slice(&env, b"ipfs://QmTempCourse");
    let certificate_id = 3;
    
    env.mock_all_auths();
    
    let client = CourseContractClient::new(&env, &course_contract);
    
    let course_id = client.create_course(
        &title,
        &institution,
        &price,
        &metadata,
        &certificate_id,
        &institution_contract
    );
    
    let courses = client.list_courses();
    assert_eq!(courses.len(), 1);
    
    client.remove_course(&course_id, &institution);
    
    let courses_after = client.list_courses();
    assert_eq!(courses_after.len(), 0);
    
    let success = {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.get_course(&course_id);
        }));
        result.is_ok()
    };
    
    assert!(!success, "Expected get_course to panic for a removed course");
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_remove_course_with_enrolled_students() {
    let env = Env::default();
    let course_contract = env.register_contract(None, CourseContract);
    
    let institution_contract = mock_institution_fn(&env, true);
    let institution = Address::generate(&env);
    let student = Address::generate(&env);
    
    let title = Bytes::from_slice(&env, b"Popular Course");
    let price = 40;
    let metadata = Bytes::from_slice(&env, b"ipfs://QmPopularCourse");
    let certificate_id = 4;
    
    env.mock_all_auths();
    
    let client = CourseContractClient::new(&env, &course_contract);
    
    let course_id = client.create_course(
        &title,
        &institution,
        &price,
        &metadata,
        &certificate_id,
        &institution_contract
    );
    
    client.enroll_in_course(&course_id, &student);
    
    client.remove_course(&course_id, &institution);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_unauthorized_operations() {
    let env = Env::default();
    let course_contract = env.register_contract(None, CourseContract);
    
    let institution_contract = mock_institution_fn(&env, true);
    let institution = Address::generate(&env);
    let unauthorized_institution = Address::generate(&env);
    
    let title = Bytes::from_slice(&env, b"Secure Course");
    let price = 60;
    let metadata = Bytes::from_slice(&env, b"ipfs://QmSecureCourse");
    let certificate_id = 5;
    
    env.mock_all_auths();
    
    let client = CourseContractClient::new(&env, &course_contract);
    
    let course_id = client.create_course(
        &title,
        &institution,
        &price,
        &metadata,
        &certificate_id,
        &institution_contract
    );
    
    client.remove_course(&course_id, &unauthorized_institution);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_unverified_institution() {
    let env = Env::default();
    let course_contract = env.register_contract(None, CourseContract);
    
    let unverified_institution_contract = mock_institution_fn(&env, false);
    let institution = Address::generate(&env);
    
    let title = Bytes::from_slice(&env, b"Unverified Course");
    let price = 20;
    let metadata = Bytes::from_slice(&env, b"ipfs://QmUnverifiedCourse");
    let certificate_id = 6;
    
    env.mock_all_auths();
    
    let client = CourseContractClient::new(&env, &course_contract);
    
    client.create_course(
        &title,
        &institution,
        &price,
        &metadata,
        &certificate_id,
        &unverified_institution_contract
    );
}
