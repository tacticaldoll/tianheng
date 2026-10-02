use crate::hermetic_git::scratch_ceiling;

/// The ceiling is the directory the fixture helper places roots under — two statements of one layout rule,
/// held equal.
///
/// `kanhe`'s normal edges may not reach `xingbiao`, so `scratch_ceiling` restates the layout rule and this is
/// the one place that names both. It stands alone in its file so the fixture-root check's declaration for
/// it covers this direction and no other.
#[test]
fn ceiling_is_the_directory_fixture_roots_live_under() {
    assert_eq!(scratch_ceiling(), xingbiao::scratch_base());
}
