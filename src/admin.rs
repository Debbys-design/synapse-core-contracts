//! Two-step admin transfer per `docs/adr/0002-two-step-admin-transfer.md`.
//!
//! A direct, single-call admin transfer is a known footgun: a typo'd address
//! permanently locks the contract. The two-step flow requires the proposed
//! admin to explicitly accept before the transfer is finalized.

use soroban_sdk::{Address, Env};

use crate::events;
use crate::storage;

/// Propose a new admin. Only the current admin may call this.
///
/// Stores `new_admin` as the pending admin, overwriting any prior pending
/// proposal (proposals do not stack). The transfer is not finalized until
/// `accept_admin` is called by the pending admin.
///
/// # Panics
/// Panics if the current admin has not authorized the call.
pub fn propose_admin(env: &Env, new_admin: Address) {
    let current = storage::get_admin(env);
    current.require_auth();
    storage::set_pending_admin(env, &new_admin);
}

/// Accept a pending admin transfer. Only the stored pending admin may call this.
///
/// Finalizes the transfer by promoting the pending admin to admin and clearing
/// the pending slot. Emits the `admin` topic event on finalization.
///
/// # Panics
/// Panics if there is no pending admin or if `caller` is not the pending admin.
/// State is not mutated on rejection.
pub fn accept_admin(env: &Env, caller: Address) {
    caller.require_auth();
    let pending = storage::get_pending_admin(env).expect("no pending admin");
    if caller != pending {
        panic!("caller is not the pending admin");
    }
    storage::set_admin(env, &pending);
    storage::clear_pending_admin(env);
    events::emit_admin(env, &pending);
}

/// Return the current admin address.
pub fn admin(env: &Env) -> Address {
    storage::get_admin(env)
}

/// Return the pending admin address, if a transfer is in progress.
pub fn pending_admin(env: &Env) -> Option<Address> {
    storage::get_pending_admin(env)
}

/// Rotate the single relay signer immediately (no timelock).
///
/// Admin-gated baseline rotation path for the single-signer trust model in
/// `docs/adr/0001-relay-signer-trust-model.md`. The new signer takes effect
/// immediately: `relay_signer()` returns it and `register_callback`
/// authenticates against it right away. Rotating to the currently-active
/// signer is a no-op on state but still succeeds and still emits the event.
///
/// # Panics
/// Panics if the current admin has not authorized the call.
pub fn set_relay_signer(env: &Env, new_signer: Address) {
    let current_admin = storage::get_admin(env);
    current_admin.require_auth();
    let old_signer = storage::get_relay_signer(env);
    storage::set_relay_signer(env, &new_signer);
    events::emit_relay_signer_rotated(env, &old_signer, &new_signer);
}
