    /// Returns the live schema version of the contract's storage layout.
    ///
    /// This is a cheap query backed by [`SCHEMA_VERSION`]; it is not derived
    /// from WASM metadata.
    pub fn schema_version(_env: Env) -> u32 {
        SCHEMA_VERSION
    }

    /// Upgrades the contract WASM to `new_wasm_hash`.
    ///
    /// Admin-gated via `require_auth()`. The caller's `expected_schema_version`
    /// must match the l
pub struct Contract;

#[contractimpl]
impl Contract {
    /// Returns the live schema version of the contract's storage layout.
    ///
    /// This is a cheap query backed by [`SCHEMA_VERSION`]; it is not derived
    /// from WASM metadata.
    pub fn schema_version(_env: Env) -> u32 {
        SCHEMA_VERSION
    }

    /// Return the current contract semver.
    ///
    /// Stable across calls within a single deployment and independent from
    /// `schema_version()`. Bumped on schema changes per `EVENTS.md`.
    pub fn version(_env: Env) -> u32 {
        CONTRACT_VERSION
    }

    /// Minimal liveness / self-consistency check.
    ///
    /// Returns `true` only when both the admin and relay signer are set. Safe to
    /// call before `initialize()`: it reports `false` rather than panicking.
    pub fn health(env: Env) -> bool {
        storage::get_admin(&env).is_some() && storage::get_relay_signer(&env).is_some()
    }

    /// Upgrades the contract WASM to `new_wasm_hash`.
    ///
    /// Admin-gated via `require_auth()`. The caller's `expected_schema_version`
    /// must match the live [`schema_version`](Self::schema_version); otherwise
    /// the call is rejected with [`ContractError::SchemaVersionMismatch`]
    /// *before* `env.deployer().update_current_contract_wasm()` is invoked.
    ///
    /// On success, `EventContractUpgraded` ("upgrade" topic) is emitted.
    /// Persistent and instance storage survive the upgrade; only temporary
    /// storage (idempotency keys) may be evicted.
    pub fn upgrade(
        env: Env,
        new_wasm_hash: BytesN<32>,
        expected_schema_version: u32,
    ) -> Result<(), ContractError> {
        // Cheapest-first ordering: authenticate the admin before any other work.
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(ContractError::Unauthorized)?;
        admin.require_auth();

        // Schema guard: reject mismatches before touching WASM.
        if expected_schema_version != SCHEMA_VERSION {
            return Err(ContractError::SchemaVersionMismatch);
        }

        // Only now is it safe to swap the contract code.
        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());

        env.events().publish(
            (UPGRADE_TOPIC,),
            EventContractUpgraded {
                new_wasm_hash,
                schema_version: SCHEMA_VERSION,
            },
        );

        Ok(())
    }

    /// Emergency admin revocation, callable only by the guardian.
    ///
    /// Per `docs/adr/0004-guardian-emergency-admin-revocation.md`, the guardian
    /// may revoke a compromised admin and replace it with `new_admin`. This
    /// entry point is gated so it can *only* revoke/replace the admin — it
    /// cannot grant the guardian any other admin powers. The guardian's
    /// `require_auth()` is enforced before any storage mutation.
    ///
    /// On success the stored admin is replaced immediately (so every
    /// admin-gated entry point, e.g. [`upgrade`](Self::upgrade), respects the
    /// new admin on the very next call) and `EventGuardianAdminRevoked` is
    /// emitted under the distinct `guardian_revoke` topic.
    pub fn guardian_revoke_admin(
        env: Env,
        new_admin: Address,
    ) -> Result<(), ContractError> {
        // Authenticate the guardian before any other work.
        let guardian: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Guardian)
            .ok_or(ContractError::Unauthorized)?;
        guardian.require_auth();

        // Capture the outgoing admin for the event (may be absent if already
        // revoked); the guardian still may install a replacement.
        let old_admin: Option<Address> = env.storage().instance().get(&StorageKey::Admin);

        // Revoke/replace only: overwrite the admin slot. No other admin powers
        // are granted to the guardian by this entry point.
        env.storage()
            .instance()
            .set(&StorageKey::Admin, &new_admin);

        env.events().publish(
            (GUARDIAN_REVOKE_TOPIC,),
            EventGuardianAdminRevoked {
                old_admin,
                new_admin,
            },
        );

        Ok(())
    }
}

/// Compile-time guard that `ContractError` stays within Soroban's 50-variant cap.
///
/// `ContractError::__VARIANT_COUNT` is a `const` emitted alongside the enum in
/// `src/types.rs`; if a future PR adds a variant past the ceiling this module
/// fails to compile, so the cap cannot silently regress.
const _: () = assert!(
    crate::types::ContractError::__VARIANT_COUNT <= MAX_CONTRACT_ERROR_VARIANTS,
    "ContractError exceeds Soroban's 50-variant cap; see DECISIONS.md 'Error-code budget'"
);
