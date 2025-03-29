#![no_std] 
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, 
    Address, Bytes, Env, Symbol, Vec, Val, IntoVal, Map, TryFromVal
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
            .invoke_contract::<u64>(
                &self.contract_id,
                &Symbol::new(self.env, "issue_certificate"),
                args.into_val(self.env),
            )
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

    pub fn get_institution(&self, id: u64) -> Val {
        let args = (id,);

        self.env
            .invoke_contract::<Val>(
                &self.contract_id,
                &Symbol::new(self.env, "get_institution"),
                args.into_val(self.env),
            )
    }
    
    pub fn is_verified(&self, id: u64) -> bool {
        let data = self.get_institution(id);
        
        if let Ok(map) = Map::<Bytes, Val>::try_from_val(self.env, &data) {
            if let Some(verified) = map.get(Bytes::from_slice(self.env, b"verified")) {
                return verified.is_true();
            }
        }
        
        false
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
        
        if !institution_client.is_verified(1) {
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

    pub fn enroll_in_course(env: Env, course_id: u64, student: Address) {
        student.require_auth();
        
        let _course: Course = env
            .storage()
            .persistent()
            .get(&course_id)
            .unwrap_or_else(|| panic_with_error!(&env, Error::CourseNotFound));
            
        let enrollment_key = Self::get_enrollment_key(&env, &course_id, &student);
        if env.storage().persistent().has(&enrollment_key) {
            panic_with_error!(&env, Error::StudentAlreadyEnrolled);
        }
        
        
        let enrollment = Enrollment {
            student: student.clone(),
            course_id,
            enrolled_at: env.ledger().timestamp(),
            completed: false,
        };
        
        env.storage().persistent().set(&enrollment_key, &enrollment);
        
        let student_courses_key = Self::get_student_courses_key(&env, &student);
        let mut student_courses = env
            .storage()
            .persistent()
            .get::<Bytes, Vec<u64>>(&student_courses_key)
            .unwrap_or_else(|| Vec::new(&env));
            
        student_courses.push_back(course_id);
        env.storage().persistent().set(&student_courses_key, &student_courses);
        
        let course_students_key = Self::get_course_students_key(&env, &course_id);
        let mut course_students = env
            .storage()
            .persistent()
            .get::<Bytes, Vec<Address>>(&course_students_key)
            .unwrap_or_else(|| Vec::new(&env));
            
        course_students.push_back(student.clone());
        env.storage().persistent().set(&course_students_key, &course_students);
    }
    
    pub fn complete_course(
        env: Env, 
        course_id: u64, 
        student: Address, 
        certificate_contract_id: Address
    ) {
        let course: Course = env
            .storage()
            .persistent()
            .get(&course_id)
            .unwrap_or_else(|| panic_with_error!(&env, Error::CourseNotFound));
            
        course.institution.require_auth();
        
        let enrollment_key = Self::get_enrollment_key(&env, &course_id, &student);
        let mut enrollment: Enrollment = env
            .storage()
            .persistent()
            .get(&enrollment_key)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Unauthorized));
            
        enrollment.completed = true;
        env.storage().persistent().set(&enrollment_key, &enrollment);
        
        let certificate_client = CertificateClient::new(&env, certificate_contract_id);
        certificate_client.issue_certificate(
            student,
            course_id,
            course.institution,
            course.metadata
        );
    }
    
    pub fn get_course(env: Env, id: u64) -> Course {
        env.storage()
            .persistent()
            .get(&id)
            .unwrap_or_else(|| panic_with_error!(&env, Error::CourseNotFound))
    }
    
    pub fn list_courses(env: Env) -> Vec<Course> {
        let mut courses = Vec::new(&env);
        
        let courses_key = Bytes::from_slice(&env, b"courses");
        let course_ids = env
            .storage()
            .persistent()
            .get::<Bytes, Vec<u64>>(&courses_key)
            .unwrap_or_else(|| Vec::new(&env));
            
        for i in 0..course_ids.len() {
            let course_id = course_ids.get(i).unwrap();
            if let Some(course) = env.storage().persistent().get::<u64, Course>(&course_id) {
                courses.push_back(course);
            }
        }
        
        courses
    }
    
    pub fn remove_course(env: Env, id: u64, institution: Address) {
        institution.require_auth();
        
        let course: Course = env
            .storage()
            .persistent()
            .get(&id)
            .unwrap_or_else(|| panic_with_error!(&env, Error::CourseNotFound));
            
        if course.institution != institution {
            panic_with_error!(&env, Error::Unauthorized);
        }
        
        let course_students_key = Self::get_course_students_key(&env, &id);
        let course_students = env
            .storage()
            .persistent()
            .get::<Bytes, Vec<Address>>(&course_students_key)
            .unwrap_or_else(|| Vec::new(&env));
            
        if !course_students.is_empty() {
            panic_with_error!(&env, Error::StudentsEnrolled);
        }
        
        env.storage().persistent().remove(&id);
        
        let courses_key = Bytes::from_slice(&env, b"courses");
        let course_ids = env
            .storage()
            .persistent()
            .get::<Bytes, Vec<u64>>(&courses_key)
            .unwrap_or_else(|| Vec::new(&env));
            
        let mut new_course_ids = Vec::new(&env);
        for i in 0..course_ids.len() {
            let course_id = course_ids.get(i).unwrap();
            if course_id != id {
                new_course_ids.push_back(course_id);
            }
        }
        
        env.storage().persistent().set(&courses_key, &new_course_ids);
    }
    
    fn get_enrollment_key(env: &Env, course_id: &u64, student: &Address) -> Bytes {
        let mut key = Bytes::new(env);
        key.append(&Bytes::from_slice(env, b"enrollment_"));
        key.append(&Bytes::from_array(env, &course_id.to_le_bytes()));
        key.append(&Bytes::from_slice(env, b"_"));
        
        key.append(&Bytes::from_slice(env, b"address_"));
        
        let address_bytes = Bytes::from_slice(env, student.to_string().to_string().as_str().as_bytes());
        key.append(&address_bytes);
        
        key
    }
    
    fn get_student_courses_key(env: &Env, student: &Address) -> Bytes {
        let mut key = Bytes::new(env);
        key.append(&Bytes::from_slice(env, b"student_courses_"));
        
        key.append(&Bytes::from_slice(env, b"address_"));
        
        let address_bytes = Bytes::from_slice(env, student.to_string().to_string().as_str().as_bytes());
        key.append(&address_bytes);
        let address_bytes = Bytes::from_slice(env, student.to_string().to_string().as_str().as_bytes());
        key.append(&address_bytes);
        
        key
    }

    fn get_course_students_key(env: &Env, course_id: &u64) -> Bytes {
        let mut key = Bytes::new(env);
        key.append(&Bytes::from_slice(env, b"course_students_"));
        key.append(&Bytes::from_array(env, &course_id.to_le_bytes()));
        key
    }
}
