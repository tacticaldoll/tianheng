# module-boundary Specification Delta

## MODIFIED Requirements

### Requirement: An unconditional path-remapped module is followed to its target

The system SHALL follow a file-form module declared with an **unconditional, direct** `#[path = "…"]` attribute (`mod foo;`) to its author-chosen target: the target's imports SHALL be observed under the declared module's logical path, and the module SHALL be a governable target at that path — matching 渾儀 (semantic) and 漏刻 (runtime), which already follow this same relocation, so all three observation dimensions agree on what rustc actually compiles. The target is resolved relative to `path_base`: the declaring file's own directory, with each enclosing inline `mod` name accumulated onto it (rustc's rule) — the crate root's own directory for a `#[path]` written there, or `<accumulated dir>/<name>` for one written inside an inline `mod name { … }`. A `#[path]`-loaded file is itself mod-rs-like, so a `#[path]` or conventional child written inside it resolves from ITS OWN directory in turn. A same-named conventional file beside the remapped declaration (e.g. `foo.rs` beside a `#[path = "weird.rs"] mod foo;`) remains an orphan Rust never compiles as that module — it SHALL NOT be governed in the remap's target place, the same "not compiled ⇒ not governed" rule as an undeclared orphan and an inline-only shadow, now applied to the remap case instead of excluding the whole logical path.

An unconditional target that does not exist on disk is a genuine broken reference (rustc itself errors on it) and SHALL be a scan error (exit 2), never a silent skip. A `#[path]` chain that resolves back to a source file already open on the path from the crate root (only possible through `#[path]`, since ordinary conventional/inline nesting is bounded by the crate's finite file list) SHALL likewise be a scan error, never an unbounded walk — tracked by the set of files open on the *current descent path*, not a monotonic whole-crate visited set, so two sibling or cousin declarations legitimately sharing one `#[path]` target (rustc compiles the same file twice, as two distinct modules) is never misreported as a cycle.

The scanner SHALL recognize both a direct `#[path = "…"]` and a `path = "…"` meta recursively wrapped in one or more `#[cfg_attr(predicate, …)]` attributes. When a direct `#[path = "…"]` attribute is present, it SHALL take precedence over any sibling `cfg_attr` paths on the same declaration. When no direct `#[path = "…"]` attribute is present and one or more `cfg_attr(..., path = "...")` attributes are recognized, the scanner SHALL collect all candidate remapped targets and perform a union-scan across all candidate target files that physically exist on disk (`path.exists()`). Candidate files that do not exist on disk SHALL be safely skipped without triggering a scan error (treating them as absent under the active compilation target). An unconditional `#[path]` attribute on an inline module (`mod foo { … }`) does not relocate the module's own content (rustc treats the attribute as a no-op for that purpose — the body already IS the module) and SHALL NOT make that inline module disappear from reachability, but it DOES relocate the base directory the inline body's OWN file-form children resolve from, exactly as it would for a file-form declaration — the system SHALL follow it there too, resolved from the declaring source's own `path_base`.

#### Scenario: A path-remapped module's imports are observed at its real target

- **WHEN** a crate declares `#[path = "weird/place.rs"] mod foo;` and `weird/place.rs` contains `use crate::other::Thing;`
- **THEN** the system observes an import of `crate::other` for `crate::foo`, attributed to `weird/place.rs`

#### Scenario: A path-remapped module is a governable target at its real file

- **WHEN** a boundary targets `crate::foo`, declared via `#[path = "weird/place.rs"] mod foo;`, and `weird/place.rs` contains a forbidden import
- **THEN** the system reports the violation naming `crate::foo` and the file `weird/place.rs` — never a constitution error, and never governing a same-named conventional orphan in its place

#### Scenario: A conventional orphan beside a path-remapped declaration is not governed

- **WHEN** a crate declares `#[path = "weird.rs"] mod foo;`, `weird.rs` is clean, and a conventional `foo.rs` also exists containing a forbidden import
- **THEN** the system reports no violation for `foo.rs` — the orphan is not compiled as `crate::foo` and is never governed in the remap's place

#### Scenario: A plain child of a path-remapped module is governed under its logical path

- **WHEN** a crate declares `#[path = "other/weird.rs"] mod kernel;`, `other/weird.rs` declares a plain `mod child;`, and `other/child.rs` contains a forbidden import
- **THEN** the system observes the import under `crate::kernel::child`, attributed to `other/child.rs` — even though `other/child.rs`'s own on-disk location does not structurally match its logical path

#### Scenario: A missing unconditional path target is a scan error

- **WHEN** a crate declares `#[path = "absent.rs"] mod foo;` and `absent.rs` does not exist
- **THEN** the system reports a scan error (exit 2), because an unconditional `#[path]` target absent from disk is a genuine broken reference rustc itself rejects — never a silent skip

#### Scenario: Two declarations sharing one path target is not a cycle

- **WHEN** a crate declares `#[path = "s.rs"] mod a;` and `#[path = "s.rs"] mod b;`, both resolving to the same real file `s.rs`
- **THEN** the system reports both `crate::a` and `crate::b` as reachable and governable, because rustc compiles `s.rs` twice as two distinct modules — never a reported cycle

#### Scenario: A path chain cycling back to an already-open file is a scan error

- **WHEN** a crate declares `mod a { #[path = "../lib.rs"] mod b; }` at the crate root, and `a`'s own accumulated directory makes `../lib.rs` resolve back to the crate root file itself
- **THEN** the system reports a scan error (exit 2) rather than looping or overflowing the stack — the cycle is on the descent path from the crate root, not merely a repeated file elsewhere in the crate

#### Scenario: A nested path crossing into a mutually-exclusive cfg sibling's own target is not a cycle

- **WHEN** a crate declares `#[cfg(feature = "a")] #[path = "variant_a.rs"] mod imp;` and `#[cfg(feature = "b")] #[path = "variant_b.rs"] mod imp;`, and `variant_a.rs` itself declares `#[path = "variant_b.rs"] mod also_b;`
- **THEN** the system follows `crate::imp::also_b` to `variant_b.rs` without reporting a cycle, because the two `#[cfg]` arms' targets are never simultaneously open in any real single build — the ancestor tracking that would otherwise misreport this is scoped per physical source file, never merged across mutually-exclusive `#[cfg]` sibling arms of one logical module path

#### Scenario: A nested path inside a plain child of an inline cfg arm is not a cycle through a sibling arm

- **WHEN** a crate declares `#[cfg(feature = "u")] mod x { mod y; }` (inline) and `#[cfg(feature = "w")] #[path = "windows_x.rs"] mod x;` (file-form), where only the inline arm declares the plain child `y`, and `y`'s own file declares `#[path = "../windows_x.rs"] mod cross;`
- **THEN** the system follows `crate::x::y::cross` to `windows_x.rs` without reporting a cycle — a plain child's ancestor set is scoped to only the source that actually declared it, never unioned across `x`'s other mutually-exclusive `#[cfg]` sources

#### Scenario: A grandchild of a plain child of a path-remapped module is governed under its logical path

- **WHEN** a crate declares `#[path = "other/weird.rs"] mod kernel;`, `other/weird.rs` declares a plain `mod child;` (resolved to `other/child.rs`), and `other/child.rs` itself declares a further plain `mod grandchild;`
- **THEN** the system observes a forbidden import in `other/child/grandchild.rs` under `crate::kernel::child::grandchild` — the ordinary stem-subdirectory convention relative to `child.rs`'s own location, since a plain child reached through a remap is not itself mod-rs-like for its own further children

#### Scenario: A stray file at a remapped module's naive structural location is not phantom-governed

- **WHEN** a crate declares `#[path = "other/weird.rs"] mod kernel;`, `other/weird.rs` declares a plain `mod child;` (resolved to the real `other/child.rs`), and an unrelated, wholly undeclared file also happens to physically sit at `kernel/child.rs` (the location a plain `mod child;` inside a NON-remapped `kernel` would occupy)
- **THEN** the system governs only `other/child.rs` under `crate::kernel::child` — the stray file at `kernel/child.rs` is never compiled by rustc (`kernel` is wholly remapped) and must never be phantom-governed alongside the real one merely for coincidentally sharing its naive structural path

#### Scenario: A cfg_attr-wrapped path attribute undergoes union-scan when target file exists

- **WHEN** a crate declares `#[cfg_attr(unix, path = "weird.rs")] mod foo;`, `weird.rs` exists on disk containing `use crate::forbidden::Y;`, and a boundary governs `crate::foo` forbidding `crate::forbidden`
- **THEN** the system observes the import in `weird.rs` for `crate::foo` and reports the violation, performing union-scan over all physically existing candidate files rather than marking the module out of scope

#### Scenario: A missing cfg_attr-wrapped path target is safely skipped

- **WHEN** a crate declares `#[cfg_attr(windows, path = "win_only.rs")] mod foo;` and `win_only.rs` does not exist on disk
- **THEN** the system skips `win_only.rs` without raising a scan error, treating absent conditional targets as inactive under the current source checkout

#### Scenario: A nested cfg_attr-wrapped path attribute is recognized as a candidate remap

- **WHEN** a crate declares `#[cfg_attr(a, cfg_attr(b, path = "weird.rs"))] mod foo;` and `weird.rs` exists on disk
- **THEN** the system recursively recognizes the applied `path` target and includes `weird.rs` in the union-scan for `crate::foo`

#### Scenario: A cfg_attr without an applied path remains governable

- **WHEN** a crate declares `#[cfg_attr(path, allow(dead_code))] mod foo;` with a conventional `foo.rs`
- **THEN** the system does not mistake the predicate named `path` for a remap and governs the conventional module normally

#### Scenario: An unconditional path attribute wins regardless of attribute order

- **WHEN** a crate declares `#[cfg_attr(some_platform, path = "b.rs")] #[path = "a.rs"] mod foo;` — a cfg-conditional remap textually BEFORE the unconditional one on the same declaration
- **THEN** the system follows the unconditional `#[path = "a.rs"]` target exactly as it would if the two attributes were written in the opposite order, since rustc compiles `a.rs` whenever `some_platform` does not hold regardless of which attribute is written first

#### Scenario: An unconditional path on an inline module relocates its own file-form children

- **WHEN** a crate declares `#[path = "thread_files"] pub mod thread { pub mod local_data; }`, `thread_files/local_data.rs` contains a forbidden import, and no `thread/` directory exists at all
- **THEN** the system observes the forbidden import under `crate::thread::local_data`, attributed to `thread_files/local_data.rs` — the `#[path]` attribute is not treated as a no-op merely because the module it precedes is inline; it relocates where the inline body's own file-form children resolve from, exactly as it would for a file-form declaration
