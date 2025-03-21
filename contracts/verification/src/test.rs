#![cfg(test)]
extern crate std;

use crate::verification::{register_certificate, DataKey};
use crate::{VerificationContract, VerificationContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use std::string::ToString;

#[test]
fn test_certificate_verification() {
    // Set up test environment
    let env = Env::default();
    let admin = Address::generate(&env);
    let requester = Address::generate(&env);

    // Deploy contract
    let contract_id = env.register(VerificationContract, ());
    let client = VerificationContractClient::new(&env, &contract_id);

    // Initialize storage directly (would normally be done through a separate init function)
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextRequestId, &1u64);
    });

    // Register a test certificate
    let cert_id = 1234u64;
    env.as_contract(&contract_id, || {
        register_certificate(
            &env,
            cert_id,
            String::from_str(&env, "John Doe"),
            String::from_str(&env, "Blockchain Development"),
            String::from_str(&env, "Stellar University"),
            1677609600, // March 1, 2023
        );
    });

    // Verify the certificate exists
    let cert_details = client.verify_certificate(&cert_id);
    assert_eq!(cert_details.student.to_string(), "John Doe");
    assert_eq!(cert_details.course.to_string(), "Blockchain Development");
    assert_eq!(cert_details.institution.to_string(), "Stellar University");
    assert_eq!(cert_details.issuance_date, 1677609600);
    assert!(cert_details.valid);

    // Submit verification request
    env.mock_all_auths();
    let request_id = client.submit_verification_request(&cert_id, &requester);
    assert_eq!(request_id, 1);

    // List verification requests
    let requests = client.list_verification_requests(&cert_id);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests.get(0).unwrap().request_id, 1);
    assert_eq!(requests.get(0).unwrap().certificate_id, cert_id);

    // Revoke verification request
    client.revoke_verification_request(&admin, &request_id);

    // Check request was removed
    let requests_after = client.list_verification_requests(&cert_id);
    assert_eq!(requests_after.len(), 0);
}

#[test]
fn test_nonexistent_certificate() {
    let env = Env::default();
    let contract_id = env.register(VerificationContract, ());
    let client = VerificationContractClient::new(&env, &contract_id);

    // Try to verify a non-existent certificate
    let cert_details = client.verify_certificate(&9999u64);
    assert!(!cert_details.valid);
    assert_eq!(cert_details.student.to_string(), "");
}

#[test]
#[should_panic(expected = "Certificate does not exist")]
fn test_request_nonexistent_certificate() {
    let env = Env::default();
    let requester = Address::generate(&env);
    let contract_id = env.register(VerificationContract, ());
    let client = VerificationContractClient::new(&env, &contract_id);

    // Mock authentication
    env.mock_all_auths();

    // Try to submit a verification request for a non-existent certificate
    client.submit_verification_request(&9999u64, &requester);
}

#[test]
#[should_panic(expected = "Only admin can revoke verification requests")]
fn test_unauthorized_revocation() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let not_admin = Address::generate(&env);
    let requester = Address::generate(&env);

    // Deploy contract
    let contract_id = env.register(VerificationContract, ());
    let client = VerificationContractClient::new(&env, &contract_id);

    // Initialize storage
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextRequestId, &1u64);
    });

    // Register certificate
    let cert_id = 1234u64;
    env.as_contract(&contract_id, || {
        register_certificate(
            &env,
            cert_id,
            String::from_str(&env, "Jane Doe"),
            String::from_str(&env, "Smart Contracts"),
            String::from_str(&env, "Stellar University"),
            1677609600,
        );
    });

    // Submit verification request
    env.mock_all_auths();
    let request_id = client.submit_verification_request(&cert_id, &requester);

    // Try to revoke with non-admin address
    client.revoke_verification_request(&not_admin, &request_id);
    // Should panic with "Only admin can revoke verification requests"
}
