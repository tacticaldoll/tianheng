#!/usr/bin/env bash
#
# Every state and failure direction of `check_whitespace_hygiene.sh`, each on a throwaway repository.
#
# This gate had no companion matrix, and it is the gate where the shared exit-contract backstop first
# misfired: `errtrace` propagates an `ERR` trap into process substitutions, and this gate reads
# `done < <(grep -n '[[:space:]]$' …)` where `grep` exiting 1 on a clean file is the ORDINARY case. That
# printed "cannot judge" once per clean file while still exiting 0, so every check reading only the exit code
# reported the gate passing. The clean-run assertion below is the one that catches it, which is why the
# absence of this file was worth closing rather than recording.
set -Eeuo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
check=$script_dir/check_whitespace_hygiene.sh

fixture_root=$(mktemp -d)
trap 'rm -rf "$fixture_root"' EXIT

# A repository whose tracked text files are clean. The gate judges the WORKTREE of tracked paths, so an
# untracked fixture file would be invisible to it and every case would pass vacuously.
new_repo() {
    local name=$1 repo
    repo=$fixture_root/$name
    mkdir -p "$repo/docs"
    git init -q "$repo"
    git -C "$repo" config user.name 'Whitespace Hygiene Test'
    git -C "$repo" config user.email 'whitespace-hygiene@example.invalid'
    printf '# Notes\n\nA clean line.\n' >"$repo/docs/notes.md"
    printf 'fn main() {}\n' >"$repo/main.rs"
    git -C "$repo" add -A
    git -C "$repo" commit -qm 'clean fixture'
    printf '%s\n' "$repo"
}

expect_pass() {
    local repo=$1 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq 0 ]] \
        || { printf 'expected exit 0, got %d: %s\n' "$status" "$output" >&2; exit 1; }
    grep -Fq 'whitespace hygiene ok' <<<"$output" \
        || { printf 'expected a clean report, got: %s\n' "$output" >&2; exit 1; }
}

# The expected exit CODE, not merely non-zero: this family separates a violation (1) from a gate that cannot
# decide (2), and collapsing them would report a misconfiguration as a clean refusal.
expect_fail() {
    local repo=$1 expected_status=$2 expected=$3 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq $expected_status ]] \
        || { printf 'expected exit %d containing %q, got exit %d: %s\n' "$expected_status" "$expected" "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected exit %d containing %q, got: %s\n' "$expected_status" "$expected" "$output" >&2; exit 1; }
}

# --- the passing direction first: a gate that only ever refuses is not a working gate ---

clean=$(new_repo clean)
expect_pass "$clean"

# A clean run must also print NOTHING on stderr. This is the assertion the gate lacked when the shared `ERR`
# backstop was installed: it fired inside the process substitution that scans each file for trailing
# whitespace, once per clean file, while the exit code stayed 0. Reading only the code hid it entirely.
clean_stderr=$("$check" "$clean" 2>&1 >/dev/null || true)
[[ -z $clean_stderr ]] \
    || { printf 'a clean run must print nothing on stderr, got: %s\n' "$clean_stderr" >&2; exit 1; }

# --- the three offence directions ---

trailing=$(new_repo trailing)
printf '# Notes\n\nTrailing here.   \n' >"$trailing/docs/notes.md"
git -C "$trailing" add -A
git -C "$trailing" commit -qm 'add trailing whitespace'
expect_fail "$trailing" 1 'trailing whitespace'

blank_eof=$(new_repo blank-eof)
printf '# Notes\n\nA line.\n\n' >"$blank_eof/docs/notes.md"
git -C "$blank_eof" add -A
git -C "$blank_eof" commit -qm 'add a blank line at EOF'
expect_fail "$blank_eof" 1 'blank line at end of file'

no_newline=$(new_repo no-newline)
printf '# Notes\n\nNo final newline.' >"$no_newline/docs/notes.md"
git -C "$no_newline" add -A
git -C "$no_newline" commit -qm 'drop the final newline'
expect_fail "$no_newline" 1 'no newline at end of file'

# --- the cannot-judge directions ---

# A repository with no tracked text file at all: reporting clean there would be the silent pass this gate
# exists to prevent, so it refuses to judge instead.
empty=$fixture_root/empty
mkdir -p "$empty"
git init -q "$empty"
git -C "$empty" config user.name 'Whitespace Hygiene Test'
git -C "$empty" config user.email 'whitespace-hygiene@example.invalid'
git -C "$empty" commit -q --allow-empty -m 'no tracked files'
expect_fail "$empty" 2 'inspected 0 tracked text files'

# And any unhandled failure at all, which is what the shared backstop is for: the sites nobody wrapped.
# `mktemp` is unwrapped and runs before any file is read, so it is the honest injection point. Measured before
# the backstop existed: this gate exited 7 with no output, a status its own header's contract does not define.
mktemp_stub=$fixture_root/mktemp-stub
mkdir -p "$mktemp_stub"
printf '#!/usr/bin/env bash\nexit 7\n' >"$mktemp_stub/mktemp"
chmod +x "$mktemp_stub/mktemp"

unhandled_status=0
unhandled_output=$(PATH="$mktemp_stub:$PATH" "$check" "$clean" 2>&1) || unhandled_status=$?
[[ $unhandled_status -eq 2 ]] \
    || { printf 'an unhandled failure must exit 2, not the utility status, got %d: %s\n' "$unhandled_status" "$unhandled_output" >&2; exit 1; }
grep -Fq 'an unhandled command failed' <<<"$unhandled_output" \
    || { printf 'an unhandled failure must say so and name where, got: %s\n' "$unhandled_output" >&2; exit 1; }

# --- a partial read is not an empty result ---

# The enumeration emits one valid row and THEN fails. Before the shared capture rule this gate consumed it through
# `done < <(git ls-files --eol)`, whose status the parent never sees: it reported `whitespace hygiene ok (1 tracked
# text files)` and exit 0 over a repository it had read one file of. The count fell from two to one in its own
# output and nothing reacted to it.
#
# The vacuity guard cannot cover this direction and that is why this case exists beside it: `inspected -eq 0` was
# built for zero rows, and a partial read gives one or more. They answer different questions.
partial_stub=$fixture_root/partial-stub
mkdir -p "$partial_stub"
cat >"$partial_stub/git" <<STUB
#!/usr/bin/env bash
for argument in "\$@"; do [[ \$argument == --eol ]] && eol=1; done
if [[ -n \$eol ]]; then
    printf 'i/lf    w/lf    attr/                 \tdocs/notes.md\n'
    exit 7
fi
exec $(command -v git) "\$@"
STUB
chmod +x "$partial_stub/git"

partial_status=0
partial_output=$(PATH="$partial_stub:$PATH" "$check" "$clean" 2>&1) || partial_status=$?
[[ $partial_status -eq 2 ]] \
    || { printf 'a producer that failed after emitting a row must exit 2, got %d: %s\n' "$partial_status" "$partial_output" >&2; exit 1; }
grep -Fq 'a failed read is not an empty result' <<<"$partial_output" \
    || { printf 'the refusal must name the partial read rather than fail incidentally, got: %s\n' "$partial_output" >&2; exit 1; }

# --- read-only, like every gate in the family ---

# On a fixture this gate has NOT already judged. Capturing `before` from a repository the gate had run over
# several times was blind by construction: a gate that writes the same file on every run leaves that file in
# `before` too, so the comparison held. Measured, not reasoned — a stray write injected into a sibling gate
# passed its read-only direction unnoticed until the fixture was made fresh.
untouched=$(new_repo untouched)
before_tree=$(git -C "$untouched" status --porcelain=v1 --untracked-files=all)
before_head=$(git -C "$untouched" rev-parse HEAD)
"$check" "$untouched" >/dev/null
[[ $(git -C "$untouched" status --porcelain=v1 --untracked-files=all) == "$before_tree" \
    && $(git -C "$untouched" rev-parse HEAD) == "$before_head" ]] \
    || { printf 'whitespace hygiene check mutated repository state\n' >&2; exit 1; }

printf 'ok whitespace hygiene state and failure matrix\n'
