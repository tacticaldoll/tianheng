## 1. Reproduce both over-reactions

- [x] 1.1 A legal subject carrying `!` in its summary is refused for want of a `BREAKING CHANGE:` footer.
- [x] 1.2 A self-contained bullet body is refused as a bare commit list.

## 2. The breaking marker is the head's

- [x] 2.1 Read it from the text before `": "`, reusing what the shape check already does.
- [x] 2.2 Both directions: a summary `!` accepted, a head `!` still requiring its footer.

## 3. A bare commit list is what the commits say

- [x] 3.1 `judge` takes the pull request's commit subjects; a body is a bare list when every bullet is one.
- [x] 3.2 No subjects supplied is a cannot-judge, not a fallback to the shape.
- [ ] 3.3 `scripts/merge-pr.sh` supplies them from `git log`, since the API truncates a headline at 69
  characters. Observe the wrapper still accept a real message.
- [x] 3.4 Directions for all three: GitHub's default refused, a terse bullet body accepted, no subjects
  refused as cannot-judge.

## 4. Record and land

- [x] 4.1 `refusal_bites`: new sites defended; census updated where declared.
- [x] 4.2 `CHANGELOG.md` under `[Unreleased]`. No version bump.
- [ ] 4.3 Full Definition of Done.
- [ ] 4.4 Sync, archive, and land as one squash PR through `scripts/merge-pr.sh` — which now supplies its own
  commit subjects.
