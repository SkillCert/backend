#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, Vec, panic_with_error};

#[derive(Clone)]
#[contracttype]
pub enum Error {
    CertificateNotFound,
    Unauthorized,
}

impl From<Error> for soroban_sdk::Error {
    fn from(e: Error) -> Self {
        Self::from_contract_error(e as u32)
    }
}

#[derive(Clone)]
#[contracttype]
pub struct Certificate {
    pub id: u64,
    pub student: Address,
    pub course_id: u64,
    pub institution: Address,
    pub issued_at: u64,
    pub metadata: Bytes,
    pub status: bool,
}

#[contract]
pub struct CertificateContract;

#[contractimpl]
impl CertificateContract {
    pub fn issue_certificate(
        env: Env,
        student: Address,
        course_id: u64,
        institution: Address,
        metadata: Bytes,
    ) -> u64 {
        // Verify that the caller is the institution
        institution.require_auth();

        // Generate a unique certificate ID
        let id: u64 = env.ledger().sequence().into();

        // Create new certificate
        let certificate = Certificate {
            id,
            student: student.clone(),
            course_id,
            institution,
            issued_at: env.ledger().timestamp().into(),
            metadata,
            status: true,
        };

        // Store the certificate
        env.storage().persistent().set(&id, &certificate);

        // Add to student's certificates list
        let mut student_certs = Self::get_student_certificates(&env, &student);
        student_certs.push_back(id);
        env.storage().persistent().set(&student, &student_certs);

        id
    }

    pub fn verify_certificate(env: Env, id: u64) -> Certificate {
        env.storage().persistent()
            .get(&id)
            .unwrap_or_else(|| panic_with_error!(&env, Error::CertificateNotFound))
    }

    pub fn revoke_certificate(env: Env, id: u64) {
        // Get the certificate
        let mut certificate: Certificate = env.storage().persistent()
            .get(&id)
            .unwrap_or_else(|| panic_with_error!(&env, Error::CertificateNotFound));
        
        // Verify that the caller is the issuing institution
        certificate.institution.require_auth();

        // Update certificate status
        certificate.status = false;
        
        // Store updated certificate
        env.storage().persistent().set(&id, &certificate);
    }

    pub fn list_certificates(env: Env, student: Address) -> Vec<Certificate> {
        let cert_ids = Self::get_student_certificates(&env, &student);
        let mut certificates = Vec::new(&env);

        for id in cert_ids.iter() {
            if let Some(cert) = env.storage().persistent().get(&id) {
                certificates.push_back(cert);
            }
        }

        certificates
    }

    // Helper function to get student's certificates IDs
    fn get_student_certificates(env: &Env, student: &Address) -> Vec<u64> {
        env.storage().persistent().get(student).unwrap_or_else(|| Vec::new(env))
    }
}