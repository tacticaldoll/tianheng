# 勘合 / kanhe

**若合符節。** — *As when the two halves of a tally fit.* (孟子·離婁下)

**This repository's record, held against itself.** Not product. Ships in no package.

A 勘合 is one document made in two halves, kept apart, and proven genuine by fitting them back
together. Every reaction here does that: it lays two texts side by side and reports where they
disagree.

- `AGENTS.md`'s Definition of Done against `.github/workflows/ci.yml`.
- `CHANGELOG.md` against the release spine the tree actually carries.
- A spec's declared observation bound against the test cited as its defence.
- A generated document against the generator its own header names.
- A capability's declared subject against what a change's diff actually touched.
- A proposed squash message against the pull request it would record — the one gate that runs at
  merge time, because a merged squash cannot be amended.
- Every refusal site against a run that perturbs it, so a refusal cannot change kind or message
  with nothing noticing.

## Not 校讎, and not one of the 三儀

校讎 already has a referent here: it is one of the **三司** and names the *amendment flow* — the
steward routing, the OpenSpec lifecycle. Taking a word that already has a referent is the misnaming
this crate's siblings exist to end, and a first draft of this crate did exactly that. 勘合 belongs
to neither vocabulary: it measures nothing, so it is no 儀, and it administers nothing, so it is no
司.

Its sibling [`shengmo`](../shengmo/README.md) (繩墨) holds the other half: the law 天衡 declares
over itself, and the reactions that run the delivered product against this workspace. Keeping the
two apart is the point — a claim about one was read as a claim about both for as long as they shared
a directory.

## Layout

- `src/` — the judgements: what a squash message must be, what a release section must say, where a
  refusal may be constructed.
- `src/tests/` — their failure matrices, beside what they test.
- `tests/` — the reactions that run against the real repository.

## License

Licensed under either of Apache-2.0 or MIT, at your option — the same terms as the family whose
record it collates, though this crate is published nowhere.
