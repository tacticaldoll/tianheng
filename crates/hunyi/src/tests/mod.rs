mod async_exposure;
mod dyn_trait;
mod finding_source_file;
mod forbidden_marker;
mod helpers;
mod impl_trait;
mod macro_and_body_nested;
mod resolver_fidelity;
mod signature;
mod trait_impl;
mod unsafe_confinement;
mod visibility;

#[test]
fn empty_composition_is_clean_without_reading_a_manifest() {
    let absent = std::env::temp_dir().join(format!(
        "tianheng-empty-semantic-composition-{}-does-not-exist/Cargo.toml",
        std::process::id()
    ));

    assert!(matches!(
        crate::check_all(&crate::SemanticBoundaries::default(), &absent),
        xuanji::Outcome::Clean(_)
    ));
}
