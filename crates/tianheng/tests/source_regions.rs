mod support;

use support::region::Source;

#[test]
fn executed_regions_respect_the_source_language() {
    let rust = Source::of("#[cfg(test)]\n// hidden\nfn guarded() {}\n");
    assert!(rust.rust().contains("#[cfg(test)]"));
    assert!(!rust.rust().contains("hidden"));

    let shell = Source::of("# hidden\nprintf '%s\\n' '// data'\n");
    assert!(!shell.shell().contains("hidden"));
    assert!(shell.shell().contains("// data"));
}
