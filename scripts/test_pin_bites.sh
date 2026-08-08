#!/usr/bin/env bash
#
# The state and failure directions of `check_pin_bites.sh` that this matrix proves, each on a throwaway
# repository. Not all of them: the gate carries refusal paths for a producer that fails mid-read, an absent
# `cargo`, an untracked records file, and a definition scan finding zero or several files, and those are the
# shared family shapes or diagnostics rather than this gate's own subject. One is neither: the target-directory
# isolation is a requirement, and the attempt to pin it — remove the export, point the gate at a pre-warmed
# directory — made cargo rebuild and passed, so nothing here proves it. Naming that here beats an
# `every` this file would not hold — the absolute-claim class this repository keeps closing. Two further
# requirements are unwatched here and were found by removing their guards and watching this matrix stay green:
# that a record's path be TRACKED (only the containment half is pinned, by `escaping`), and that the checkout
# carry a working repository (swapping it for an export of tracked content changes nothing below). Both
# refusals work — each was driven from a fixture by hand — but neither is defended here.
#
# The fixtures are minimal cargo workspaces rather than checkouts of this one, and that is the point of the
# shape: the gate under test builds what it judges, so a fixture carrying this repository's dependency graph
# would make each direction pay a cold workspace build. A single crate with one test and one recognizer holds
# every property exercised BELOW, and each direction runs in about a second — not every property the gate has:
# the target-directory isolation has no direction here, and the two properties about what a killed pin does not
# prove are declared bounds rather than behaviours to exhibit.
#
# Every assertion names the expected exit CODE rather than merely non-zero: this family separates a violation
# (1) from a gate that cannot decide (2), and a matrix blind to that difference cannot defend it — measured in
# the sibling release-coherence gate, where exactly that blindness let a 1-into-2 collapse ride green.
#
# The cannot-judge directions carry the weight here. A gate that reports a mutation as a dead pin when the
# mutation never applied, or never compiled, is a false clean reached through the very reading it exists to
# replace, and each of those is a separate direction below.
set -Eeuo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
check=$script_dir/check_pin_bites.sh

fixture_root=$(mktemp -d)
trap 'rm -rf "$fixture_root"' EXIT

# The recognizer under test in every fixture, and the pin that claims to defend it. `exposes` returns true only
# for a line that both opens with `pub ` and names `dyn `; the pin's bound is that a continuation line carrying
# the second without the first is not recognized. Dropping the `pub ` half closes that bound, so the pin must
# fail — the same shape as this repository's own first record, in miniature.
RECOGNIZER='pub fn exposes(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("pub ") && trimmed.contains("dyn ")
}'

PIN='#[test]
fn a_continuation_line_is_not_recognized() {
    assert!(fixt::exposes("pub fn f() -> Box<dyn T> {"));
    assert!(!fixt::exposes(") -> Box<dyn T> {"));
}'

# A repository the gate can judge: a one-crate workspace, a spec citing the pin, and a record file.
new_repo() {
    local name=$1 records=$2 repo=$fixture_root/$1
    mkdir -p "$repo/crates/fixt/src" "$repo/crates/fixt/tests" "$repo/openspec/specs/fixture" "$repo/scripts/lib"
    printf '[workspace]\nmembers = ["crates/fixt"]\nresolver = "2"\n' >"$repo/Cargo.toml"
    printf '[package]\nname = "fixt"\nversion = "0.0.0"\nedition = "2021"\n' >"$repo/crates/fixt/Cargo.toml"
    printf '%s\n' "$RECOGNIZER" >"$repo/crates/fixt/src/lib.rs"
    printf '%s\n' "$PIN" >"$repo/crates/fixt/tests/pin.rs"
    {
        printf '# fixture Specification\n\n## Requirements\n### Requirement: A recognizer reads one line\n\n'
        printf '#### Scenario: A continuation line is not recognized (bound)\n\n'
        printf -- '- **PINNED-BY** `a_continuation_line_is_not_recognized`\n'
    } >"$repo/openspec/specs/fixture/spec.md"
    printf '%s\n' "${records%$'\n'}" >"$repo/scripts/lib/pin_mutations.tsv"
    git -C "$repo" init -q
    git -C "$repo" add -A
    git -C "$repo" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qm fixture
    printf '%s\n' "$repo"
}

# `from` -> `to` for the record files below, as literal TABs.
record() { printf 'a_continuation_line_is_not_recognized\tcrates/fixt/src/lib.rs\t%s\t%s\n' "$1" "$2"; }

KILLS=$(record 'trimmed.starts_with("pub ") && trimmed.contains("dyn ")' 'trimmed.contains("dyn ")')

expect_pass() {
    local repo=$1 expected=$2 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq 0 ]] \
        || { printf 'expected exit 0, got %d: %s\n' "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected success containing %q, got: %s\n' "$expected" "$output" >&2; exit 1; }
}

expect_fail() {
    local repo=$1 expected_status=$2 expected=$3 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq $expected_status ]] \
        || { printf 'expected exit %d containing %q, got exit %d: %s\n' "$expected_status" "$expected" "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected exit %d containing %q, got: %s\n' "$expected_status" "$expected" "$output" >&2; exit 1; }
}

# --- the passing direction first: a gate that only ever refuses is not a working gate ---

kills=$(new_repo kills "$KILLS")
expect_pass "$kills" 'pin bites ok (1 declared mutations'

# The uncovered remainder is disclosed on a clean run, so a pass cannot be read as every pin exercised.
expect_pass "$kills" 'carry no mutation'

# A clean run must print nothing on stderr. The exit-contract backstop is installed here too, and a trap that
# fires on an ordinary non-zero return would print once per clean run while the code stayed 0 — invisible to
# every check that reads only the code, which is how it went unnoticed in a sibling gate.
clean_stderr=$("$check" "$kills" 2>&1 >/dev/null || true)
[[ -z $clean_stderr ]] \
    || { printf 'a clean run must print nothing on stderr, got: %s\n' "$clean_stderr" >&2; exit 1; }

# The gate mutates a separate checkout, never the repository it was pointed at. A gate that edited tracked
# files and was interrupted between edit and restore would have destroyed work.
#
# The reading is wider than `git status` because the checkout is a WORKTREE of the judged repository and shares
# its common `.git`: a registration left behind, or a ref written from inside the tree under test, is invisible
# to porcelain and to HEAD. Both were produced while this change was under review, so the instrument is widened
# rather than trusted.
untouched=$(new_repo untouched "$KILLS")
# A `post-checkout` hook in the judged repository. The checkout under test shares that repository's common
# directory, so without `core.hooksPath` this fires INSIDE the tree under test with write access to the judged
# repository's refs — measured, it plants a tag that survives the gate's own cleanup. The assertion below sees
# it only because this fixture produces it; a widened instrument observing nothing is not a guard.
mkdir -p "$untouched/.git/hooks"
cat >"$untouched/.git/hooks/post-checkout" <<'HOOK'
#!/bin/sh
git update-ref refs/tags/planted-by-hook HEAD
HOOK
chmod +x "$untouched/.git/hooks/post-checkout"
judged_state() {
    git -C "$1" status --porcelain
    git -C "$1" rev-parse HEAD
    git -C "$1" show-ref || true
    git -C "$1" worktree list
}
before=$(judged_state "$untouched")
"$check" "$untouched" >/dev/null 2>&1
after=$(judged_state "$untouched")
[[ $before == "$after" ]] \
    || { printf 'the gate mutated the repository it judged:\nbefore: %s\nafter: %s\n' "$before" "$after" >&2; exit 1; }

# A tracked path may still be a SYMLINK out of the tree, and following it edits a file the gate has no business
# touching — destructively, if the run is killed between the write and the restore. Tracked-ness alone accepted
# this; containment is resolved.
escaping=$(new_repo escaping "$KILLS")
outside=$fixture_root/outside.txt
# It carries the anchor, so a gate that follows the link actually WRITES here rather than declining for an
# unrelated reason. Without that the direction passes against the unfixed gate and proves nothing.
printf 'AUTHOR CONTENT\ntrimmed.starts_with("pub ") && trimmed.contains("dyn ")\n' >"$outside"
ln -s "$outside" "$escaping/crates/fixt/src/victim_link"
sed -i 's|crates/fixt/src/lib.rs|crates/fixt/src/victim_link|' "$escaping/scripts/lib/pin_mutations.tsv"
git -C "$escaping" add -A
git -C "$escaping" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qm escaping
expect_fail "$escaping" 2 'not a file under the tree under test'
[[ $(head -1 "$outside") == 'AUTHOR CONTENT' && $(grep -c 'starts_with' "$outside") == 1 ]] \
    || { printf 'the gate rewrote a file outside the tree under test\n' >&2; exit 1; }

# --- the violation direction (exit 1) ---

# The class this gate exists for: the pin cannot tell the reaction from a perturbation of it.
inert=$(new_repo inert "$(record 'trimmed.contains("dyn ")' 'trimmed.contains("dyn ")')")
expect_fail "$inert" 1 'occupying the place of a defence'

# A mutation is an assertion about a defence, so one naming a test no bound cites asserts about nothing — and
# its passing would read as coverage of a citation that does not exist.
uncited=$(new_repo uncited "$KILLS")
sed -i 's/^a_continuation_line_is_not_recognized/a_test_no_bound_cites/' "$uncited/scripts/lib/pin_mutations.tsv"
git -C "$uncited" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qam uncited
expect_fail "$uncited" 1 'names a test no declared bound cites'

# --- the cannot-judge direction (exit 2) ---

# An anchor that matches nothing describes a perturbation that was never applied, which is a different fact
# from the pin not biting. Reporting the second for the first is how a rotted record reads as an exercised pin.
absent=$(new_repo absent "$(record 'a substring this file does not carry' 'anything')")
expect_fail "$absent" 2 'occurs 0 times'

# An anchor matching twice names a set rather than a site: substituting the first occurrence perturbs something
# other than what the record describes.
ambiguous=$(new_repo ambiguous "$(record 'let trimmed' 'let trimmed')")
printf '\npub fn twin(line: &str) -> bool {\n    let trimmed = line.trim_start();\n    trimmed.is_empty()\n}\n' \
    >>"$ambiguous/crates/fixt/src/lib.rs"
git -C "$ambiguous" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qam ambiguous
expect_fail "$ambiguous" 2 'occurs 2 times'

# The control run. A cited test that fails for its own reasons would otherwise read as a pin that bites the
# moment a mutation is applied — the `f() == f()` shape, arrived at from the other side.
already_failing=$(new_repo already-failing "$KILLS")
sed -i 's/assert!(fixt::exposes("pub fn f() -> Box<dyn T> {"));/assert!(!fixt::exposes("pub fn f() -> Box<dyn T> {"));/' \
    "$already_failing/crates/fixt/tests/pin.rs"
git -C "$already_failing" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qam already-failing
expect_fail "$already_failing" 2 'does not pass on the unmutated tree'

# An order-dependent pin: it writes a marker and asserts the marker's absence, so it passes once and fails
# ever after. The first control cannot see this — the failure it would need to observe is the one it causes —
# so the mutated run fails for a reason the mutation had no part in and the citation reads as exercised. A
# constructed false clean, closed by running the control again after the restore.
order_dependent=$(new_repo order-dependent "$KILLS")
cat >"$order_dependent/crates/fixt/tests/pin.rs" <<'PIN'
#[test]
fn a_continuation_line_is_not_recognized() {
    let marker = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ran-once.marker");
    assert!(!marker.exists(), "this pin passes only on its first run");
    std::fs::write(&marker, "ran").expect("the manifest directory is writable");
    assert!(fixt::exposes("pub fn f() -> Box<dyn T> {"));
}
PIN
git -C "$order_dependent" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qam order-dependent
expect_fail "$order_dependent" 2 'fails on the restored tree'

# A pin that rewrites its own source on a later run: the mutated run fails for a reason the mutation had no
# part in, and the restored-tree run then matches NOTHING and exits 0 over zero tests. Without the vacuity
# rule on that third run site, the gate read it as the restored tree still passing and reported the citation
# exercised.
self_erasing=$(new_repo self-erasing "$KILLS")
cat >"$self_erasing/crates/fixt/tests/pin.rs" <<'PIN'
#[test]
fn a_continuation_line_is_not_recognized() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let marker = dir.join("ran-once.marker");
    if marker.exists() {
        std::fs::write(dir.join("tests/pin.rs"), "// erased
").expect("the target is writable");
        panic!("a later run, failing for a reason no mutation caused");
    }
    std::fs::write(&marker, "ran").expect("the target is writable");
    assert!(fixt::exposes("pub fn f() -> Box<dyn T> {"));
}
PIN
git -C "$self_erasing" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qam self-erasing
expect_fail "$self_erasing" 2 'did not run exactly one test'

# A cited test defined in a MODULE of an integration target rather than at its root. Falling through to `--lib`
# ran a different test of the same name and reported its death as the citation's — so the fixture carries that
# decoy, generated through a macro so the definition scan cannot see it while the harness still registers it.
# The cited pin is left inert under the mutation, so a gate running the right test reports it surviving.
module_of_a_target=$(new_repo module-of-a-target "$KILLS")
mkdir -p "$module_of_a_target/crates/fixt/tests/suite"
cat >"$module_of_a_target/crates/fixt/tests/suite/mod.rs" <<'PIN'
#[test]
fn a_continuation_line_is_not_recognized() {
    assert!(fixt::exposes("pub fn f() -> Box<dyn T> {"));
}
PIN
printf 'mod suite;\n' >"$module_of_a_target/crates/fixt/tests/pin.rs"
cat >>"$module_of_a_target/crates/fixt/src/lib.rs" <<'DECOY'

macro_rules! decoy {
    ($name:ident) => {
        #[test]
        fn $name() {
            assert!(!crate::exposes(") -> Box<dyn T> {"));
        }
    };
}

#[cfg(test)]
mod tests {
    decoy!(a_continuation_line_is_not_recognized);
}
DECOY
git -C "$module_of_a_target" add -A
git -C "$module_of_a_target" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qm module-of-a-target
expect_fail "$module_of_a_target" 2 'neither an integration target root nor a library source file'

# The same class through a BIN target, which the first repair's denylist left open: a cited test in
# `src/bin/`, and a same-named library test the definition scan cannot see. The selector is an allowlist now,
# so anything it cannot map to the target that registers it refuses instead of running another.
bin_of_a_package=$(new_repo bin-of-a-package "$KILLS")
mkdir -p "$bin_of_a_package/crates/fixt/src/bin"
cat >"$bin_of_a_package/crates/fixt/src/bin/tool.rs" <<'BIN'
fn main() {}

#[cfg(test)]
mod t {
    #[test]
    fn a_continuation_line_is_not_recognized() {
        assert!(fixt::exposes("pub fn f() -> Box<dyn T> {"));
    }
}
BIN
rm "$bin_of_a_package/crates/fixt/tests/pin.rs"
cat >>"$bin_of_a_package/crates/fixt/src/lib.rs" <<'DECOY'

macro_rules! decoy {
    ($name:ident) => {
        #[test]
        fn $name() {
            assert!(!crate::exposes(") -> Box<dyn T> {"));
        }
    };
}

#[cfg(test)]
mod tests {
    decoy!(a_continuation_line_is_not_recognized);
}
DECOY
git -C "$bin_of_a_package" add -A
git -C "$bin_of_a_package" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qm bin-of-a-package
expect_fail "$bin_of_a_package" 2 'neither an integration target root nor a library source file'

# --- a run that executed no test (the scenario three refusal sites implement) ---
#
# A filter matching nothing exits 0 having run nothing, which by status alone is a pin that survived. Both
# halves are exercised: the harness registering the name but running it zero times, and the harness not
# registering it at all.

ignored=$(new_repo ignored "$KILLS")
sed -i 's/^#\[test\]$/#[test]\n#[ignore]/' "$ignored/crates/fixt/tests/pin.rs"
git -C "$ignored" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qam ignored
expect_fail "$ignored" 2 'did not run exactly one test'

unregistered=$(new_repo unregistered "$KILLS")
sed -i 's/^#\[test\]$/#[cfg(feature = "absent")]\n#[test]/' "$unregistered/crates/fixt/tests/pin.rs"
git -C "$unregistered" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qam unregistered
expect_fail "$unregistered" 2 'resolves to 0 registered tests'

# A cited name the harness registers twice does not name the citation either.
ambiguous_name=$(new_repo ambiguous-name "$KILLS")
printf '\nmod twin {\n    #[test]\n    fn a_continuation_line_is_not_recognized() {\n        assert!(fixt::exposes("pub fn f() -> Box<dyn T> {"));\n    }\n}\n' \
    >>"$ambiguous_name/crates/fixt/tests/pin.rs"
git -C "$ambiguous_name" -c user.email=f@f -c user.name=f -c commit.gpgsign=false commit -qam ambiguous-name
expect_fail "$ambiguous_name" 2 'resolves to 2 registered tests'

# A mutation that breaks the build observed nothing. `cargo test` exits non-zero for a compile error too, and
# reading that as a failing-and-therefore-biting pin is the false clean this gate exists to refuse.
uncompilable=$(new_repo uncompilable "$(record 'trimmed.contains("dyn ")' 'trimmed.no_such_method()')")
expect_fail "$uncompilable" 2 'does not compile, so the pin was never exercised'

# Every property of zero mutations holds, and reporting that as conformance is the vacuity direction this
# repository has re-opened most often.
empty=$(new_repo empty '# only prose
')
expect_fail "$empty" 2 'declares no mutation'

# The same file with its last prose line indented by a TAB, which the gate refuses as a malformed record
# rather than reading as prose. This is not a cosmetic variant: it is the shape that defeated the guard above.
# The vacuity count and the record loop were two readers with two splitting rules, and TAB is IFS whitespace —
# so the line was a mutation to one and prose to the other, and a records file with nothing to run exited 0
# saying `pin bites ok`. Whitespace hygiene accepts a leading TAB, so no sibling gate stood between that file
# and a commit. One parser now answers both questions, and the answer here is a refusal.
tab_prose=$(new_repo tab-prose '# only prose
	# indented, and still prose
')
expect_fail "$tab_prose" 2 'a comment must open at column one'

# A literal TAB inside a substring shifts every field after it. Refusing loudly beats splitting silently: the
# record would otherwise describe a perturbation nobody wrote — measured on the unfixed parser, which shifted
# `let`/`trimmed` into the path and anchor fields and applied nonsense. This direction guards the DIAGNOSTIC,
# not the exit class: both parsers reach 2 here, the old one by failing to compile what it had mangled.
literal_tab=$(new_repo literal-tab "$(printf 'a_continuation_line_is_not_recognized\tcrates/fixt/src/lib.rs\tlet\ttrimmed\tx')")
expect_fail "$literal_tab" 2 'TAB'

# --- the remainder is over a population, not over a record count ---

# Two records may name one cited test; nothing forbids it and it is a normal thing to want. Counting records
# against distinct names made the disclosure read `-1 of 1`, which is not a count of anything.
two_records=$(new_repo two-records "$KILLS
$(record '&& trimmed.contains("dyn ")' '|| trimmed.contains("dyn ")')")
expect_pass "$two_records" '2 declared mutations over 1 of 1 distinct cited tests'
expect_pass "$two_records" '0 distinct cited tests carry no mutation'

# The gate judges tracked content, so a directory that is not a worktree is undecidable rather than clean.
plain=$fixture_root/plain
mkdir -p "$plain"
expect_fail "$plain" 2 'is not a git worktree'

printf 'ok pin bites state and failure matrix\n'
