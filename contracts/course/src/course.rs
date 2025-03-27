#![no_std] 
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error,  Address, Bytes, Env, Symbol, Vec, Val, IntoVal, Map
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    CourseNotFound = 1,
    Unauthorized = 2,
    InstitutionNotVerified = 3,
    InsufficientFunds = 4,
    StudentAlreadyEnrolled = 5,
    StudentsEnrolled = 6,
}

#[derive(Clone)]
#[contracttype]
pub struct Course {
    pub id: u64,
    pub title: Bytes,
    pub institution: Address,
    pub price: u64,
    pub metadata: Bytes,
    pub certificate_id: u64,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Enrollment {
    pub student: Address,
    pub course_id: u64,
    pub enrolled_at: u64,
    pub completed: bool,
}

#[contract]
pub struct CourseContract;


pub struct CertificateClient<'a> {
    env: &'a Env,
    contract_id: Address,
}

impl<'a> CertificateClient<'a> {
    pub fn new(env: &'a Env, contract_id: Address) -> Self {
        Self { env, contract_id }
    }

    pub fn issue_certificate(
        &self,
        student: Address,
        course_id: u64,
        institution: Address,
        metadata: Bytes,
    ) -> u64 {
        let args = (
            student,
            course_id,
            institution.clone(),
            metadata
        );

        self.env
            .invoke_contract(
                &self.contract_id,
                Symbol::from_str(&self.env, "issue_certificate").unwrap(),
                args.into_val(self.env),
            ).into_val(self.env)
    }
}

pub struct InstitutionClient<'a> {
    env: &'a Env,
    contract_id: Address,
}

impl<'a> InstitutionClient<'a> {
    pub fn new(env: &'a Env, contract_id: Address) -> Self {
        Self { env, contract_id }
    }

    pub fn get_institution(&self, id: u64) -> Map<Bytes, Val> {
        let args = (id,);

        self.env
            .invoke_contract(
                &self.contract_id,
                Symbol::from_str(&self.env, "get_institution").unwrap(),
                args.into_val(self.env),
            ).into_val(self.env)
    }
}
    
#[contractimpl]
impl CourseContract {
    pub fn create_course(
        env: Env,
        title: Bytes,
        institution: Address,
        price: u64,
        metadata: Bytes,
        certificate_id: u64,
        institution_contract_id: Address,
    ) -> u64 {
        institution.require_auth();

        let institution_client = InstitutionClient::new(&env, institution_contract_id);
        let institution_data = institution_client.get_institution(1);

        if!institution_data.get(Bytes::from_slice(&env, b"verified"))
           .unwrap_or_else(|| false.into_val(&env))
           .is_truthy() {
            panic_with_error!(&env, Error::InstitutionNotVerified);
        }

        let id: u64 = env.ledger().sequence().into();
        let created_at = env.ledger().timestamp();

        let course = Course {
            id,
            title,
            institution: institution.clone(),
            price,
            metadata,
            certificate_id,
            created_at,
        };

        env.storage().persistent().set(&id, &course);

        let courses_key = Bytes::from_slice(&env, b"courses");
        let mut course_ids = env
            .storage()
            .persistent()
            .get::<Bytes, Vec<u64>>(&courses_key)
            .unwrap_or_else(|| Vec::new(&env));

        course_ids.push_back(id);
        env.storage().persistent().set(&courses_key, &course_ids);

        id
    }
}
