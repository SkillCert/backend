#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec,
};

#[contract]
pub struct RevokedRegistry;

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct revoked_certificate {
    pub certificate_id: u64,
    pub revoked_by: Address,
    pub reason: Bytes,
    pub revoked_at: u64,
}

pub const ADMIN: Symbol = symbol_short!("admin");
pub const REVOKED_LIST: Symbol = symbol_short!("revoked");

#[contractimpl]
impl RevokedRegistry {
    pub fn Set_admin(env: Env, admin: Address) {
        let stored_admin: Option<Address> = env.storage().instance().get(&ADMIN);
        if stored_admin.is_none() {
            env.storage().instance().set(&ADMIN, &admin);
        } else {
            panic!("admin_already_set");
        }
    }
    pub fn Revoke_Certificate(env: Env, caller: Address, certificate_id: u64, reason: Bytes) {
        let admin: Address = env.storage().instance().get(&ADMIN).expect("admin not set");
        if caller != admin {
            panic!("only_admin_can_revoke_certificates");
        }

        let revoked_at = env.ledger().timestamp();

        let new_revoked_certificate = revoked_certificate {
            certificate_id,
            revoked_by: caller.clone(),
            reason,
            revoked_at,
        };

        let mut revoked_list: Vec<revoked_certificate> =
            match env.storage().instance().get(&REVOKED_LIST) {
                Some(x) => x,
                None => Vec::new(&env),
            };

        revoked_list.push_back(new_revoked_certificate.clone());

        env.storage().instance().set(&REVOKED_LIST, &revoked_list);

        env.events()
            .publish(("revoked_certificate", caller), new_revoked_certificate);
    }

    pub fn is_revoked(env: Env, certificate_id: u64) -> bool {
        let revoked_list: Vec<revoked_certificate> =
            match env.storage().instance().get(&REVOKED_LIST) {
                Some(x) => x,
                None => Vec::new(&env),
            };

        for x in revoked_list.iter() {
            if x.certificate_id == certificate_id {
                return true;
            }
        }

        false
    }

    pub fn All_Revoked_Certificates(env: Env) -> Vec<revoked_certificate> {
        match env.storage().instance().get(&REVOKED_LIST) {
            Some(x) => x,
            None => Vec::new(&env),
        }
    }

    pub fn Get_Revocation_Details(env: Env, certificate_id: u64) -> Option<(Address, Bytes, u64)> {
        let revoked_list: Vec<revoked_certificate> =
            match env.storage().instance().get(&REVOKED_LIST) {
                Some(x) => x,
                None => Vec::new(&env),
            };

        for x in revoked_list.iter() {
            if x.certificate_id == certificate_id {
                return Some((x.revoked_by, x.reason, x.revoked_at));
            }
        }
        None
    }
}
