# Example — 渾儀 (hunyi) standalone

Adopt **just the semantic dimension**: the public-API exposure linter. Where 圭表 asks *does
`domain` import `infra`?*, 渾儀 asks *does a `pub` surface expose a forbidden type?* — the
complement. A type imported for internal use is fine; a type named in a `pub` signature is a
leak, and one named by a fully-qualified path (no `use`) is invisible to a token scanner but
caught here.

```toml
[dependencies]
hunyi = "0.1"   # the semantic instrument carries the quarantined `syn` — the honest footprint
```

Declare the law and react:

```rust
use hunyi::check;

let outcome = check(&constitution(), std::path::Path::new("Cargo.toml"));
std::process::exit(outcome.exit_code() as i32); // 0 clean · 1 violation · 2 constitution error
```

## Run it

```sh
cargo run --bin demo     # renders the reaction, exits 1 (api::connection leaks infra::DbPool)
cargo test               # asserts the leak reacts, and that a non-exposed type does not
```

> Graduate from one 儀 to the composed constitution via the
> [`tianheng`](https://crates.io/crates/tianheng) shell — see the `composed` example.
> Experimental / pre-1.0: public faces may change until real adoption pressure settles them.
