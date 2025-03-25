use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Bytes, Env, Vec
};


#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]

pub enum Error {
    InstitutionNotFound = 1,
    Unauthorized = 2,
    AdminNotSet = 3,
    InstitutionHasCertificates = 4,
    AdminAlreadySet = 5,
}


#[derive(Clone)]
#[contracttype]
pub struct Institution {
    pub id: u64,
    pub name: Bytes,
    pub wallet: Address,
    pub verified: bool,
    pub metadata: Bytes,
    pub created_at: u64,
}

#[contract]
pub struct InstitutionContract;

#[contractimpl]
impl InstitutionContract {
    pub fn register_institution(env: Env, name: Bytes, wallet: Address, metadata: Bytes) -> u64 {
        let id: u64 = env.ledger().sequence().into();
        let created_at = env.ledger().timestamp();

        let institution = Institution {
            id,
            name,
            wallet: wallet.clone(),
            verified: false,
            metadata,
            created_at,
        };

        env.storage().persistent().set(&id, &institution);

        let key = Bytes::from_slice(&env, b"institutions");
        let mut institution_ids = env
            .storage()
            .persistent()
            .get::<Bytes, Vec<u64>>(&key)
            .unwrap_or_else(|| Vec::new(&env));

        institution_ids.push_back(id);
        env.storage().persistent().set(&key, &institution_ids);

        id
    }

    pub fn verify_institution(env: Env, id: u64, admin: Address) {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            panic_with_error!(&env, Error::Unauthorized);
        }
        let mut institution: Institution = env
            .storage()
            .persistent()
            .get(&id)
            .unwrap_or_else(|| panic_with_error!(&env, Error::InstitutionNotFound));

        institution.verified = true;
        env.storage().persistent().set(&id, &institution);
    }

    pub fn get_institution(env: Env, id: u64) -> Institution {
        env.storage()
            .persistent()
            .get(&id)
            .unwrap_or_else(|| panic_with_error!(&env, Error::InstitutionNotFound))
    }

    pub fn remove_institution(env: Env, id: u64, admin: Address) {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            panic_with_error!(&env, Error::Unauthorized);
        }

        let mut certs_key_bytes = Bytes::new(&env);
        certs_key_bytes.append(&Bytes::from_slice(&env, b"certs_"));
        certs_key_bytes.append(&Bytes::from_array(&env, &id.to_le_bytes()));

        let has_issued_certs = env.storage().persistent().has(&certs_key_bytes);
        if has_issued_certs {
            panic_with_error!(&env, Error::InstitutionHasCertificates);
        }

        env.storage().persistent().remove(&id);

        let key = Bytes::from_slice(&env, b"institutions");
        let institution_ids = env
            .storage()
            .persistent()
            .get::<Bytes, Vec<u64>>(&key)
            .unwrap_or_else(|| Vec::new(&env));

        let mut new_institution_ids = Vec::new(&env);
        for inst_id in institution_ids.iter() {
            if inst_id != id {
                new_institution_ids.push_back(inst_id);
            }
        }

        env.storage().persistent().set(&key, &new_institution_ids);
    }

    pub fn list_institutions(env: Env) -> Vec<Institution> {
        let mut institutions = Vec::new(&env);

        let key = Bytes::from_slice(&env, b"institutions");
        let institution_ids = env
            .storage()
            .persistent()
            .get::<Bytes, Vec<u64>>(&key)
            .unwrap_or_else(|| Vec::new(&env));

        for id in institution_ids.iter() {
            if let Some(inst) = env.storage().persistent().get::<u64, Institution>(&id) {
                institutions.push_back(inst);
            }
        }

        institutions
    }

    pub fn set_admin(env: Env, admin: Address) {
        let key = Bytes::from_slice(&env, b"admin");
        if env.storage().persistent().has(&key) {
            panic_with_error!(&env, Error::AdminAlreadySet);
        }
        env.storage().persistent().set(&key, &admin);
    }
}

fn get_admin(env: &Env) -> Option<Address> {
    env.storage()
        .persistent()
        .get::<Bytes, Address>(&Bytes::from_slice(env, b"admin"))
}

fn is_admin(env: &Env, admin: &Address) -> bool {
    get_admin(env).map_or(false, |admin_address| *admin == admin_address)
}
