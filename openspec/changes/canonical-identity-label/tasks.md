# Tasks

## 1. The shared primitive

- [x] 1.1 Add `xingbiao::path_label(&Path) -> String`, built on `Path::components()` joined with `/`,
      each component's bytes percent-escaped by the rule `probes::encoded` already stated. Documents why
      `components()` rather than character substitution, and the two normalizations it implies
      (`./a` → `a`, `a//b` → `a/b`).
- [x] 1.2 Unit-tested: realistic relative paths byte-identical to the previous encoding; a `%` path; a
      non-UTF-8 path (and two differing only there keeping two labels); an absolute path keeping its
      leading `/`; a unix backslash staying one component and distinct from the two-component path. Plus
      the platform-invariant assertion and a `#[cfg(windows)]` case, both labelled as not executing here.

## 2. 圭表 and 渾儀's compilation unit

- [x] 2.1 `compilation_unit_label` rewritten on `path_label`, so a Windows checkout labels `src\lib.rs`
      as `src/lib.rs`. Because that rendering is total, its `None` now has exactly one possible cause.
      Doc updated, including the no-`manifest_path` fallback.
- [x] 2.2 **Scope corrected — the non-UTF-8 half of the finding is refuted, not fixed.** The task as
      written assumed a non-UTF-8 root inside the package directory was being refused with a false
      diagnostic. It is unreachable, measured four ways: `cargo metadata` under a non-UTF-8 directory
      fails outright (`error: path contains invalid UTF-8 characters`, exit 101); an auto-discovered
      target whose file name is not valid UTF-8 is silently omitted from the target list; a `Cargo.toml`
      is UTF-8 so a `[[bin]] path` literal cannot spell one; and `src_path`/`manifest_path` arrive as
      JSON strings, so any path built from them is valid UTF-8 by construction and `to_str()` cannot
      return `None`. The `None`-means-one-thing property still lands, as a free consequence of the shared
      primitive being infallible rather than as a fix.
- [x] 2.3 **Guard dropped as unwritable, with the reason recorded.** A test cannot construct the input:
      cargo refuses to read the fixture. The measurements in 2.2 stand in its place, and the proposal
      records them so the finding is not re-raised. `PROJECT.md`'s minimalism bound forbids the branch
      and diagnostic review proposed for this state.

## 3. 漏刻's observed file

- [x] 3.1 Private `encoded` retired in favour of `path_label`, at both call sites (`labeled`, and
      `scan_rust_file`'s absolute-`#[path]` branch which keeps the path as the literal wrote it).
      `labeled` keeps its strip-prefix-else-absolute rule, which the shared function does not change.
- [x] 3.2 No `unauditable-probe` label changed: the full suite passes unchanged (42 groups, 0 failures),
      including the audit tests that pin labels for the injectivity and absolute-`#[path]` rules this
      window established.

## 4. Declaration

- [x] 4.1 Sync the `structured-violation-identity` delta.
- [x] 4.2 CHANGELOG: the Windows re-key it prevents, the shared rule, and the measured no-op for
      existing unix baselines — plus the refuted half, so the record does not overclaim.
- [x] 4.3 Full Definition of Done (13 gates), with the Windows half recorded as argued from std's own
      source rather than executed.
