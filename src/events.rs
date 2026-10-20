//! Contract event definitions.
//!
//! Events are emitted via `env.events().publish(...)` and are keyed by a
//! topic tuple. The schemas here mirror `EVENTS.md`; keep them in sync.

use soroban_sdk::{contracttype, Address, Env};

/// Topic symbol used for relay-related events (matches the "relay" topic in
/// `EVENTS.md`).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRelaySignerRotated {
    /// The relay signer that was active before the rotation.
    pub old_signer: Address,
    /// The relay signer that is active after the rotation.
    pub new_signer: Address,
}

/// Emit an [`EventRelaySignerRotated`] event under the `relay` topic.
///
/// This is emitted on every successful `set_relay_signer` call, including
/// no-op rotations to the currently-active signer.
pub fn emit_relay_signer_rotated(env: &Env, old_signer: Address, new_signer: Address) {
    let event = EventRelaySignerRotated {
        old_signer,
        new_signer,
    };
    env.events().publish((soroban_sdk::symbol_short!("relay"),), event);
}
