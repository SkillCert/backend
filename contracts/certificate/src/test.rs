#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{vec, Env};

#[test]
fn test_certificate_lifecycle() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CertificateContract);
    let client = CertificateContract::new(&env, &contract_id);

    // Setup test accounts
    let student = Address::generate(&env);
    let institution = Address::generate(&env);
    let course_id = 1u64;
    let metadata = Bytes::from_array(&env, &[1, 2, 3]);

    // Test certificate issuance
    env.mock_all_auths();
    let cert_id = client.issue_certificate(&student, &course_id, &institution, &metadata);
    
    // Test certificate verification
    let cert = client.verify_certificate(&cert_id);
    assert_eq!(cert.student, student);
    assert_eq!(cert.course_id, course_id);
    assert_eq!(cert.institution, institution);
    assert_eq!(cert.metadata, metadata);
    assert_eq!(cert.status, true);

    // Test listing certificates
    let certs = client.list_certificates(&student);
    assert_eq!(certs.len(), 1);
    assert_eq!(certs.get(0).unwrap().id, cert_id);

    // Test certificate revocation
    env.mock_all_auths();
    client.revoke_certificate(&cert_id);
    let revoked_cert = client.verify_certificate(&cert_id);
    assert_eq!(revoked_cert.status, false);
}

#[test]
fn test_multiple_certificates() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CertificateContract);
    let client = CertificateContract::new(&env, &contract_id);

    let student = Address::generate(&env);
    let institution = Address::generate(&env);
    
    env.mock_all_auths();
    
    // Issue multiple certificates
    let cert_id1 = client.issue_certificate(
        &student,
        &1u64,
        &institution,
        &Bytes::from_array(&env, &[1]),
    );
    
    let cert_id2 = client.issue_certificate(
        &student,
        &2u64,
        &institution,
        &Bytes::from_array(&env, &[2]),
    );

    // Verify student has multiple certificates
    let certs = client.list_certificates(&student);
    assert_eq!(certs.len(), 2);
    
    // Verify both certificates are valid
    assert_eq!(client.verify_certificate(&cert_id1).status, true);
    assert_eq!(client.verify_certificate(&cert_id2).status, true);
}