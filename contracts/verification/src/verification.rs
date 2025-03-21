use soroban_sdk::{contracttype, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct VerificationRequest {
    pub request_id: u64,
    pub certificate_id: u64,
    pub requester: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct CertificateDetails {
    pub student: String,
    pub course: String,
    pub institution: String,
    pub issuance_date: u64,
    pub valid: bool,
}

#[contracttype]
pub enum DataKey {
    Admin,
    NextRequestId,
    VerificationRequest(u64), // request_id -> VerificationRequest
    CertificateRequests(u64), // certificate_id -> Vec<request_id>
    Certificate(u64),         // certificate_id -> CertificateDetails
}

pub fn submit_verification_request(env: Env, certificate_id: u64, requester: Address) -> u64 {
    // Require authorization from requester
    requester.require_auth();

    // Check if certificate exists
    if !certificate_exists(&env, certificate_id) {
        panic!("Certificate does not exist");
    }

    // Get next request ID
    let request_id = get_next_request_id(&env);

    // Create verification request
    let request = VerificationRequest {
        request_id,
        certificate_id,
        requester: requester.clone(),
        timestamp: env.ledger().timestamp(),
    };

    // Store verification request
    env.storage()
        .instance()
        .set(&DataKey::VerificationRequest(request_id), &request);

    // Add request ID to certificate's request list
    let mut cert_requests: Vec<u64> = env
        .storage()
        .instance()
        .get(&DataKey::CertificateRequests(certificate_id))
        .unwrap_or_else(|| Vec::new(&env));

    cert_requests.push_back(request_id);
    env.storage().instance().set(
        &DataKey::CertificateRequests(certificate_id),
        &cert_requests,
    );

    request_id
}

pub fn verify_certificate(env: Env, certificate_id: u64) -> CertificateDetails {
    // Check if certificate exists
    if !certificate_exists(&env, certificate_id) {
        return CertificateDetails {
            student: String::from_str(&env, ""),
            course: String::from_str(&env, ""),
            institution: String::from_str(&env, ""),
            issuance_date: 0,
            valid: false,
        };
    }

    // Get certificate details
    let certificate_details: CertificateDetails = env
        .storage()
        .instance()
        .get(&DataKey::Certificate(certificate_id))
        .unwrap();

    certificate_details
}

pub fn list_verification_requests(env: Env, certificate_id: u64) -> Vec<VerificationRequest> {
    // Get request IDs for certificate
    let request_ids: Vec<u64> = env
        .storage()
        .instance()
        .get(&DataKey::CertificateRequests(certificate_id))
        .unwrap_or_else(|| Vec::new(&env));

    // Get request details
    let mut requests: Vec<VerificationRequest> = Vec::new(&env);
    for i in 0..request_ids.len() {
        let request_id = request_ids.get(i).unwrap();
        let request: VerificationRequest = env
            .storage()
            .instance()
            .get(&DataKey::VerificationRequest(request_id))
            .unwrap();
        requests.push_back(request);
    }

    requests
}

pub fn revoke_verification_request(env: Env, admin: Address, request_id: u64) {
    // Require admin authorization
    admin.require_auth();

    // Check if admin is authorized
    let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();

    if stored_admin != admin {
        panic!("Only admin can revoke verification requests");
    }

    // Get verification request
    let request: VerificationRequest = env
        .storage()
        .instance()
        .get(&DataKey::VerificationRequest(request_id))
        .unwrap();

    // Remove request from certificate's request list
    let cert_requests: Vec<u64> = env
        .storage()
        .instance()
        .get(&DataKey::CertificateRequests(request.certificate_id))
        .unwrap_or_else(|| Vec::new(&env));

    // Filter out the revoked request
    let mut new_requests: Vec<u64> = Vec::new(&env);
    for i in 0..cert_requests.len() {
        let id = cert_requests.get(i).unwrap();
        if id != request_id {
            new_requests.push_back(id);
        }
    }

    // Update certificate requests
    env.storage().instance().set(
        &DataKey::CertificateRequests(request.certificate_id),
        &new_requests,
    );

    // Remove verification request
    env.storage()
        .instance()
        .remove(&DataKey::VerificationRequest(request_id));
}

// Helper functions
fn certificate_exists(env: &Env, certificate_id: u64) -> bool {
    env.storage()
        .instance()
        .has(&DataKey::Certificate(certificate_id))
}

fn get_next_request_id(env: &Env) -> u64 {
    let id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextRequestId)
        .unwrap_or(1);

    // Increment and store next ID
    env.storage()
        .instance()
        .set(&DataKey::NextRequestId, &(id + 1));

    id
}

// Certificate management functions (normally would be in another contract)
// Added here to support testing and functionality

pub fn register_certificate(
    env: &Env,
    certificate_id: u64,
    student: String,
    course: String,
    institution: String,
    issuance_date: u64,
) {
    let certificate = CertificateDetails {
        student,
        course,
        institution,
        issuance_date,
        valid: true,
    };

    env.storage()
        .instance()
        .set(&DataKey::Certificate(certificate_id), &certificate);
}
