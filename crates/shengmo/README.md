# 繩墨 / shengmo

**惟木從繩則正。** — *Wood that follows the inked line is made straight.* (尚書·說命上)

**This repository's own law, and the reactions that run [Tianheng](https://github.com/tacticaldoll/tianheng)'s delivered product against this workspace.** Not product. Ships in no package.

繩墨 (the carpenter's inked line) is snapped across the work to mark true. Everything is judged
against it, and the line is not part of the furniture — which is the property this crate exists to
have. `cargo publish` never packages it, and that is the criterion this repository gives for
machinery being *governance* rather than *product*.

It holds two things:

- **The law.** `src/law.rs` declares 天衡's self-constitution through the same published surface an
  adopter uses — `Constitution`, `CrateBoundary`, `SansIoPure` — so the repository's own governance
  exercises exactly the API it ships. A declaration is code, not a test: what runs it is a reaction,
  and what reads it is the generated projection [`AGENTS.self-law.md`](../../AGENTS.self-law.md).
- **The reactions that run it.** `tests/self_governance.rs` evaluates that constitution against this
  workspace as a `cargo test` gate, and `tests/examples_suite.rs` asserts each example's **exit
  code** — a demo that reacts exits 1, a run-mode that only reports exits 0 — so an example is held
  to what its documentation claims rather than to compiling.

## Not one of the 三儀, and not one of the 三司

璇璣, 星表, 圭表, 渾儀, 漏刻 and 天衡 are instruments (儀): they measure, and they are the product.
垂象, 實錄 and 校讎 are offices (司): they administer a reaction's surface, its record, and its
amendment — and none of them is a crate. 繩墨 is neither. It measures nothing and administers
nothing; it is the line the workshop keeps, and it belongs to the workshop rather than to what
leaves it.

Its sibling [`kanhe`](../kanhe/README.md) (勘合) holds the other half of this repository's
governance: not the law, but whether the repository's own record agrees with itself.

## Layout

- `src/` — the law, and the locator every reaction reads the repository through.
- `src/tests/` — failure matrices, beside the judgements they test.
- `tests/` — the reactions themselves.

## License

Licensed under either of Apache-2.0 or MIT, at your option — the same terms as the family it
governs, though this crate is published nowhere.
