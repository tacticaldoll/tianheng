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

## The Windows half is argued, not measured

There is no Windows runner and no wine in this environment, so the claim "`src\lib.rs` yields
`src/lib.rs` on Windows" is **not executed here**. What is measured is the load-bearing half of the
argument: on unix a `\` is carried through as one component's data (so the function is demonstrably
delegating to `Path`, not hardcoding `/`), and `components()`' Windows separator parsing is
`std::path`'s documented behaviour. A test asserts the invariant directly — no label contains
`MAIN_SEPARATOR` unless that separator is `/` — which is vacuous on unix and load-bearing on Windows,
and is labelled as such so no reader mistakes a green unix run for coverage of the case.

## Alternative rejected

Giving `compilation_unit_label` a `Result` with two distinct causes, as first proposed by review. It
fixes the misdiagnosis and leaves the divergence: 圭表/渾儀 would still refuse an input 漏刻 governs, and
refusing a package that compiles is a scope loss the family pays for nothing. Escaping judges it, and
makes `None` mean one thing by construction rather than by a better sentence — the same reason the
per-root deferral earlier in this window was fixed with a type rather than with a clearer message.
