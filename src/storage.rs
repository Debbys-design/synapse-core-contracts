use soroban_sdk::{contracttype, Address, Env};

/// Storage keys used by the contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The current admin address.
    Admin,
    /// A proposed admin awaiting acceptance (two-step transfer, ADR-0002).
    PendingAdmin,
}

/// Returns the current admin, if one has been set.
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

/// Persists the current admin address.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Returns the pending admin proposal, if any.
pub fn get_pending_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::PendingAdmin)
}

/// Stores a pending admin proposal, overwriting any prior proposal.
pub fn set_pending_admin(env: &Env, pending: &Address) {
    env.storage().instance().set(&DataKey::PendingAdmin, pending);
}

/// Clears any pending admin proposal.
pub fn clear_pending_admin(env: &Env) {
    env.storage().instance().remove(&DataKey::PendingAdmin);
}
