//! The constitution the adopter writes — the imitable surface (潛移). One unsafe-confinement
//! boundary: `unsafe` may live only under `crate::ffi`.
use hunyi::UnsafeBoundary;

/// The declared law: all `unsafe` in this crate is confined to the `crate::ffi` subtree.
pub fn constitution() -> Vec<UnsafeBoundary> {
    vec![UnsafeBoundary::in_crate("unsafe_confinement")
        .only_under(["crate::ffi"])
        .because("unsafe lives only behind the ffi module — everywhere else is safe by contract")]
}
