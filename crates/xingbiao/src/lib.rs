//! 星表 (xīngbiǎo) — the single reader of truth for **what the workspace tree IS**.
//!
//! Reads `cargo metadata --no-deps` and looks up packages and their crate-root source files:
//! the tabulated catalog every observation dimension references before it observes. Spawns
//! `cargo` and parses its JSON (`serde_json` + std only, no `syn`).
//!
//! **The criterion, because this charter widens, and an enumeration of what it holds goes
//! stale.** A fact belongs here when every dimension must agree on it *before* observing, and it is a
//! fact about the tree rather than a reading of what the tree contains. Which packages exist and where
//! their roots are; whether two paths are one file; whether a path is there at all, and whether being
//! unable to tell is the same answer as its absence. Each of those is asked by 圭表, 渾儀 and 漏刻
//! alike, and a dimension answering it privately is how three readers came to disagree about one tree.
//!
//! **What the criterion refuses** is as load-bearing as what it admits: an interpretation of source is a
//! dimension's own observation, not a fact about the tree. `cargo metadata` says which file is a crate
//! root; what the tokens in that file mean is 圭表's or 渾儀's answer and belongs to whichever is
//! asking. The line is between *what is there* and *what it says*.
//!
//! The widenings are named rather than left to be inferred: the path-identity primitives
//! ([`canonicalize_or_fail`], [`try_visit`]) arrived for a module-graph cycle guard, and the
//! filesystem-answer policy ([`is_regular_file`], [`is_directory`] and the crate-private criterion behind
//! them) arrived after all three dimensions were measured collapsing *absent* into *unreadable* — each is
//! one notch finer than the last, on the same
//! question of what the tree holds.
//!
//! **Fixture infrastructure.** This widening is test-only: [`claim_scratch`] and `Unreadable`.
//! Both are `#[doc(hidden)]` items that exist because integration and unit test trees across member
//! crates cannot share a `cfg(test)` helper, yet must enforce identical policies (atomic temporary
//! root reservation without adopting pre-existing symlinks, and permission-removal verification that
//! restores on drop and asserts under `TIANHENG_WORKSPACE_TESTS`). They ship hidden in the published
//! crate so all test suites agree on fixture safety and refusal semantics without hand-rolled copies.
//!
//! Sits beneath all three observation dimensions — static (圭表), semantic (渾儀) and runtime (漏刻,
//! through its CI-only `audit` face) — preventing twin-drift in what they take the tree to be.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

mod path_identity;

#[cfg(test)]
mod tests;

pub use path_identity::{canonicalize_or_fail, try_visit};

/// Target `kind` strings that denote a library crate root (library types + `proc-macro`).
const LIBRARY_KINDS: [&str; 6] = ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"];

/// Run `cargo metadata --no-deps --format-version 1` for the workspace at `manifest_path`.
pub fn cargo_metadata(manifest_path: &Path) -> Result<Value, String> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest_path)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("cargo metadata failed: {}", output.status)
        } else {
            stderr
        });
    }
    serde_json::from_slice(&output.stdout).map_err(|err| err.to_string())
}

/// Find a workspace member package by name in parsed metadata.
pub fn find_package<'a>(metadata: &'a Value, package: &str) -> Option<&'a Value> {
    metadata["packages"]
        .as_array()?
        .iter()
        .find(|candidate| candidate["name"].as_str() == Some(package))
}

/// Whether a `cargo metadata` target's `kind` array contains `wanted` — the one-target shape
/// check shared by [`crate_root_file`] (picking one library/bin target) and [`crate_roots`]
/// (every library/bin target of one package), through which [`member_root_files`] reaches it for the
/// whole workspace.
fn target_has_kind(target: &Value, wanted: &str) -> bool {
    target["kind"]
        .as_array()
        .is_some_and(|kinds| kinds.iter().any(|k| k.as_str() == Some(wanted)))
}

/// Which of a package's crate roots are compiled, told apart from the two ways there can be none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrateRoots {
    /// The metadata reports no target — the shape synthetic metadata in a caller's own tests carries, for which a
    /// caller reads the conventional source directory.
    Unreported,
    /// It reports targets, none a library kind or a binary — an example, a test, a bench or a build script alone —
    /// so no compiled root reads the package's `src/`, and reading it would judge a directory nothing builds.
    NoneCompiled,
    /// At least one library-kind or binary root.
    Compiled(CompiledRoots),
}

/// A package's library-kind and binary roots, in Cargo's reported order, each once, and at least one — by
/// construction, since only [`crate_roots`] builds one, so an empty list cannot stand for either state that has
/// none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledRoots(Vec<PathBuf>);

impl CompiledRoots {
    /// The roots, in Cargo's reported order.
    pub fn as_slice(&self) -> &[PathBuf] {
        &self.0
    }
}

impl CrateRoots {
    /// The compiled roots, and none for either state that has none.
    pub fn compiled(&self) -> &[PathBuf] {
        match self {
            Self::Compiled(roots) => roots.as_slice(),
            Self::Unreported | Self::NoneCompiled => &[],
        }
    }
}

/// What ONE package's metadata says about its compiled crate roots: **every** one — each library-kind target
/// and each `bin` target, in Cargo's reported order, deduplicated.
///
/// The per-package counterpart of [`member_root_files`] (which spans the workspace) and the plural of
/// [`crate_root_file`] (which picks one). A package's roots are separate compilation units: they each
/// denote the module path `crate` and neither's declarations belong in the other's module graph, so a
/// dimension that governs a package governs each root as its own corpus. Returning them all is what lets
/// a violation written in a `bin` beside a library be observed at all.
///
/// The uniqueness is **total**, not adjacency-dependent, and that distinction is load-bearing here in a
/// way it is not in [`member_root_files`] (which sorts first, so `Vec::dedup` is total for it by
/// construction). Two targets may name the same `path` — Cargo accepts it and builds both — and Cargo
/// reports targets sorted by NAME, so the two reports are adjacent only if no third target's name sorts
/// between them. `Vec::dedup` alone therefore left `[x, y, x]` intact, and the root was scanned once per
/// report. Order is Cargo's own and is preserved, because a caller's sibling-root exclusion reads this
/// slice positionally against the root it is currently walking.
///
/// A target Cargo reports always carries its `src_path`, so a library or binary target without one is metadata
/// no Cargo wrote; it contributes no root, the answer synthetic metadata that omits the field would expect.
pub fn crate_roots(package: &Value) -> CrateRoots {
    let targets = match package["targets"].as_array() {
        Some(targets) if !targets.is_empty() => targets,
        _ => return CrateRoots::Unreported,
    };
    let mut seen = std::collections::HashSet::new();
    let roots: Vec<PathBuf> = targets
        .iter()
        .filter(|t| {
            LIBRARY_KINDS.iter().any(|k| target_has_kind(t, k)) || target_has_kind(t, "bin")
        })
        .filter_map(|t| t["src_path"].as_str().map(PathBuf::from))
        .filter(|root| seen.insert(root.clone()))
        .collect();
    if roots.is_empty() {
        CrateRoots::NoneCompiled
    } else {
        CrateRoots::Compiled(CompiledRoots(roots))
    }
}

/// A path as a **canonical identity label**: `/` as its only component separator, and every byte the
/// path carried preserved.
///
/// The one answer to "what is the label for an observed path", shared by every dimension that records
/// one — 圭表/渾儀's compilation unit ([`compilation_unit_label`]) and 漏刻's observed file. Written once
/// here because two sites answering separately is how they came to disagree.
///
/// **Separator.** Built on [`Path::components`], joined with `/`, and that is the whole reason it is
/// correct rather than an implementation detail: separator semantics are delegated to `std::path`
/// instead of re-implemented. Substituting characters would be wrong in both directions — on unix `\`
/// is a legal byte *within* a name, so replacing it would map the single file `a\b` and the file `b`
/// inside directory `a` onto one label, destroying the injectivity this exists for; on Windows both
/// `\` and `/` separate, so replacing one is incomplete. `components()` is the only thing that knows
/// which is which per platform. A component cannot contain `/` anywhere, so `/` in a label
/// unambiguously means a component boundary.
///
/// Canonical path separators ensure cross-platform baseline stability: without this, paths would
/// produce `src/lib.rs` on Linux and `src\lib.rs` on Windows, causing a baseline recorded on one
/// platform to match nothing on the other.
///
/// **Bytes.** Every byte that is not part of a valid UTF-8 sequence is percent-escaped, and a literal
/// `%` becomes `%25` so no escaped label can be spelled by an unescaped one. `Path::display()` is
/// **lossy** — it replaces each undecodable byte with U+FFFD — so two paths differing only in such
/// bytes would produce one label, one identity, and a baseline accepting the first would silently
/// suppress the second's never-accepted violation. This half is load-bearing for 漏刻, whose labels come
/// from filesystem walks where such a name is reachable; for 圭表/渾儀 it cannot trigger, since their
/// paths are built from `cargo metadata`'s JSON strings and Cargo refuses to operate under a non-UTF-8
/// path at all (`error: path contains invalid UTF-8 characters`). Holding one rule for both is what
/// keeps the dimension where it cannot trigger from drifting away from the one where it can.
///
/// `as_encoded_bytes`'s encoding is unspecified but self-consistent within a platform, which is all this
/// needs: a label is never decoded back, only compared with another label produced the same way. On
/// Windows that encoding is WTF-8, so an unpaired surrogate's bytes escape exactly as an invalid unix
/// byte does.
///
/// **Stated normalizations.** `Component::CurDir` contributes nothing and repeated separators collapse,
/// so `./a` and `a//b` label as `a` and `a/b`. Both name the same file, and neither form is reachable
/// from the inputs these labels are built from — a `cargo metadata` `src_path` or a walked path, both
/// already canonical — but the normalization is stated rather than left implicit. `Component::ParentDir`
/// is preserved as `..`, being unresolvable without touching the filesystem. A root-only path labels
/// as `/`: the separator is written before the NEXT component, and a lone `RootDir` has none.
pub fn path_label(path: &Path) -> String {
    fn push_escaped(out: &mut String, name: &std::ffi::OsStr) {
        fn push_text(out: &mut String, text: &str) {
            for ch in text.chars() {
                if ch == '%' {
                    out.push_str("%25");
                } else {
                    out.push(ch);
                }
            }
        }

        let mut rest = name.as_encoded_bytes();
        loop {
            match std::str::from_utf8(rest) {
                Ok(text) => {
                    push_text(out, text);
                    return;
                }
                Err(err) => {
                    let (valid, invalid) = rest.split_at(err.valid_up_to());
                    push_text(
                        out,
                        std::str::from_utf8(valid)
                            .expect("`valid_up_to()` bounds a checked-valid prefix"),
                    );
                    let skip = err.error_len().unwrap_or(invalid.len()).max(1);
                    for byte in &invalid[..skip.min(invalid.len())] {
                        out.push_str(&format!("%{byte:02X}"));
                    }
                    rest = &invalid[skip.min(invalid.len())..];
                }
            }
        }
    }

    let mut label = String::new();
    let mut first = true;
    for component in path.components() {
        match component {
            std::path::Component::RootDir => {
                label.push('/');
                first = true;
                continue;
            }
            std::path::Component::CurDir => continue,
            _ => {}
        }
        if !first {
            label.push('/');
        }
        first = false;
        match component {
            std::path::Component::Prefix(prefix) => push_escaped(&mut label, prefix.as_os_str()),
            std::path::Component::ParentDir => label.push_str(".."),
            std::path::Component::Normal(name) => push_escaped(&mut label, name),
            std::path::Component::RootDir | std::path::Component::CurDir => unreachable!(),
        }
    }
    label
}

/// A compilation unit's stable identity label: its root source path **relative to the package's own
/// manifest directory** (`src/lib.rs`, `src/main.rs`, `tools/x.rs`).
///
/// `None` when the root does not lie under that directory. A caller SHALL treat that as **cannot
/// judge** (a constitution error), not as a reason to fall back to the path as given: that path is the
/// clone's own location, so keeping it would make the identity checkout-dependent — the same commit in
/// two clones yielding two identities, and a baseline recorded in one matching nothing in the other.
///
/// This is deliberately NOT the rule 漏刻 applies to a file reached through an absolute `#[path]`
/// literal, and the difference is the whole reason this returns `None` rather than the raw path: that
/// literal is **committed text**, identical in every checkout, so keeping it verbatim is exactly what
/// makes it stable. A root path outside the manifest directory is the checkout's own location, so
/// keeping it verbatim is what makes it unstable. Same shape, opposite consequence.
///
/// When the metadata carries no `manifest_path` — the shape synthetic metadata in a caller's own tests
/// has; real `cargo metadata` always carries it — the root's file name is used, which is stable and
/// sufficient because such metadata declares a single root.
///
/// The relative path is rendered by [`path_label`], so the label is the platform-independent one: a
/// Windows checkout labels `src\lib.rs` as `src/lib.rs`, matching what Linux CI recorded, instead of
/// re-firing every baseline entry as new. Because that rendering is total, `None` has exactly **one**
/// possible cause — `strip_prefix` failing — so the `out_of_package_root_error` a caller raises from it
/// is true whenever it fires, by construction rather than by wording.
pub fn compilation_unit_label(package: &Value, root_file: &Path) -> Option<String> {
    match package["manifest_path"]
        .as_str()
        .map(Path::new)
        .and_then(Path::parent)
    {
        Some(dir) => root_file.strip_prefix(dir).ok().map(path_label),
        None => root_file
            .file_name()
            .map(|name| path_label(Path::new(name))),
    }
}

/// Resolve a crate's root source file from `cargo metadata` (library target else `bin` target).
pub fn crate_root_file(package: &Value) -> Option<PathBuf> {
    let targets = package["targets"].as_array()?;
    let pick = targets
        .iter()
        .find(|t| LIBRARY_KINDS.iter().any(|k| target_has_kind(t, k)))
        .or_else(|| targets.iter().find(|t| target_has_kind(t, "bin")))?;
    pick["src_path"].as_str().map(PathBuf::from)
}

/// The workspace root directory Cargo resolved for this metadata read — the directory holding the
/// workspace manifest, whichever member manifest `--manifest-path` happened to name (Cargo resolves
/// upward to the same root either way).
///
/// Read for its **stability**, not for locating anything: it is the one directory that does not move
/// when a workspace gains, loses, or relocates a member, which is what makes it the right thing to
/// label an observed file *relative to* when that label is baseline identity (see 漏刻's
/// `audit_probe_coverage` anchor). A path derived from the observed member set instead — their
/// longest common prefix, say — is checkout-independent yet shifts the moment the set does, silently
/// restating every recorded label.
///
/// `None` when the field is absent, which real `cargo metadata` output always carries; a caller
/// holding synthetic metadata is expected to supply its own anchor rather than receive a guess.
pub fn workspace_root(metadata: &Value) -> Option<PathBuf> {
    metadata["workspace_root"].as_str().map(PathBuf::from)
}

/// Workspace member source-root directories (deduplicated and sorted).
pub fn member_src_dirs(metadata: &Value) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = metadata["packages"]
        .as_array()
        .map(|packages| {
            packages
                .iter()
                .filter_map(crate_root_file)
                .filter_map(|root| root.parent().map(Path::to_path_buf))
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    dirs.dedup();
    dirs
}

/// Every workspace member library, proc-macro, and binary crate-root source file reported by Cargo.
///
/// Which target kinds have a governable crate root is [`crate_roots`]'s filter; what is this
/// function's own is the scope — every package rather than one — and the ordering: it sorts and
/// dedups across packages, so the corpus is deterministic whatever order Cargo lists members in.
pub fn member_root_files(metadata: &Value) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = metadata["packages"]
        .as_array()
        .map(|packages| {
            packages
                .iter()
                .flat_map(|package| crate_roots(package).compiled().to_vec())
                .collect()
        })
        .unwrap_or_default();
    roots.sort();
    roots.dedup();
    roots
}

/// The runtime audit's corpus and label anchor for the workspace at `manifest_path`, in one read.
///
/// The corpus is every member's crate-root file; the anchor is Cargo's own resolved `workspace_root`, made
/// absolute. The anchor matters more than it looks: the audit labels every observed file relative to it and that
/// label is **baseline identity**, so it must be the one directory that moves neither with the checkout location
/// nor with the workspace's member set. It is the same directory whichever member manifest `--manifest-path`
/// named.
///
/// The fallback to the given manifest's own directory exists for metadata carrying no `workspace_root` field — a
/// synthetic value in a unit test; a real `cargo metadata` read always carries it. `absolute` is used rather than
/// canonicalization so the cargo-reported root is never rewritten, and it refuses only an empty path, hence the
/// working-directory last resort for a bare `Cargo.toml` whose parent is empty.
///
/// This lives here, in the single reader of truth, because two dimensions derived it separately once: the shell
/// computed it for the runtime audit while a runtime observer would have had to compute it again, and a
/// twin derivation of a baseline-identity anchor is the drift this crate exists to prevent.
pub fn audit_corpus_and_anchor(manifest_path: &Path) -> Result<(Vec<PathBuf>, PathBuf), String> {
    let metadata = cargo_metadata(manifest_path)?;
    let roots = member_root_files(&metadata);
    let anchor = match workspace_root(&metadata) {
        Some(root) => root,
        None => {
            let manifest_dir = manifest_path.parent().unwrap_or(Path::new(""));
            std::path::absolute(manifest_dir).or_else(|_| std::env::current_dir()).map_err(|err| {
                format!(
                    "no anchor: {manifest_path:?} names no absolute directory and the working directory \
                     cannot be read ({err}). Every observed file is labelled relative to the anchor, so \
                     inventing one would mislabel a whole baseline rather than fail"
                )
            })?
        }
    };
    Ok((roots, anchor))
}

/// Whether a `metadata` failure means the target IS NOT THERE, as against this reader not finding out.
///
/// **`Path::is_file` answers `false` for both, and the three dimensions each collapsed them.** A
/// `#[cfg]`-gated or `cfg_attr`-remapped declaration is allowed to have an absent target — that is the
/// tolerance a cfg-blind walker owes a conditional module — so a target this reader merely could not stat
/// was swallowed by that tolerance, with whatever the subtree holds going unobserved. That is the false
/// negative the Core Contract forbids, and it stood in 圭表, 渾儀 and 漏刻 alike.
///
/// **The criterion, so a later error kind is placed rather than guessed.** An error saying *this path
/// cannot name anything* is an absence; one saying *this reader could not find out* is not. Measured as
/// uid 1000: a directory at mode `000` holding `mod.rs` gives `is_file` false with kind `PermissionDenied`
/// — this reader could not find out. With a plain file where a module directory would be, the kind is
/// `NotADirectory` — a component that is not a directory cannot have children, so the target cannot exist.
///
/// `FilesystemLoop` is deliberately NOT an absence: a cycle is a thing this reader could not resolve, and
/// a walker that tolerates it silently would drop whatever the cycle hides. Kinds beyond the two measured
/// are left on the loud side rather than sorted on a guess.
///
/// It lives here because all three dimensions ask it and none may ask another: the substrate is where a
/// question they share is answered once, the way `canonicalize_or_fail` already is. They ask it **through**
/// [`is_regular_file`] and [`is_directory`], which are its only callers, so the criterion itself is
/// crate-private — a published name is a promise this crate would have to keep, and nothing outside asks
/// for it. A dimension that later needs to place a new error kind changes the criterion here, which is the
/// point of it living here at all.
pub(crate) fn is_absence(kind: std::io::ErrorKind) -> bool {
    matches!(
        kind,
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
    )
}

/// Whether `path` is a regular file, or why this reader could not tell — the criterion is
/// `is_absence`, crate-private beside these two, which are its only callers.
pub fn is_regular_file(path: &Path) -> Result<bool, String> {
    match std::fs::metadata(path) {
        Ok(found) => Ok(found.is_file()),
        Err(err) if is_absence(err.kind()) => Ok(false),
        Err(err) => Err(format!(
            "cannot read '{}' to decide whether it backs a module — {err}; an unreadable target is not \
             an absent one, and tolerating it would leave whatever it holds unobserved",
            path.display()
        )),
    }
}

/// Whether `path` is a directory, or why this reader could not tell — [`is_regular_file`]'s sibling.
pub fn is_directory(path: &Path) -> Result<bool, String> {
    match std::fs::metadata(path) {
        Ok(found) => Ok(found.is_dir()),
        Err(err) if is_absence(err.kind()) => Ok(false),
        Err(err) => Err(format!(
            "cannot read '{}' to decide whether it holds a module's children — {err}; an unreadable \
             directory is not an absent one",
            path.display()
        )),
    }
}

/// Claim `path` as a directory this process created, refusing to adopt one that already exists.
///
/// **Fixture infrastructure, deliberately withheld from the API contract.** It is `pub` because test targets
/// in other crates need it across crate boundaries, and `#[doc(hidden)]` because 星表's domain is
/// declared workspace data — a scratch-directory claim is not in it. It lives here for a dependency-graph
/// reason (the lightest crate already reachable from every dimension), and a dependency-graph reason is not
/// a domain fit.
///
/// The distinction is not cosmetic at `0.5.0`, which is the release that first publishes it: a documented
/// item is a promise an adopter may build on and this family cannot then withdraw, while a hidden one keeps
/// every present caller compiling and commits to nothing. Zero call sites outside a test target or a
/// `#[cfg(test)]` module were measured before hiding it.
///
/// **`create_dir_all` accepts a pre-existing symlink to a directory.** Measured: `create_dir_all` on such a
/// link returns `Ok(())` and every subsequent write lands in the link's *target*, while `create_dir` cannot
/// follow a symlink and returns `AlreadyExists` — the narrow call is what makes the refusal possible at all.
///
/// A fixture scratch root is typically built from `temp_dir()` and a predictable name (a process id, a
/// counter), so it is guessable by anyone on the machine. The ordinary `remove_dir_all` then `create_dir_all`
/// idiom leaves a window between the two calls in which a planted symlink is silently adopted, and everything
/// the fixture writes afterward lands wherever that link points. `create_dir` closes the window rather than
/// narrowing it: the root is claimed only if this call is the one that created it, and `remove_dir_all`
/// itself removes a symlink at the path rather than following it, so an attacker must win a race rather than
/// leave something lying around in advance.
///
/// **This is about the root only.** Every directory beneath a root this call already claimed is safe to build
/// with the ordinary `create_dir_all`, because nothing outside the fixture's own control could have planted
/// anything there first.
#[doc(hidden)]
pub fn claim_scratch(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir(path)
}

/// An RAII guard that restricts a file or directory to mode 000 during test execution
/// and restores its original permissions on drop.
///
/// **Fixture infrastructure, deliberately withheld from the API contract.**
/// Like [`claim_scratch`], it is `pub` because test targets across crates need it across crate
/// boundaries, and `#[doc(hidden)]` because 星表's domain is declared workspace data.
///
/// Under a privileged user (such as root in a container), mode 000 does not restrict access.
/// Outside `TIANHENG_WORKSPACE_TESTS`, it restores permissions and returns `None` so the calling
/// test can skip gracefully. Inside `TIANHENG_WORKSPACE_TESTS`, a silent skip is forbidden and
/// asserts.
#[cfg(unix)]
#[doc(hidden)]
pub struct Unreadable<'a> {
    path: &'a Path,
    original: std::fs::Permissions,
}

#[cfg(unix)]
impl<'a> Unreadable<'a> {
    /// Restrict `path` to mode 000, returning `None` if mode 000 does not bite (e.g. running as root).
    pub fn try_new(path: &'a Path) -> Option<Self> {
        use std::os::unix::fs::PermissionsExt;
        let original = std::fs::metadata(path)
            .expect("read initial permissions")
            .permissions();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o000))
            .expect("drop read/traverse permissions");
        let accessible = if path.is_dir() {
            std::fs::read_dir(path).is_ok()
        } else {
            std::fs::read(path).is_ok()
        };
        if accessible {
            std::fs::set_permissions(path, original)
                .expect("restore permissions when mode 000 did not bite");
            assert!(
                std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
                "mode 000 did not restrict the path — running as root would make this direction vacuous"
            );
            return None;
        }
        Some(Self { path, original })
    }
}

#[cfg(unix)]
impl Drop for Unreadable<'_> {
    fn drop(&mut self) {
        if let Err(err) = std::fs::set_permissions(self.path, self.original.clone()) {
            if std::thread::panicking() {
                eprintln!(
                    "Unreadable: failed to restore permissions for '{}' during unwind: {err}",
                    self.path.display()
                );
            } else {
                panic!(
                    "Unreadable: failed to restore permissions for '{}': {err}",
                    self.path.display()
                );
            }
        }
    }
}
