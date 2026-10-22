//! Storage helpers for the Soroban contract.
//!
//! All persistent, temporary, and instance ledger entries are addressed via
//! the plain (non-namespaced) [`StorageKey`] enum. Namespaced storage keys
//! were explicitly reverted per `QUARANTINE.md`; do not reintroduce them.

use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env};

use crate::errors::ContractError;

/// Plain (non-namespaced) storage key addressing every distinct piece of
/// contract state across the persistent, temporary, and instance tiers.
///
/// Associated data is kept minimal and copy-cheap because a `StorageKey` is
/// constructed on every storage access.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    /// Contract admin address (instance).
    Admin,
    /// Authorized relay signer address (instance).
    RelaySigner,
    /// Global pause flag (instance).
    Paused,
    /// Default per-transaction ceiling (instance).
    DefaultCeiling,
    /// Per-transaction record, keyed by transaction id (persistent).
    Transaction(u64),
    /// Status-index slot for a given status (persistent).
    StatusIndexSlot(u32),
    /// Count of entries in the status index (persistent).
    StatusIndexCount,
    /// Idempotency key guarding against duplicate submissions (temporary).
    IdempotencyKey(BytesN<32>),
    /// Relay approval record for a transaction (temporary).
    RelayApproval(u64),
    /// Param-registry entry, keyed by parameter name (persistent).
    ParamRegistryEntry(Bytes),
    /// Bond record for an address (persistent).
    BondRecord(Address),
    /// Treasury balance (persistent).
    TreasuryBalance,
    /// Treasury configuration (instance).
    TreasuryConfig,
    /// Upgrade history (persistent).
    UpgradeHistory,
    /// Pending upgrade awaiting execution (instance).
    PendingUpgrade,
    /// Schema compatibility range (instance).
    SchemaCompatRange,
}

/// Returns `true` when the given key is stored in the instance tier.
pub fn is_instance_key(key: &StorageKey) -> bool {
    matches!(
        key,
        StorageKey::Admin
            | StorageKey::RelaySigner
            | StorageKey::Paused
            | StorageKey::DefaultCeiling
            | StorageKey::TreasuryConfig
            | StorageKey::PendingUpgrade
            | StorageKey::SchemaCompatRange
    )
}

/// Returns `true` when the given key is stored in the temporary tier.
pub fn is_temporary_key(key: &StorageKey) -> bool {
    matches!(
        key,
        StorageKey::IdempotencyKey(_) | StorageKey::RelayApproval(_)
    )
}

/// Reads a value from instance storage for the given key.
pub fn get_instance<T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &StorageKey,
) -> Option<T> {
    env.storage().instance().get(key)
}

/// Writes a value to instance storage for the given key.
pub fn set_instance<T: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &StorageKey,
    value: &T,
) {
    env.storage().instance().set(key, value);
}

/// Reads a value from persistent storage for the given key.
pub fn get_persistent<T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &StorageKey,
) -> Option<T> {
    env.storage().persistent().get(key)
}

/// Writes a value to persistent storage for the given key.
pub fn set_persistent<T: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &StorageKey,
    value: &T,
) {
    env.storage().persistent().set(key, value);
}

/// Reads a value from temporary storage for the given key.
pub fn get_temporary<T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &StorageKey,
) -> Option<T> {
    env.storage().temporary().get(key)
}

/// Writes a value to temporary storage for the given key.
pub fn set_temporary<T: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &StorageKey,
    value: &T,
) {
    env.storage().temporary().set(key, value);
}

/// Persist the active relay signer address in instance storage.
pub fn set_relay_signer(env: &Env, signer: &Address) {
    env.storage()
        .instance()
        .set(&StorageKey::RelaySigner, signer);
}

/// Read the active relay signer address from instance storage.
///
/// Returns a clean [`ContractError::NotInitialized`] when the contract has not
/// been initialized yet, rather than panicking on a missing storage key.
pub fn get_relay_signer(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&StorageKey::RelaySigner)
        .ok_or(ContractError::NotInitialized)
}

/// Persist the paused flag in instance storage.
pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&StorageKey::Paused, &paused);
}

/// Read the paused flag from instance storage.
///
/// Defaults to `false` when the flag has never been written, so a freshly
/// initialized contract is treated as unpaused.
pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&StorageKey::Paused)
        .unwrap_or(false)
}

/// Shared circuit-breaker guard.
///
/// Rejects with [`ContractError::ContractPaused`] when the contract is paused.
/// Intended to be called first by every state-mutating entry point so that
/// read-only queries remain available while paused.
pub fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    if is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}
}
