# Example — 圭表 (guibiao) standalone

Adopt **just the static dimension**: the syn-free import/dependency linter, one dependency,
no `syn`. This is a tiny hexagonal app whose `domain` must not import `infra` — and
deliberately does, so you can watch 圭表 react.

```toml
[dependencies]
guibiao = "0.2"   # the whole footprint — light by design
```

Declare the law in Rust (`src/governance.rs`), then react:

```rust
use guibiao::check;

let outcome = check(&constitution(), std::path::Path::new("Cargo.toml"));
std::process::exit(outcome.exit_code() as i32); // 0 clean · 1 violation · 2 constitution error
```

## Run it

```sh
cargo run --bin demo     # renders the reaction, exits 1 (the domain→infra import)
cargo test               # asserts the reaction + the adoption ladder + the stability contract
```

`tests/reaction.rs` demonstrates, as runnable proof:

- **the reaction** — the `domain → infra` import trips the enforce boundary (exit 1);
- **the adoption ladder** — `warn` reports without gating (exit 0); a `Baseline` grandfathers
  existing violations (exit 0) while an un-baselined one still reacts (exit 1);
- **identity ⊥ presentation/metadata** — semantic identity is governed target + `RuleKey` +
  `StructuredFactIdentity`, so finding wording and `file` relocation do not churn the baseline.

> Most adopters graduate from one 儀 to the composed constitution via the
> [`tianheng`](https://crates.io/crates/tianheng) shell — see the `composed` example.
> Experimental / pre-1.0: public faces may change until real adoption pressure settles them.
