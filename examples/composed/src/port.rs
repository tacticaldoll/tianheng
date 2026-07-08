//! The port — a `dyn`-dispatched seam between the app and its adapters. Carrying the
//! `louke::Tracked` supertrait lets a runtime probe recover the concrete type behind a
//! `dyn Adapter` (so 漏刻 can read *which* adapter actually crossed the seam).

/// The architectural seam. Every adapter implements it; a runtime boundary governs which
/// concrete origins may cross.
pub trait Adapter: louke::Tracked {
    /// Handle a request at the seam.
    fn handle(&self);
}
