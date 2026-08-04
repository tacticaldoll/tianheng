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

inspected=0
offenses=0

while IFS= read -r file; do
  inspected=$((inspected + 1))
  crate_dir=""
  case "$file" in
  crates/*) crate_dir="crates/$(printf '%s' "$file" | cut -d/ -f2)" ;;
  esac

  # Trailing `.`/`,`/`)`/`` ` `` are prose punctuation, not part of the path. A reference containing a
  # glob is a pattern rather than a location and is not resolvable by existence.
  while IFS= read -r reference; do
    [ -n "$reference" ] || continue
    case "$reference" in
    *'*'*) continue ;;
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
  done < <(grep -oE '(crates|scripts|openspec|docs|examples|\.github)/[A-Za-z0-9_./*-]+|(^|[^A-Za-z0-9_/-])tests/[A-Za-z0-9_/-]+\.rs' -- "$file" 2>/dev/null |
    sed -E 's/^[^A-Za-z0-9_.]+//; s/[.,)`]+$//' | sort -u)
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
