// Fixture crate: a leftover `assert_boundary!` probe referencing a seam `ghost` that no
// `RuntimeBoundary` declares (a boundary deleted, its probe left behind). The runtime CI audit
// must react to this undeclared seam even though the constitution declares no runtime boundary —
// the case the shell previously skipped. Scanned lexically by the audit; need not compile.
pub fn crossing() {
    assert_boundary!("ghost", ());
}
