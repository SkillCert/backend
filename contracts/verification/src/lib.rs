#![no_std]
mod test;
mod verification;

use soroban_sdk::{contract, contractimpl, Address, Env, Vec};
use verification::{
    list_verification_requests, revoke_verification_request, submit_verification_request,
    verify_certificate, VerificationRequest,
};

#[contract]
pub struct VerificationContract;

#[contractimpl]
impl VerificationContract {
    pub fn verify_certificate(env: Env, certificate_id: u64) -> verification::CertificateDetails {
        verify_certificate(env, certificate_id)
    }

    pub fn submit_verification_request(env: Env, certificate_id: u64, requester: Address) -> u64 {
        submit_verification_request(env, certificate_id, requester)
    }

    pub fn list_verification_requests(env: Env, certificate_id: u64) -> Vec<VerificationRequest> {
        list_verification_requests(env, certificate_id)
    }

    pub fn revoke_verification_request(env: Env, admin: Address, request_id: u64) {
        revoke_verification_request(env, admin, request_id)
    }
}
