//! The api layer — and a deliberate reach into `infra`, which 圭表's module boundary reacts to.
use crate::infra::Store;

/// Reads through the infra type directly, which the declared boundary forbids.
pub fn lookup(store: &Store, key: &str) -> Option<String> {
    store.get(key)
}
