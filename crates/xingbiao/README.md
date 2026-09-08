# 星表 / xingbiao

**眾儀所稽,一表為宗。** — *What every instrument consults, one catalogue as its source.*

**The shared declared-workspace-data substrate of [Tianheng](https://github.com/tacticaldoll/tianheng) — the reader below the 三儀.**

星表 (the star catalogue) reads `cargo metadata --no-deps` and looks up packages and their
crate-root source files: the tabulated register of declared workspace data every observation
dimension references before it observes. It spawns `cargo` and parses its JSON — **`serde_json`
+ std only, no `syn`**.

It sits **below the 三儀**, like 璇璣 (the reaction model): a dimension depends on it one-way
(downward), so the static (圭表) and semantic (渾儀) dimensions read the workspace through **one**
reader instead of two hand-copied twins that drift apart. It is **not 璇璣** — 璇璣 is the
measure-only reaction model that renders no verdict, whereas 星表 does IO and *observes*.

It exposes:

- `cargo_metadata` — run `cargo metadata --no-deps` and parse the JSON (workspace members only).
- `find_package` — find a workspace member by package name.
- `crate_root_file` — a crate's root source file (`lib`, else `proc-macro`, else `bin`), the
  single resolution both dimensions share so they cannot disagree on which crates are judgeable.

Consumed as a library within the Tianheng workspace; it is not a standalone tool.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
