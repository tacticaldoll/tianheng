# Published-artifact provenance

What commit each published version of the family actually names, and where that disagrees with the
tag. Recorded because the disagreements are **permanent**: `cargo publish` writes the sha1 of whatever
`HEAD` it ran on into every tarball's `.cargo_vcs_info.json`, and a version can never be re-uploaded.
Nothing here is repairable — it is inventoried so a reader verifying a published crate against this
repository is not left to rediscover it, and so the two mechanisms that produced it are named rather
than re-learned.

`main` as the only publish source is now a reaction — `crates/tianheng/tests/publish_source.rs`, reached
through `scripts/publish.sh` (see *Branching and release* in `AGENTS.md`). This record is the state
that reaction was written against.

## Inventory

Audited 2026-08-05 across **all 96 published tarballs** — every version of `xuanji`, `guibiao`,
`hunyi`, `louke`, `tianheng`, and `xingbiao` (which joined at `0.1.6`, so `0.1.0`–`0.1.5` have five
crates rather than six). Within every single version the recorded commit is **identical across all of
its crates**, so the inventory is per version: each release was one publish run from one checkout.

| Version | Recorded in the tarballs | `vX.Y.Z` names | Verdict |
|---|---|---|---|
| `0.1.0` | `48c532fc20e41a5c03d3c91e6c2acdbf9ddc1b90` | `2f903fb5` | orphaned by the history rewrite |
| `0.1.1` | `932377307bd599fd6a390d6f8e012d81929e5e57` | `c2424136` | orphaned by the history rewrite |
| `0.1.2` | `1a4f1024a2a9c2ce0d96994f6ca5b7839712c420` | `ee5f05c8` | orphaned by the history rewrite |
| `0.1.3` | `af6c833bb9986d6ddf9990237b59729b449c45c2` | `b1257a8e` | orphaned by the history rewrite |
| `0.1.4` | `a173be4a68bf4c9b252a8ec78e65c8e1da7bb001` | `44d7beda` | orphaned by the history rewrite |
| `0.1.5` | `7d389dc2e04373ce90e1b045b986411dfa0c3904` | `1d384d36` | orphaned by the history rewrite |
| `0.1.6` | `050ca1788b19c72b44d98e156c6e293be92d4c55` | `120945e6` | orphaned by the history rewrite |
| `0.1.7` | `033feffc778ec29ac5972d7b14ede1827a2e40f4` | `fd48c473` | orphaned by the history rewrite |
| `0.1.8` | `739c5a4e07a71f7004cfe8e8f822aee50459b9d9` | `4cb9360b` | orphaned by the history rewrite |
| `0.1.9` | `e67c55695d7fe651ca09e04d11f8bc26d61d2da8` | `06426e5c` | orphaned by the history rewrite |
| `0.1.10` | `478387f306815d2f9b844b8b76ef6461c01b44e7` | `04fe9a6e` | orphaned by the history rewrite |
| `0.2.0` | `73bf246ad86380f0272b15cacfbbdc0e17096392` | same | **agrees with the tag** |
| `0.2.1` | `1df44f9793f7d643b3191bfee97f3c8b115b87a8` | same | **agrees with the tag** |
| `0.2.2` | `993c82b1d9f017b89b17adbbff8e5a4a45bc3e6b` | `38319262` | orphaned by a later force-push |
| `0.2.3` | `34b114dd4895eed8f30eaad5da2616b5fdac9a98` | same | **agrees with the tag** |
| `0.3.0` | `66e3096e2bb06c360eb350be4641198814b429ef` | same | **agrees with the tag** |
| `0.4.0` | `f1dba52c0281d402f11c4e578ab5cd4eae2a9be8` | `e645a549` | published from the release branch |

## The two mechanisms

They are genuinely different, and only one of them is what the publish-source gate refuses.

### Publishing from somewhere other than the tagged `main` commit — `0.4.0`

`f1dba52` is the tip of `release/0.4.0` (`chore(release): prepare 0.4.0`), not `e645a549`, the
`release: 0.4.0` commit on `main` that `v0.4.0` tags. The publish ran from the release branch's
checkout instead of from `main` after the squash.

**No published content is affected.** The two commits have the same tree hash
(`458ffeb53be919f39fab9c90a0aeea655f32c3c9`), every crate's subtree hash matches, and every shipped
tracked file in all six tarballs is byte-identical to `main` — including each `Cargo.toml.orig`. The
only differences are the three files cargo generates (the normalized `Cargo.toml`, the per-crate
`Cargo.lock`, and `.cargo_vcs_info.json` itself). This is precisely what makes the class easy to miss:
cargo records the **commit**, not the content, so an identical tree does not make a release branch's
tip an acceptable source.

`f1dba52` remains resolvable — `release/0.4.0` is retained on the remote, as every release branch back
to `release/0.2.0` is. It is anchored by no tag, deliberately: enshrining a permanent ref outside
protected `main` would contradict the rule this record exists to support.

This is the class `crates/tianheng/tests/publish_source.rs` refuses. Run against a worktree at `f1dba52` it
exits `1`; against `main` it exits `0`.

### A published release snapshot amended afterwards — `0.2.2`

`993c82b1` is *also* a `release: 0.2.2` commit, its parent `1df44f97` being `main`'s `release: 0.2.1`,
GitHub-verified, committed `2026-07-22T00:41:29Z`. So `0.2.2` **was** published from `main`'s release
snapshot; the source was right at the time. About an hour and forty minutes later `main` was
force-pushed with a sibling snapshot, `38319262` (`2026-07-22T02:22:45Z`, same parent), and `v0.2.2`
tags that one instead. The published pointer was orphaned after the fact, not misdirected.

The two trees differ in exactly **four files**, all inside one since-pruned change directory,
`2026-07-21-cleanup-louke-phantom-tests`, under `openspec/changes/archive/` — the release-baseline
archive pruning. All 225 remaining blobs are identical. None of those four is inside a crate
directory, and
`cargo publish` packages only files within each crate's own directory, so **no published crate content
differs either**.

`993c82b1` is absent from a fresh clone's object store but still resolves through the GitHub API,
which is the only reason this could be reconstructed at all.

**The rule this yields:** once a version is on crates.io, its `release: X.Y.Z` commit is immutable.
Amending or force-pushing it away orphans a pointer that can never be rewritten. The publish-source
gate cannot see this coming — at publish time the check passed — so this half stays a convention, with
a post-hoc audit (comparing each published version's recorded sha against its tag) as the reaction it
would take. That audit is not built: it needs network access and a recorded baseline for every
anomaly above, and no second occurrence has been observed.

### `0.1.0`–`0.1.10` — unknowable

All eleven recorded commits return `422 No commit found` from the GitHub API: they were orphaned by the
2026-07-17 history rewrite and repository re-creation, which re-signed every commit and therefore
changed every sha1. Whether those eleven publishes ran from `main` **can no longer be determined**, and
no artifact or ref survives from which to determine it. They are listed for completeness, not as
findings. Their tags were lightweight and unsigned until 2026-08-05, when all eleven were re-created as
signed annotated tags pointing at their own unchanged commits.

## Reproducing the audit

Read-only, and it re-derives the table above rather than trusting it. For one version:

```bash
crate=xuanji version=0.4.0
curl -sL -o pkg.crate "https://static.crates.io/crates/$crate/$crate-$version.crate"
tar -xOzf pkg.crate "$crate-$version/.cargo_vcs_info.json"  # -> {"git":{"sha1":"…"},"path_in_vcs":…}
git rev-list -n 1 "v$version"                               # the commit the tag names
```

A recorded commit missing from the local object store may still resolve on the remote:
`gh api repos/tacticaldoll/tianheng/commits/<sha1>`. Comparing two trees without fetching either is
`gh api "repos/tacticaldoll/tianheng/git/trees/<tree-sha>?recursive=1"` on both and diffing the
`path`/`sha` pairs — which is how the four-file `0.2.2` difference above was established. Note that the
compare endpoint's `files` array is the wrong tool for two diverged commits: it reports the diff from
their merge base, so it presents everything the release added rather than how the two snapshots differ.
