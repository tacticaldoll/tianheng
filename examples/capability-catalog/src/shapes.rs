//! Deliberate public type-shape faults.

/// A port used only to put a trait object on the public surface.
pub trait Port {}

/// Exposes a written `dyn Trait` shape.
pub fn dyn_port() -> Box<dyn Port> {
    panic!("contract fixture is never executed")
}

/// Exposes a written return-position `impl Trait` shape.
pub fn numbers() -> impl Iterator<Item = u8> {
    [1].into_iter()
}

/// Exposes the implicit, compiler-inserted `impl Future` existential — the other signal
/// `no_existential_leak` composes alongside the written-`impl Trait` one above.
pub async fn fetch() -> u8 {
    0
}
