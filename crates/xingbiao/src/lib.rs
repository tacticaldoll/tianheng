//! 星表 (xīngbiǎo) — the shared declared-workspace-data substrate.
//!
//! Reads `cargo metadata --no-deps` and looks up packages and their crate-root source files:
//! the tabulated catalog every observation dimension references before it observes. Spawns
//! `cargo` and parses its JSON (`serde_json` + std only, no `syn`). Also carries the shared
//! path-identity primitives ([`canonicalize_or_fail`], [`try_visit`]) a module-graph cycle/dedup
//! guard needs — the same "single reader of truth" role, one file-identity notch finer than
//! which file is a crate root.
//!
//! Sits beneath static (圭表) and semantic (渾儀) dimensions as a single reader of truth,
//! preventing twin-drift in target resolution across observation dimensions.
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
/// check shared by [`crate_root_file`] (picking one library/bin target) and
/// [`member_root_files`] (filtering every library/bin target across the workspace).
fn target_has_kind(target: &Value, wanted: &str) -> bool {
    target["kind"]
        .as_array()
        .is_some_and(|kinds| kinds.iter().any(|k| k.as_str() == Some(wanted)))
}

/// **Every** compiled crate root of ONE package — each library-kind target and each `bin` target, in
/// Cargo's reported order, deduplicated.
///
/// The per-package counterpart of [`member_root_files`] (which spans the workspace) and the plural of
/// [`crate_root_file`] (which picks one). A package's roots are separate compilation units: they each
/// denote the module path `crate` and neither's declarations belong in the other's module graph, so a
/// dimension that governs a package governs each root as its own corpus. Returning them all is what lets
/// a violation written in a `bin` beside a library be observed at all.
///
/// Empty when the metadata reports no target — the shape synthetic metadata in a caller's own tests
/// carries. A caller SHALL treat that as "fall back to the conventional source directory", not as "this
/// package has no source": dropping that fallback silently un-governs every such test fixture.
///
/// The uniqueness is **total**, not adjacency-dependent, and that distinction is load-bearing here in a
/// way it is not in [`member_root_files`] (which sorts first, so `Vec::dedup` is total for it by
/// construction). Two targets may name the same `path` — Cargo accepts it and builds both — and Cargo
/// reports targets sorted by NAME, so the two reports are adjacent only if no third target's name sorts
/// between them. `Vec::dedup` alone therefore left `[x, y, x]` intact, and the root was scanned once per
/// report. Order is Cargo's own and is preserved, because a caller's sibling-root exclusion reads this
/// slice positionally against the root it is currently walking.
pub fn crate_root_files(package: &Value) -> Vec<PathBuf> {
    let mut seen = std::collections::HashSet::new();
    package["targets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|t| {
            LIBRARY_KINDS.iter().any(|k| target_has_kind(t, k)) || target_has_kind(t, "bin")
        })
        .filter_map(|t| t["src_path"].as_str().map(PathBuf::from))
        .filter(|root| seen.insert(root.clone()))
        .collect()
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
/// Without this, one commit produced `src/lib.rs` on Linux and `src\lib.rs` on Windows, and a baseline
/// recorded by CI matched nothing for a Windows contributor — every entry re-firing as new. That is the
/// checkout-dependence class this window closed five times, along the one axis none of those five
/// covered: not where the repository sits, but which platform read it.
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
/// is preserved as `..`, being unresolvable without touching the filesystem.
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
                    // `valid_up_to()` bounds a checked-valid prefix, so this cannot fail.
                    push_text(out, std::str::from_utf8(valid).unwrap_or_default());
                    // `error_len() == None` means the input ends mid-sequence: every remaining byte is
                    // unusable, so escape all of them rather than looping forever on the same slice.
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
            // A `RootDir` contributes no text, so the separator written before the NEXT component is
            // what becomes the leading `/`. A path that is only `RootDir` therefore labels as `/`.
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
pub fn member_root_files(metadata: &Value) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = metadata["packages"]
        .as_array()
        .map(|packages| {
            packages
                .iter()
                .flat_map(|package| {
                    package["targets"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|target| {
                            LIBRARY_KINDS.iter().any(|k| target_has_kind(target, k))
                                || target_has_kind(target, "bin")
                        })
                        .filter_map(|target| target["src_path"].as_str().map(PathBuf::from))
                })
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
            // The working directory is the last resort for a bare `Cargo.toml` whose parent is empty. Where
            // even that cannot be read there is no anchor, and the error channel in this signature says so:
            // an invented root mislabels every observed file, silently, because the anchor *is* baseline
            // identity. Inventing one here would also be the defensive over-foolproofing of an impossible
            // state the minimalism bound forbids.
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
