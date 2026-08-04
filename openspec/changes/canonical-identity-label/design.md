# Design

## Where the answer lives

星表, because it is already the substrate all three dimensions read before they observe, and because
the two callers are in different dimensions — putting it in either would make one dimension depend on
the other, which the self-law forbids. 漏刻 already depends on 星表 behind its `audit` feature, which
is the same feature that gates `labeled`, so no new edge is created.

## The primitive

```rust
pub fn path_label(path: &Path) -> String
```

Built on `Path::components()`, joined with `/`. That choice is the whole design, for one reason:
**separator semantics are delegated to `std::path` rather than re-implemented.** A hand-rolled
`replace('\\', "/")` would be wrong in both directions — on unix `\` is a legal filename byte, so
replacing it would conflate the single file `a\b` with the file `b` inside directory `a`, breaking the
injectivity this exists to provide; on Windows both `\` and `/` separate, so replacing one is
incomplete. `components()` is the only thing that knows which is which per platform.

Each component contributes:

| Component | Contributes | Why |
|---|---|---|
| `Prefix` | its escaped text | Windows `C:` — part of the path's identity when the absolute fallback keeps it |
| `RootDir` | an empty part | so the `/`-join produces the leading `/` naturally |
| `CurDir` | nothing | `./a` and `a` are the same file |
| `ParentDir` | `..` | preserved; it is not resolvable without touching the filesystem |
| `Normal` | its escaped text | the names |

Escaping is `encoded`'s existing rule, moved: percent-escape every byte not part of a valid UTF-8
sequence, and `%` as `%25` so no escaped label can be spelled by an unescaped one. A component cannot
contain `/` on any platform, so `/` in a label unambiguously means a component boundary.

## Why this does not re-key existing baselines

Measured against the current `encoded(path.as_os_str())` for every shape that actually occurs:

| Input | New | Old | |
|---|---|---|---|
| `src/lib.rs` | `src/lib.rs` | `src/lib.rs` | same |
| `src/bin/x.rs` | `src/bin/x.rs` | `src/bin/x.rs` | same |
| `/abs/src/lib.rs` | `/abs/src/lib.rs` | `/abs/src/lib.rs` | same |
| `tools/outside.rs` | `tools/outside.rs` | `tools/outside.rs` | same |
| `with%pct/f.rs` | `with%25pct/f.rs` | `with%25pct/f.rs` | same |
| `src/ba\xFFd.rs` | `src/ba%FFd.rs` | `src/ba%FFd.rs` | same |
| `a\b` (unix, one file) | `a\b` | `a\b` | same |
| `./a` | `a` | `./a` | differs |
| `a//b` | `a/b` | `a//b` | differs |

The two that differ are normalizations of forms that name the same file, and neither is reachable from
the inputs these labels are built from: both sites strip a prefix from a `cargo metadata` `src_path` or
a walked path, which are already canonical. They are stated as a bound rather than left implicit.

## The Windows half is argued from std's source, not measured

There is no Windows runner and no wine in this environment, so the claim "`src\lib.rs` yields
`src/lib.rs` on Windows" is **not executed here**. It rests on two things, both checkable:

- `Components` splits on `sys::path::is_sep_byte`, and the installed std source defines that per
  platform: `library/std/src/sys/path/windows.rs:12` is `path_separator_bytes!(b'\\', b'/')`, while
  `.../unix.rs:5` is `path_separator_bytes!(b'/')`. So `\` separates on Windows and is data on unix,
  which is the entire behaviour this design leans on — and also why substituting characters would be
  wrong on unix and incomplete on Windows.
- The unix half of the same delegation IS measured: `a\b` labels as `a\b` (one component) and is
  asserted distinct from `a/b`. That is what demonstrates the function reads `Path`'s answer rather
  than hardcoding a separator; given that, the Windows answer follows from the definitions above.

Two tests carry it: one asserting the invariant directly (no label contains `MAIN_SEPARATOR` unless
that separator is `/`), which is vacuous on unix and load-bearing on Windows, and one `#[cfg(windows)]`
test stating the case outright. Both are labelled as not running here, so no reader mistakes a green
unix suite for coverage of the platform the change is about.

## Where the byte-injectivity rule earns its place, and where it is free

The escaping half of `path_label` is load-bearing for 漏刻 only. 漏刻's labels come from filesystem
walks (`read_dir`), where a name that is not valid UTF-8 is genuinely reachable — which is why
`encoded` exists and why its doc already reasons about WTF-8. 圭表/渾儀's come from Cargo's JSON, which
cannot carry such a path (see the proposal's four measurements). So the shared function is not being
generalized on speculation: it is being moved to where both callers can hold the rule that one of them
provably needs, and the other gets an infallible `to_str()` for free.

That "for free" is worth naming precisely, because it is the difference between this change and the one
review proposed. Making `path_label` infallible means `compilation_unit_label`'s `None` can only come
from `strip_prefix`, so the constitution error it triggers is true whenever it fires — by construction,
not because the sentence was improved. Review's alternative was to give the function a `Result` with
two distinct causes. That would have added a branch and a diagnostic for a state cargo cannot produce,
which `PROJECT.md`'s minimalism bound forbids ("no defensive over-foolproofing of impossible states"),
and it would have left the separator defect — the one that is actually reachable — untouched.
