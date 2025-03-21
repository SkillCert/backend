#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, Symbol, Vec};
use Revoked::{RevokedRegistry, RevokedRegistryClient, ADMIN};

#[test]
fn test_set_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RevokedRegistry);
    let client = RevokedRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.Set_admin(&admin);

    let stored_admin: Address = env.as_contract(&contract_id, || {
        env.storage().instance().get(&ADMIN).unwrap()
    });

    assert_eq!(stored_admin, admin);
}

#[test]
#[should_panic(expected = "admin_already_set")]
fn test_set_admin_twice_should_fail() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RevokedRegistry);
    let client = RevokedRegistryClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    client.Set_admin(&admin1);
    client.Set_admin(&admin2); // Should panic
}

#[test]
fn test_revoke_certificate() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RevokedRegistry);
    let client = RevokedRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let certificate_id = 123 as u64;
    let reason = Bytes::from_slice(&env, b"Forgery");

    client.Set_admin(&admin);

    client.Revoke_Certificate(&admin, &certificate_id, &reason);

    let is_revoked = client.is_revoked(&certificate_id);
    assert!(is_revoked);
}

#[test]
#[should_panic(expected = "only_admin_can_revoke_certificates")]
fn test_non_admin_cannot_revoke_certificate() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RevokedRegistry);
    let client = RevokedRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let certificate_id = 123;
    let reason = Bytes::from_slice(&env, b"Forgery");

    client.Set_admin(&admin);

    // Non-admin tries to revoke a certificate (should fail)
    client.Revoke_Certificate(&non_admin, &certificate_id, &reason);
}

#[test]
fn test_get_revocation_details() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RevokedRegistry);
    let client = RevokedRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let certificate_id = 123;
    let reason = Bytes::from_slice(&env, b"Forgery");

    client.Set_admin(&admin);
    client.Revoke_Certificate(&admin, &certificate_id, &reason);

    let details = client
        .Get_Revocation_Details(&certificate_id)
        .expect("Certificate should be revoked");

    assert_eq!(details.0, admin);
    assert_eq!(details.1, reason);
}

#[test]
fn test_all_revoked_certificates() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RevokedRegistry);
    let client = RevokedRegistryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let cert1 = 101;
    let cert2 = 102;
    let reason1 = Bytes::from_slice(&env, b"Forgery");
    let reason2 = Bytes::from_slice(&env, b"Plagiarism");

    client.Set_admin(&admin);
    client.Revoke_Certificate(&admin, &cert1, &reason1);
    client.Revoke_Certificate(&admin, &cert2, &reason2);

    let revoked_list = client.All_Revoked_Certificates();
    assert_eq!(revoked_list.len(), 2);
}
