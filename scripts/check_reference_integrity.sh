#!/usr/bin/env bash
#
# Every in-repository path a document or a source comment points at must exist.
#
# Why a reaction rather than a sweep: this class has now been fixed by hand twice. `docs: sweep the
# repository's documents for drift against the code` (e741ed6) corrected it across `*.md` and did not
# extend to `*.rs` comments, and a module split landing after that sweep reintroduced it in both — five
# `.rs` comments and two documents pointing at files that had been renamed or split away. A sweep is a
# snapshot; the next rename re-opens what it closed. The reason a reader is harmed is small and
# cumulative: they grep for a named file, find nothing, and cannot tell whether the reference is stale or
# their checkout is wrong.
#
# Scope, and why each bound is drawn where it is:
#
#   * `*.md` and `*.rs` only. A shell script legitimately CONSTRUCTS paths that do not exist in the
#     repository — `scripts/test_example_suite.sh` does `mkdir -p "$WORK/examples/alpha"` — so scanning
#     scripts would report a synthetic fixture as a stale reference.
#   * A `crates/<name>/…` reference is checked only when `<name>` is a real workspace member. Docs and
#     tests are full of deliberately illustrative packages (`crates/a/src/lib.rs`, `crates/foo/src`,
#     `crates/shared/src/thing.rs`) which are not meant to exist. This rule needs no hand-written
#     allowlist of them and maintains itself in the direction that matters: a crate genuinely added under
#     one of those names starts being checked the moment it becomes a member.
#   * Existence is judged against **tracked content**, never against the working directory. A filesystem
#     check makes the verdict depend on local untracked state: this gate's first version passed locally
#     and failed in CI on three references — `.github/prompts/`, `.github/skills/`, and
#     `examples/capability-catalog/Cargo.lock` — which exist in a developer's clone and in no fresh
#     checkout. That is the same checkout-dependence class the identity labels in this window were fixed
#     for, and it would have made this gate's own answer unreproducible.
#   * A reference git **ignores** is skipped, because such prose is usually *about* the path being
#     absent. All three of the above are `.gitignore` entries, and the sentences citing them say so
#     ("Stopped tracking …", "Ignored …"). Asking git rather than listing them keeps the rule honest as
#     `.gitignore` changes.
#   * Root-level references are covered by two unambiguous forms, because a BARE filename in prose is
#     usually generic rather than a location: `Cargo.toml` almost always means "some package's
#     manifest", `lib.rs` means "a crate root", `spec.md` means "the capability's spec", and
#     `README.md` names one of fourteen. So:
#       - a **markdown link target** (`](path)`) is checked unconditionally, being a link by syntax;
#       - a **bare filename** is checked only when that basename is tracked at the repository root and
#         nowhere else, which is what makes it unambiguous (`PROJECT.md`, `BACKLOG.md`, `deny.toml` — but
#         not `Cargo.toml`, `README.md`, or `spec.md`). This also skips names that are illustrative by
#         construction: README's own code samples write `AGENTS.my-project-law.md` into the *adopter's*
#         repository, and it is correctly not expected here.
#     Neither form catches a bare reference to a document that has been renamed AWAY (its basename is
#     then tracked nowhere, so the second rule declines to judge it). The governance-document assertion
#     below is what catches that, and it is a REQUIRED set rather than an allowlist: an allowlist of
#     exceptions rots silently, a required set fails loudly the moment reality diverges from it.
#   * A bare `tests/…rs` reference is satisfied if that file exists under ANY member's `tests/`, not
#     only the referring member's. Such a reference is genuinely cross-crate in practice — 圭表's crate
#     doc points at "`tianheng` workspace `tests/self_governance.rs`", naming the crate in prose rather
#     than in the path — so resolving it against the referring member alone reports a reference a reader
#     can follow perfectly well. The weaker rule still catches the whole class this gate is for: a file
#     that was renamed or split away exists under no member at all.
#
# Exit 0 clean, 1 violation, 2 cannot judge — the family's own Core Contract.
set -euo pipefail

cd "$(dirname "$0")/.."

members=""
for dir in crates/*/; do
  [ -f "${dir}Cargo.toml" ] || continue
  members="$members $(basename "$dir")"
done
if [ -z "$members" ]; then
  echo "cannot judge: found no workspace member under crates/, so the illustrative-vs-real rule below" >&2
  echo "would skip every crates/ reference and this gate would report clean without checking any" >&2
  exit 2
fi

is_member() {
  case " $members " in
  *" $1 "*) return 0 ;;
  *) return 1 ;;
  esac
}

# The reference forms this gate recognizes:
#   1. a path under one of the repository's own top-level directories;
#   2. a bare `tests/…rs`, resolved against the members (see the header);
#   3. a markdown link target — `](path)` — which is a link by syntax, so never generic;
#   4. a bare filename, admitted only when unambiguous (see `is_unambiguous_basename`).
REFERENCE_PATTERN='(crates|scripts|openspec|docs|examples|\.github)/[A-Za-z0-9_./*-]+|(^|[^A-Za-z0-9_/-])tests/[A-Za-z0-9_/-]+\.rs|\]\([A-Za-z0-9_.#/-]+\)|[A-Za-z0-9_][A-Za-z0-9_.-]*\.(md|toml|sh|yml|lock)'

# The repository's governance surface, as a REQUIRED set: each of these must be tracked. This is the
# rename detector the bare-name rule cannot be — rename `PROJECT.md` and its basename is tracked
# nowhere, so a reference to it becomes unjudgeable rather than wrong. Asserting the documents
# themselves exist turns that silence into a loud failure. A required set is safe to write down where
# an allowlist is not: this one fails the moment it goes stale, so it cannot quietly excuse anything.
GOVERNANCE_DOCUMENTS='AGENTS.md AGENTS.self-law.md BACKLOG.md CHANGELOG.md COOKBOOK.md PROJECT.md README.md Cargo.toml deny.toml'

# Every tracked path, plus each of their ancestor directories — a directory is not itself a git object,
# so `docs/` must be recognized through the files under it.
tracked=$(mktemp)
trap 'rm -f "$tracked"' EXIT
git ls-files | awk -F/ '{
  print
  path = ""
  for (i = 1; i < NF; i++) { path = path $i; print path; path = path "/" }
}' | sort -u >"$tracked"

# Present in the repository as committed. Deliberately NOT a filesystem test: see the header.
is_tracked() {
  grep -qxF -- "$1" "$tracked"
}

# A bare filename names one location only if exactly one tracked path ends in it, and that path IS it —
# i.e. it lives at the repository root and nowhere else. `PROJECT.md` and `deny.toml` qualify;
# `Cargo.toml` (root plus every crate), `README.md` (root plus fourteen), `lib.rs`, and `spec.md` do not,
# and a bare mention of those means the generic thing, not the root file.
is_unambiguous_basename() {
  [ "$(grep -cxF -- "$1" "$tracked")" = 1 ] && [ "$(grep -cE "(^|/)$(printf '%s' "$1" | sed 's/[.[\*^$]/\\&/g')\$" "$tracked")" = 1 ]
}

for document in $GOVERNANCE_DOCUMENTS; do
  is_tracked "$document" || {
    echo "cannot judge: '$document' is named as one of this repository's governance documents and is" >&2
    echo "not tracked. If it was renamed, update GOVERNANCE_DOCUMENTS in this script and every prose" >&2
    echo "reference to it; a bare reference to a renamed-away document is unjudgeable, which is exactly" >&2
    echo "why this set is asserted rather than inferred." >&2
    exit 2
  }
done

inspected=0
offenses=0

while IFS= read -r file; do
  inspected=$((inspected + 1))
  crate_dir=""
  case "$file" in
  crates/*) crate_dir="crates/$(printf '%s' "$file" | cut -d/ -f2)" ;;
  esac

  # `grep` exit 1 means "no match" and exit >1 means "could not read". Discarding stderr and running
  # inside a process substitution hid the second entirely: `set -e` cannot see it there, the file still
  # counted as inspected, and an unreadable tracked file left the run reporting clean. Captured here so
  # the two are distinguishable, and the unreadable case refuses to judge.
  status=0
  matches=$(grep -oE "$REFERENCE_PATTERN" -- "$file" 2>/dev/null) || status=$?
  if [ "$status" -gt 1 ]; then
    echo "cannot judge: cannot read tracked file '$file' (grep exit $status) — a file this gate claims" >&2
    echo "to cover must not be counted as inspected without having been read" >&2
    exit 2
  fi

  # Trailing `.`/`,`/`)`/`` ` `` are prose punctuation, not part of the path. A reference containing a
  # glob is a pattern rather than a location and is not resolvable by existence.
  while IFS= read -r reference; do
    [ -n "$reference" ] || continue
    case "$reference" in
    *'*'*) continue ;;
    esac

    # A markdown link target (marked \x01 during extraction) is resolved relative to the REFERRING
    # FILE's directory, which is what a markdown link means — `../../BACKLOG.md` from
    # `docs/history/x.md` is the root document, not a missing one. Prose paths, by contrast, are written
    # repo-relative throughout this repository. `realpath -m` normalizes `..` without requiring the
    # path to exist, so a genuinely broken link still reports its resolved form.
    from_link=0
    case "$reference" in
    $'\x01'*)
      from_link=1
      reference=${reference#$'\x01'}
      [ -n "$reference" ] || continue
      case "$reference" in
      *:*) continue ;;
      esac
      reference=$(realpath -m --relative-to="$PWD" -- "$(dirname -- "$file")/$reference")
      ;;
    esac

    target="$reference"
    case "$reference" in
    crates/*)
      name=$(printf '%s' "$reference" | cut -d/ -f2)
      is_member "$name" || continue
      ;;
    tests/*)
      # Only meaningful inside a member; elsewhere `tests/…` names nothing locatable.
      [ -n "$crate_dir" ] || continue
      target=""
      for member in $members; do
        if is_tracked "crates/$member/$reference"; then
          target="crates/$member/$reference"
          break
        fi
      done
      if [ -z "$target" ]; then
        offenses=$((offenses + 1))
        echo "$file: references '$reference', which is tracked under no workspace member"
      fi
      continue
      ;;
    */*) ;;
    *)
      # A bare filename in PROSE is generic unless it names exactly one tracked location (see the
      # header). A link target is exempt: it is a link by syntax, so it is never generic — and a
      # relative one resolving to a root-level name (`../../GONE.md`) would otherwise be swallowed by
      # the very ambiguity rule that exists for prose. Measured: without this exemption a broken
      # relative link was reported clean.
      [ "$from_link" = 1 ] || is_unambiguous_basename "$reference" || continue
      ;;
    esac

    # A trailing slash marks prose naming a directory; the tracked set holds directories unslashed.
    target=${target%/}
    if ! is_tracked "$target"; then
      # Prose about a deliberately-ignored artifact is not a stale reference — it is usually *about*
      # that path being absent. Ask git rather than keep a list.
      #
      # The path is passed WITH a trailing slash, and that is load-bearing rather than cosmetic:
      # `git check-ignore` decides whether a path is a directory by looking at the filesystem, so for a
      # directory-only pattern (`.gitignore` here holds `/.github/prompts/`) the bare form answers
      # "ignored" in a clone where the directory happens to exist and "not ignored" in a fresh one —
      # reintroducing, inside the ignore check, the very checkout-dependence this gate was just fixed
      # for. Measured in a fresh clone: bare `.github/prompts` is not ignored, `.github/prompts/` is. The
      # slash form is also correct for file patterns, since a pattern without a trailing slash matches a
      # name whether it is a file or a directory (`examples/*/Cargo.lock/` → ignored), and it does not
      # make an unignored path look ignored (`crates/guibiao/src/lib.rs/` → not ignored).
      if git check-ignore -q -- "$target/" 2>/dev/null; then
        continue
      fi
      offenses=$((offenses + 1))
      echo "$file: references '$reference', which is not tracked in this repository"
    fi
  done < <(printf '%s\n' "$matches" |
    sed -E 's#^\]\((.*)\)$#\x01\1#; s/^[^\x01A-Za-z0-9_.]+//; s/[.,)`]+$//; s/#.*$//' |
    grep -v '^\x01?$' | sort -u)
done < <(git ls-files '*.md' '*.rs')

if [ "$inspected" -eq 0 ]; then
  echo "cannot judge: inspected 0 files — no tracked *.md or *.rs, so this gate would report clean" >&2
  echo "without having read anything" >&2
  exit 2
fi

if [ "$offenses" -gt 0 ]; then
  echo >&2
  echo "reference integrity: $offenses stale in-repository reference(s) across $inspected files" >&2
  echo "remedy: point each at the file that now holds the referenced item, or drop the reference — a" >&2
  echo "reader who greps for a named path and finds nothing cannot tell stale prose from a bad checkout" >&2
  exit 1
fi

echo "reference integrity ok ($inspected tracked .md/.rs files)"
