use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// The supertrait a governed trait carries so a probe can recover the concrete type behind
/// a `dyn Trait` without trait upcasting (rust 1.85). Write `trait DomainPort: louke::Tracked`;
/// the blanket impl supplies `as_any` for every `'static` type, so no per-type boilerplate is
/// needed. A `&dyn Trait` over **borrowed** (non-`'static`) data cannot be probed — `Any`
/// requires `'static` (a stated bound).
pub trait Tracked: Any {
    /// Recover `&dyn Any` (and thus the concrete `TypeId`) from a trait object.
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> Tracked for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// --- The fold-hasher: TypeId is already a hash; never SipHash ----------------

/// A std-only hasher that folds the written bytes. A `TypeId` is already a good hash, so
/// folding avoids SipHash's per-lookup cost (the overhead-spike's only trap). `write` is
/// implemented for the general byte path (not only `write_u64`/`write_u128`), since a
/// `TypeId`'s hash may route through either across toolchains.
#[derive(Default)]
pub(crate) struct FoldHasher(u64);

impl Hasher for FoldHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0.rotate_left(8) ^ b as u64;
        }
    }
    fn write_u64(&mut self, i: u64) {
        self.0 ^= i;
    }
    fn write_u128(&mut self, i: u128) {
        self.0 ^= i as u64 ^ (i >> 64) as u64;
    }
}

pub(crate) type TidMap<V> = HashMap<TypeId, V, BuildHasherDefault<FoldHasher>>;
