//! `install` fails loud on a duplicate declaration rather than silently overwriting it (a silent
//! overwrite would let the last declaration shadow an earlier law — a declared boundary that never
//! enforces, the one bug Tianheng forbids). Its own test binary because the duplicate check fires
//! *before* the write-once `REGISTRY.set`, so these panics never set the process-global registry.

struct Thing;

#[test]
#[should_panic(expected = "declared more than once")]
fn a_duplicate_seam_fails_loud() {
    louke::install(
        [
            louke::RuntimeBoundary::at("dup-seam")
                .only_origins(["app::a"])
                .because("first"),
            louke::RuntimeBoundary::at("dup-seam")
                .only_origins(["app::b"])
                .because("second — would silently shadow the first"),
        ],
        std::iter::empty::<louke::OriginEntry>(),
    );
}

#[test]
#[should_panic(expected = "registered more than once")]
fn a_duplicate_origin_fails_loud() {
    louke::install(
        std::iter::empty::<louke::RuntimeBoundary>(),
        [
            louke::register_origin!(Thing),
            louke::register_origin!(Thing),
        ],
    );
}
