//! Contract entry points for the two-step admin transfer flow.
//!
//! See `docs/adr/0002-two-step-admin-transfer.md` for the state machine.

use soroban_sdk::{contractimpl, Address, Env};

use crate::admin::{self, AdminError};
use crate::events;
use crate::storage;

/// Contract type exposing the admin-transfer entry points.
pub struct Contract;

#[contractimpl]
impl Contract {
    /// Propose a new admin. Only the current admin may call this.
    ///
    /// Overwrites any previously pending proposal rather than stacking.
    pub fn propose_admin(env: Env, new_admin: Address) -> Result<(), AdminError> {
        let current = storage::get_admin(&env).ok_or(AdminError::NotInitialized)?;
        current.require_auth();
        storage::set_pending_admin(&env, &new_admin);
        Ok(())
    }

    /// Accept a pending admin transfer. Only the stored `pending_admin` may call this.
    ///
    /// Finalizes the transfer, clears `pending_admin`, and emits the `admin` event.
    pub fn accept_admin(env: Env, caller: Address) -> Result<(), AdminError> {
        caller.require_auth();
        let pending = storage::get_pending_admin(&env).ok_or(AdminError::NoPendingAdmin)?;
        if caller != pending {
            return Err(AdminError::NotPendingAdmin);
        }
        storage::set_admin(&env, &pending);
        storage::clear_pending_admin(&env);
        events::admin_transferred(&env, &pending);
        Ok(())
    }

    /// Read-only query returning the current admin, if set.
    pub fn admin(env: Env) -> Option<Address> {
        storage::get_admin(&env)
    }

    /// Read-only query returning the pending admin proposal, if any.
    pub fn pending_admin(env: Env) -> Option<Address> {
        storage::get_pending_admin(&env)
    }
}

// Re-export the admin error type for callers.
pub use admin::AdminError as ContractError;
