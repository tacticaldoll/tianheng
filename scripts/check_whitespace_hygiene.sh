#!/usr/bin/env bash
#
# Whitespace hygiene across every tracked text file: no trailing whitespace on any line, no blank
# line at end of file, and a final newline present.
#
# Why a reaction rather than reviewer diligence: `cargo fmt` governs `.rs` only, so nothing in the
# repository checked `.md`, `.toml`, `.sh`, or `.yml` — and three blank lines at EOF reached a
# release branch in the 0.4.0 window through 23 touched spec files and two independent full-range
# adversarial reviews, one of which ran `cargo fmt --all --check` and reported it passing. Neither
# review had anything to consult, because the property was stated nowhere and reacted nowhere. A
# checkable property belongs in a reaction (`PROJECT.md`'s drift law), never in prose or in the
# hope that the next reader notices.
#
# `git diff --check` was the obvious candidate and is the wrong shape for a gate: it answers about a
# diff, so its verdict depends on the base it is given, and a contributor running it locally with no
# argument checks only unstaged work. This asserts the invariant over the repository content itself,
# so the answer does not move with the caller's base.
#
# Files are read from the **worktree**, with a line-ending CR normalized away first. Both halves of
# that are load-bearing, and each rules out the other's failure:
#
#   * Reading the worktree, not the index (`git cat-file blob :path`), because the index holds only
#     what is STAGED. A contributor who adds trailing whitespace and runs this gate would be told
#     `clean` — the gate having read the previous content — and would commit the offence. That is a
#     false negative, the one outcome `PROJECT.md`'s Core Contract forbids outright, and no amount of
#     platform-independence buys it back.
#   * Normalizing `\r$` away, because `\r` is a `[[:space:]]` character and a worktree under
#     `core.autocrlf` holds CRLF. Reading it raw reported every line of every file as trailing
#     whitespace on a Windows checkout (measured: 7 offenses against the same content that yields 3
#     on Linux) — a flood that gets a gate disabled rather than obeyed. The substitution removes
#     exactly the line-terminating CR: `text \r\n` keeps its trailing space, `text\r\r\n` keeps the
#     inner CR, and a CR mid-line is untouched, so every genuine offense survives normalization.
#     The CR is written as a LITERAL byte via `printf`, not as `\r` in the pattern: BSD `sed` does not
#     interpret that escape on the left-hand side. `tr -d '\r'` would be portable and is the wrong
#     tool — it deletes mid-line CRs too, so `text\r\r\n` would stop being an offense at all.
#
# The verdict is therefore a property of the content, not of who checked it out — the same
# platform-independence this window required of every identity label.
#
# Exit 0 clean, 1 violation, 2 cannot judge — the family's own Core Contract, so this script reads
# the same way as the reactions it sits beside.
set -euo pipefail

cd "$(dirname "$0")/.."

# Paths are parsed out of `git ls-files --eol` on its TAB separator. Git *quotes* a path containing
# whitespace or a backslash, which would reach the loop below as a mangled name and be silently
# checked as the wrong file — or not at all. Refuse to judge instead: the one forbidden outcome is a
# file passing without being read.
if git ls-files | grep -q '[[:space:]"\\]'; then
  echo "cannot judge: a tracked path contains whitespace, a quote, or a backslash, which this" >&2
  echo "script's TAB-separated parse of \`git ls-files --eol\` cannot address unambiguously:" >&2
  git ls-files | grep '[[:space:]"\\]' >&2
  exit 2
fi

blob=$(mktemp)
trap 'rm -f "$blob"' EXIT

inspected=0
offenses=0

report() {
  offenses=$((offenses + 1))
  echo "$1"
}

while IFS=$'\t' read -r eol_info path; do
  # Git's own text/binary verdict, not a re-implemented heuristic: `i/-text` is how it reports a
  # blob it treats as binary. Whitespace is not a property of such a file.
  case "$eol_info" in
  i/-text*) continue ;;
  esac

  # A tracked path can be absent from the worktree (a staged deletion). There is no content to
  # judge, so it is neither an offense nor an inspection.
  [ -f "$path" ] || continue

  inspected=$((inspected + 1))

  # See the header: the worktree is the subject, with the line-terminating CR normalized away so the
  # verdict does not depend on the checkout's line endings.
  sed "s/$(printf '\r')\$//" -- "$path" >"$blob"

  # A zero-byte file has no line to be wrong about. (A one-byte `\n` placeholder — what this
  # repository's two `.gitkeep` files actually hold — is not caught either, and correctly so: the
  # blank-line test below fires on a blank line that *content* precedes, and there is none.)
  [ -s "$blob" ] || continue

  while IFS= read -r line_no; do
    report "$path:$line_no: trailing whitespace"
  done < <(grep -n '[[:space:]]$' -- "$blob" | cut -d: -f1)

  # A final newline TERMINATES the last line; a second one begins a blank line that no line
  # terminates. `tail -c 2` distinguishes them without reading the whole file.
  if [ "$(tail -c 2 -- "$blob" | od -An -c | tr -s ' ')" = " \\n \\n" ]; then
    report "$path:$(wc -l <"$blob"): blank line at end of file"
  fi

  # `wc -l` counts newlines, so an unterminated last line is not among them: the offending line is
  # the one after the count. Reporting the count itself named line 0 of a single-line file.
  if [ -n "$(tail -c 1 -- "$blob")" ]; then
    report "$path:$(($(wc -l <"$blob") + 1)): no newline at end of file"
  fi
done < <(git ls-files --eol)

# Vacuity guard: a gate that inspected nothing reports clean, which is the silent pass this whole
# script exists to prevent. It can only mean the parse above stopped matching git's output.
if [ "$inspected" -eq 0 ]; then
  echo "cannot judge: inspected 0 tracked text files — \`git ls-files --eol\` produced no parseable" >&2
  echo "rows, so this gate would report clean without having read anything" >&2
  exit 2
fi

if [ "$offenses" -gt 0 ]; then
  echo >&2
  echo "whitespace hygiene: $offenses offense(s) across $inspected tracked text files" >&2
  echo "remedy: drop the trailing whitespace, or the blank line before EOF, keeping the single" >&2
  echo "newline that terminates the last line" >&2
  exit 1
fi

echo "whitespace hygiene ok ($inspected tracked text files)"
