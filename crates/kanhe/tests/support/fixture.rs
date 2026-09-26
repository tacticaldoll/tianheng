//! The controlled fixture the wrapper directions run under, written once for both wrappers.
//!
//! The merge and publish directions each held their own copy of this: the scratch root, the executables, the
//! `PATH` that finds them first, and the block a controlled gate writes its verdict with. The copies agreed by
//! maintenance and had already drifted — the verdict block's comment was rewritten in one and not the other.
//!
//! **What the two controlled `cargo` stubs share is here; what they log is each direction's own.** Both splice the
//! verdict block and the pass line, which the wrappers' guards read, from this module. Each writes its own log
//! line because each harness reads a different record of the run — the merge directions the commits it was
//! handed, the publish directions the environment the act inherited — and a shared line would carry fields one
//! side never reads.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A scratch root claimed through `xingbiao::claim_scratch`, so it cannot adopt a path planted before it, and
/// removed on drop, so an assertion failing between its creation and the end of the run leaves nothing behind.
pub struct Scratch(PathBuf);

impl Scratch {
    /// A fresh root under the system temporary directory, named `<prefix>-<pid>-<n>`.
    pub fn claim(prefix: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        loop {
            let candidate = std::env::temp_dir().join(format!(
                "{prefix}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match xingbiao::claim_scratch(&candidate) {
                Ok(()) => return Self(candidate),
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(err) => panic!(
                    "cannot acquire a scratch root {}: {err}",
                    candidate.display()
                ),
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

/// A root that cannot be removed is settled by `xingbiao::settle_cleanup`, as the other guards calling it settle
/// theirs.
impl Drop for Scratch {
    fn drop(&mut self) {
        xingbiao::settle_cleanup(
            "Scratch: removing",
            &self.0,
            std::fs::remove_dir_all(&self.0),
        );
    }
}

/// The directories a wrapper run is staged in: a claimed scratch root, the `bin` its stubs are found in first,
/// and the wrapper's own `TMPDIR`, so what it leaves behind is observable and lands in the fixture rather than in
/// the developer's `/tmp`.
pub struct Harness {
    scratch: Scratch,
    bin: PathBuf,
    tmp: PathBuf,
}

impl Harness {
    /// A fresh scratch root with its `bin` and `tmp` made.
    pub fn claim(prefix: &str) -> Self {
        let scratch = Scratch::claim(prefix);
        let bin = scratch.path().join("bin");
        std::fs::create_dir(&bin).expect("create controlled PATH");
        let tmp = scratch.path().join("tmp");
        std::fs::create_dir(&tmp).expect("create the wrapper's temporary directory");
        Self { scratch, bin, tmp }
    }

    pub fn path(&self) -> &Path {
        self.scratch.path()
    }

    pub fn bin(&self) -> &Path {
        &self.bin
    }

    pub fn tmp(&self) -> &Path {
        &self.tmp
    }

    /// The names of what the run left in its `TMPDIR`.
    pub fn leftover(&self) -> Vec<String> {
        std::fs::read_dir(&self.tmp)
            .expect("read the wrapper's temporary directory")
            .map(|entry| {
                entry
                    .expect("a temporary directory entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }
}

/// Stop before the subject when a host tool a fixture's stub runs is absent, naming the tool.
///
/// A stub that pipes through a tool the host lacks fails inside the wrapper it stands behind, and the wrapper then
/// reports the state it met — correctly — as a fact about its subject. So a direction whose stub needs one says so
/// first, and an absent tool reads as itself.
pub fn require_host_tool(tool: &str) {
    let found = crate::support::bash::bash()
        .args(["-c", r#"command -v "$1" >/dev/null"#, "probe", tool])
        .status()
        .expect("bash runs");
    assert!(
        found.success(),
        "this fixture's stub runs `{tool}`, which is not on PATH — install it, since its absence would read as \
         findings about the wrapper under test"
    );
}

/// Write `text` to `path` and make it executable.
pub fn write_executable(path: &Path, text: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, text).expect("write controlled executable");
    let mut permissions = std::fs::metadata(path)
        .expect("read controlled executable metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("make controlled executable runnable");
}

/// The file's text, or empty where it was never written.
pub fn read_if_present(path: &Path) -> std::io::Result<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(err) => Err(err),
    }
}

/// A `PATH` that finds the executables in `bin` before anything on the host's.
pub fn path_with(bin: &Path) -> String {
    let host = std::env::var_os("PATH").unwrap_or_default();
    format!("{}:{}", bin.display(), host.to_string_lossy())
}

/// The summary a controlled gate's `cargo test` prints for one passing test, which each wrapper's
/// `require_one_pass` reads — held against that function's pattern by
/// `the_controlled_gate_s_pass_line_is_the_one_the_wrappers_require`, so the stub and the guard cannot drift.
pub const GATE_PASS_LINE: &str =
    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";

/// The line a controlled `cargo` prints [`GATE_PASS_LINE`] with, spliced into each wrapper's stub.
pub fn gate_pass_stub() -> String {
    format!("printf '%s\\n' '{GATE_PASS_LINE}'\n")
}

/// The block a controlled `cargo` runs to report the gate's verdict, spliced into each wrapper's stub.
pub const GATE_VERDICT_STUB: &str = r##"# The gate reports on the channel whether it agrees or refuses, so a controlled gate that only prints
# `1 passed` is a gate that ran and judged nothing — which is what the wrapper's success path refuses.
# `no-verdict` is the mode that keeps that state constructible.
#
# Only where the channel was opened: the wrapper hands the channel to the gate alone, so a run of this
# executable without it — the act, where the same executable stands in for the tool — is not the gate's run,
# and an unguarded write would fail under `set -u` there and record a verdict no gate reached.
if [[ ${FAKE_GATE_VERDICT-} != none && -n ${TIANHENG_GATE_VERDICT-} ]]; then
    printf '%s' 'Clean' > "$TIANHENG_GATE_VERDICT"
fi
"##;
