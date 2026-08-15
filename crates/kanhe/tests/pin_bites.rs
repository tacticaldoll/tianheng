//! Repository check: a pinning citation is held to **biting**, not only to running.
//!
//! `bound_register.rs` decides that a `PINNED-BY` citation names a test the harness registers. It cannot
//! decide that the test would fail if the behavior it defends changed — measured rather than argued: replacing
//! a cited pin's entire body with a binding that asserts nothing left the suite green and the register
//! reporting its citation count clean. This check runs each cited test against a tree where the defended behavior
//! has been perturbed and requires it to fail, because whether a test bites is a question about running a
//! program and no reading of text answers it.
//!
//! Coverage is partial and this says so on every clean run, in the shape `docs/observation-bounds.md` already
//! leads with its unpinned count: reporting only the mutations it ran would be the reads-as-coverage failure
//! it exists to end, one level up.
//!
//! **It is gated behind `TIANHENG_PIN_BITES`** and named in the Definition of Done and in CI on its own line.
//! It checks out a worktree and builds it, so running it inside every `cargo test --workspace` would make the
//! ordinary suite pay for it; leaving it to run only when something remembers to would be worse, which is why
//! it is a line of its own rather than a default-ignored test.
//!
//! **The three-way exit contract does not survive the move to Rust.** A shell gate separated a violation (1)
//! from a gate that cannot decide (2); a test passes or fails. Every cannot-judge condition here therefore
//! **fails**, loudly and saying so — the safe direction, because the alternative is a check that reports
//! clean over a perturbation it never applied.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use kanhe::bound_register_parse::citations_in;

const RECORDS: &str = "crates/kanhe/tests/fixtures/pin_mutations.tsv";

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join(RECORDS).is_file(),
        shengmo::workspace::marker_set(),
    )
}

fn run(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new(args[0])
        .args(&args[1..])
        .current_dir(dir)
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .unwrap_or_else(|err| panic!("cannot run {args:?}: {err}"));
    (
        out.status.code(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn must(dir: &Path, what: &str, args: &[&str]) -> String {
    let (code, output) = run(dir, args);
    assert_eq!(
        code,
        Some(0),
        "{what} failed; a failed read is not an empty result: {output}"
    );
    output
}

/// One declared mutation: the perturbation a pinning citation must die under.
struct Record {
    name: String,
    file: String,
    from: String,
    to: String,
}

/// `\n` and `\t` are unescaped in the two substrings, so a perturbation spanning lines is still one record.
fn unescape(s: &str) -> String {
    s.replace("\\n", "\n").replace("\\t", "\t")
}

/// Parse the records **once**, by one rule.
///
/// Counting them by one splitting rule and processing them by another is how a file holding nothing to run
/// once exited 0: TAB is IFS whitespace, so a TAB-indented comment was a record to one reader and prose to the
/// other. One parser, exact tab-count splitting.
fn parse_records(root: &Path) -> Vec<Record> {
    let text = must(
        root,
        &format!("`git show HEAD:{RECORDS}`"),
        &["git", "show", &format!("HEAD:{RECORDS}")],
    );
    let mut records = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            4,
            "a record in {RECORDS} carries {} TAB(s) where a record is four TAB-separated fields:\n{line}",
            fields.len() - 1
        );
        assert!(
            !fields[0].is_empty() && !fields[1].is_empty() && !fields[2].is_empty(),
            "a record carries an empty test name, path, or anchor:\n{line}"
        );
        records.push(Record {
            name: fields[0].to_string(),
            file: fields[1].to_string(),
            from: unescape(fields[2]),
            to: unescape(fields[3]),
        });
    }
    assert!(
        !records.is_empty(),
        "{RECORDS} declares no mutation; every property of zero mutations holds, and reporting that as a \
         clean run is the vacuity direction"
    );
    records
}

/// Every test name a `PINNED-BY` line cites anywhere in a tracked spec, read from HEAD rather than the
/// worktree, mapped to the bound id(s) it defends where it defends any.
///
/// `bound_register_parse::citations_in` (the same canonical scanner `pinning_citations` uses for the
/// worktree — a second hand-written recognizer of the same `#### Scenario:`/`- **PINNED-BY**` grammar is
/// exactly the twin-drift class this repository keeps closing) already resolves a citation to the bound it
/// defends **wherever the citation appears**, not only under a scenario `marks_a_bound` accepts: an ordinary
/// requirement scenario can carry the identical citation line to name the test that verifies IT, and this
/// check exists to hold *a pinning citation*, not only a registered bound's, to biting. A name found only
/// under an ordinary scenario is entered with an empty id list rather than dropped, so
/// [`every_declared_mutation_kills_the_pin_it_names`] still recognizes it as a real citation instead of
/// reporting — as it once did, when this read only the bound-scoped subset — that no declared bound cites a
/// test HEAD plainly cites. Read from `HEAD` per spec file rather than the worktree (`citations_in` takes
/// already-read text precisely so a caller can hand it either), matching this check's own discipline
/// everywhere else.
fn cited_bounds(root: &Path) -> HashMap<String, Vec<String>> {
    let listing = must(
        root,
        "`git ls-files -- openspec/specs`",
        &["git", "ls-files", "--", "openspec/specs"],
    );
    let mut by_name: HashMap<String, Vec<String>> = HashMap::new();
    for path in listing.lines().filter(|p| p.ends_with("/spec.md")) {
        let capability = path
            .strip_prefix("openspec/specs/")
            .and_then(|rest| rest.strip_suffix("/spec.md"))
            .unwrap_or(path);
        let text = must(
            root,
            &format!("`git show HEAD:{path}`"),
            &["git", "show", &format!("HEAD:{path}")],
        );
        for citation in citations_in(capability, path, &text) {
            let short = citation
                .name
                .rsplit("::")
                .next()
                .unwrap_or(&citation.name)
                .to_string();
            let entry = by_name.entry(short).or_default();
            if let Some(id) = citation.bound {
                entry.push(id);
            }
        }
    }
    assert!(
        !by_name.is_empty(),
        "HEAD's specs carry no PINNED-BY citation; a record naming a cited test cannot be judged against an \
         empty set"
    );
    by_name
}

/// Every declared mutation's name resolves to at least one real bound id, not just to "cited".
///
/// This runs on every ordinary `cargo test -p kanhe`, unlike `every_declared_mutation_kills_the_pin_it_names`
/// (gated behind `TIANHENG_PIN_BITES`, since it checks out a worktree and builds it) — `cited_bounds` itself
/// costs only a `git ls-files` and one `git show` per spec, so this regression does not need that gate.
///
/// **`RECORDS` names citations of registered bounds, deliberately, not any `PINNED-BY` line at all.**
/// `cited_bounds` maps a name cited only under an ordinary (non-bound) scenario to an empty id list rather
/// than dropping it — so `every_declared_mutation_kills_the_pin_it_names`'s existence check no longer
/// mistakes a real citation for a fabricated one — but this assertion still refuses it here, by design: a
/// declared mutation is a claim about a bound's defence, and a citation with no bound to defend is not one.
/// Adding a `RECORDS` entry for one of those names still fails, correctly, right here rather than at the
/// existence check above.
#[test]
fn every_declared_mutation_s_name_resolves_to_a_real_bound_id() {
    let Some(root) = workspace_root() else {
        return;
    };
    let records = parse_records(&root);
    let cited = cited_bounds(&root);
    for record in &records {
        let bound_ids = cited.get(&record.name).unwrap_or_else(|| {
            panic!(
                "`{}` is a declared mutation's name but cited_bounds() maps nothing to it",
                record.name
            )
        });
        assert!(
            !bound_ids.is_empty(),
            "`{}` is cited but resolves to no bound id",
            record.name
        );
        assert!(
            bound_ids
                .iter()
                .all(|id| !id.is_empty() && id.contains('/')),
            "`{}` resolves to a malformed bound id {bound_ids:?}; a bound id is `<capability>/<slug>`",
            record.name
        );
    }
}

/// A detached worktree at HEAD, removed when this is dropped.
///
/// Detached and at HEAD, so an interrupted run has edited nothing of the author's — and, unlike an export of
/// tracked content, it carries a working repository, without which a pin that reads the repository through
/// git fails its own control run. Hooks are disabled: a `post-checkout` hook would otherwise run inside the
/// tree under test with write access to the judged repository's refs.
struct Scratch {
    root: PathBuf,
    tree: PathBuf,
    work: PathBuf,
}

impl Scratch {
    fn new(root: &Path) -> Self {
        let work = std::env::temp_dir().join(format!("tianheng-pin-bites-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&work);
        xingbiao::claim_scratch(&work).expect("the scratch root is writable");
        std::fs::create_dir_all(work.join("no-hooks")).expect("the scratch root is writable");
        let tree = work.join("tree");
        must(
            root,
            "checking HEAD out into a scratch worktree",
            &[
                "git",
                "-c",
                &format!("core.hooksPath={}", work.join("no-hooks").display()),
                "worktree",
                "add",
                "--quiet",
                "--detach",
                &tree.display().to_string(),
                "HEAD",
            ],
        );
        Self {
            root: root.to_path_buf(),
            tree,
            work,
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = Command::new("git")
            .args(["worktree", "remove", "--force"])
            .arg(&self.tree)
            .current_dir(&self.root)
            .output();
        let _ = std::fs::remove_dir_all(&self.work);
    }
}

/// Which cargo target runs the cited test, as an **allowlist** rather than a fallthrough.
///
/// Assuming a library test for whatever did not match ran a *different* test of the same name and reported
/// that one's death as the citation's.
fn selector(tree: &Path, name: &str) -> Vec<String> {
    let hits = must(
        tree,
        &format!("locating where `{name}` is defined"),
        &[
            "git",
            "grep",
            "-l",
            "-E",
            &format!("fn {name}[[:space:]]*[(<]"),
            "--",
            "crates/",
        ],
    );
    let defined: Vec<&str> = hits.lines().collect();
    assert_eq!(
        defined.len(),
        1,
        "`{name}` is defined in {} files under crates/; the target to run it in cannot be derived from a set",
        defined.len()
    );
    let path = defined[0];
    let package = path
        .strip_prefix("crates/")
        .and_then(|rest| rest.split('/').next())
        .unwrap_or_else(|| {
            panic!("{path} is not under crates/<package>/, so the package to run `{name}` in is underivable")
        });

    if let Some(stem) = path
        .strip_prefix(&format!("crates/{package}/tests/"))
        .and_then(|rest| rest.strip_suffix(".rs"))
        .filter(|stem| !stem.contains('/'))
    {
        return vec![
            "-p".into(),
            package.into(),
            "--test".into(),
            stem.to_string(),
        ];
    }
    if path.starts_with(&format!("crates/{package}/src/"))
        && !path.starts_with(&format!("crates/{package}/src/bin/"))
    {
        return vec!["-p".into(), package.into(), "--lib".into()];
    }
    panic!(
        "`{name}` is defined in {path}, which is neither an integration target root nor a library source \
         file; the target to run it in is not derivable, and guessing one would run a different test"
    );
}

/// `passed + failed == 1` on the one `test result:` line — a filter matching nothing exits 0 over zero tests.
fn ran_exactly_one(log: &str) -> bool {
    log.lines()
        .filter_map(|line| line.strip_prefix("test result: "))
        .filter_map(|rest| {
            let (passed, tail) = rest.split_once(" passed; ")?;
            let passed: usize = passed.rsplit(' ').next()?.parse().ok()?;
            let failed: usize = tail.split(' ').next()?.parse().ok()?;
            Some(passed + failed)
        })
        .next()
        == Some(1)
}

fn cargo_args<'a>(selector: &'a [String], tail: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["cargo", "test", "--all-features"];
    args.extend(selector.iter().map(String::as_str));
    args.extend_from_slice(tail);
    args
}

/// The name `--exact` needs, which is the registered path rather than the bare identifier.
///
/// A cited test may live inside a module, so its registered name is `dsl::tests::<name>`; filtering on the
/// bare identifier matches nothing and exits 0 over zero tests. The harness is asked which registered name
/// the citation means, and exactly one must answer — a filter matching several does not name the citation.
fn resolve_test_name(tree: &Path, selector: &[String], name: &str) -> String {
    let log = must(
        tree,
        &format!("enumerating the tests of the target defining `{name}`"),
        &cargo_args(selector, &["--", "--list"]),
    );
    let listed: Vec<&str> = log
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .filter(|registered| *registered == name || registered.ends_with(&format!("::{name}")))
        .collect();
    assert_eq!(
        listed.len(),
        1,
        "`{name}` resolves to {} registered tests in that target; a filter matching none runs nothing and \
         exits 0, and one matching several does not name the citation",
        listed.len()
    );
    listed[0].to_string()
}

#[test]
fn every_declared_mutation_kills_the_pin_it_names() {
    let Some(root) = workspace_root() else {
        return;
    };
    if std::env::var_os("TIANHENG_PIN_BITES").is_none() {
        eprintln!(
            "pin bites: skipped — set TIANHENG_PIN_BITES=1 to run it. It is named on its own line in the \
             Definition of Done and in CI, so skipping here is a cost decision rather than a hole."
        );
        return;
    }

    let records = parse_records(&root);
    let cited = cited_bounds(&root);
    let scratch = Scratch::new(&root);
    let tree = scratch.tree.clone();

    for record in &records {
        let name = &record.name;
        assert!(
            cited.contains_key(name),
            "the record for `{name}` names a test no declared bound cites; a mutation is an assertion about \
             a citation, and one without a citation asserts nothing"
        );

        // Tracked AND contained, separately. A tracked symlink is tracked, and following one rewrites a file
        // outside the tree — destructively, if the run is killed between the write and the restore.
        let (code, _) = run(
            &root,
            &["git", "ls-files", "--error-unmatch", record.file.as_str()],
        );
        assert_eq!(
            code,
            Some(0),
            "the record for `{name}` names {}, which HEAD does not track; a mutation edits tracked content",
            record.file
        );
        let target = tree.join(&record.file);
        let resolved = std::fs::canonicalize(&target).unwrap_or_else(|err| {
            panic!(
                "the record for `{name}` names {}, which cannot be resolved under the tree under test: {err}",
                record.file
            )
        });
        let tree_real = std::fs::canonicalize(&tree).expect("the scratch tree resolves");
        assert!(
            resolved.starts_with(&tree_real),
            "the record for `{name}` names {}, which resolves to {resolved:?} and so is not a file under \
             the tree under test",
            record.file
        );

        let selector = selector(&tree, name);

        // The control: the unmutated tree must build, and the cited test must pass on it, or its failure
        // under a mutation would say nothing.
        let (code, log) = run(&tree, &cargo_args(&selector, &["--no-run"]));
        assert_eq!(
            code,
            Some(0),
            "the unmutated tree does not build for `{name}`:\n{log}"
        );
        let resolved_name = resolve_test_name(&tree, &selector, name);
        let (code, log) = run(
            &tree,
            &cargo_args(&selector, &["--", "--exact", &resolved_name]),
        );
        assert_eq!(
            code,
            Some(0),
            "`{name}` does not pass on the unmutated tree, so its failure under a mutation would say nothing:\n{log}"
        );
        assert!(
            ran_exactly_one(&log),
            "the control run for `{name}` did not run exactly one test; a filter matching nothing exits 0 \
             over nothing:\n{log}"
        );

        // Apply the mutation. The anchor must match EXACTLY once: an anchor matching twice names a set rather
        // than a site, and substituting the first occurrence would perturb somewhere nobody declared.
        let original = std::fs::read_to_string(&resolved).expect("the record's file is readable");
        let occurrences = original.matches(record.from.as_str()).count();
        assert_eq!(
            occurrences, 1,
            "the record for `{name}` has an anchor matching {occurrences} times in {}; a perturbation that \
             was never applied is a different fact from a pin that does not bite",
            record.file
        );
        std::fs::write(&resolved, original.replace(&record.from, &record.to))
            .expect("the mutation is writable");

        let (build, build_log) = run(&tree, &cargo_args(&selector, &["--no-run"]));
        let survived = if build != Some(0) {
            std::fs::write(&resolved, &original).expect("the restore is writable");
            panic!(
                "the mutation for `{name}` does not compile, so the perturbation was never applied — a \
                 different fact from a pin that does not bite:\n{build_log}"
            );
        } else {
            let (code, log) = run(
                &tree,
                &cargo_args(&selector, &["--", "--exact", &resolved_name]),
            );
            let one = ran_exactly_one(&log);
            std::fs::write(&resolved, &original).expect("the restore is writable");
            assert!(
                one,
                "the mutated run for `{name}` did not run exactly one test:\n{log}"
            );
            code == Some(0)
        };

        let bound_ids = cited.get(name).cloned().unwrap_or_default();
        assert!(
            !survived,
            "`{name}` passes against the mutation declared for it in {} (replacing `{}` with `{}`), so the \
             citation defends nothing: {} can change at that point and the pin will not notice",
            record.file,
            record.from,
            record.to,
            bound_ids.join(", ")
        );

        // Where the mutated run failed, the control runs AGAIN after the restore. One control rules out a
        // test that fails on its own; it does not rule out one whose failure the control itself caused — a
        // pin writing a marker and asserting its absence passes exactly once.
        let (code, log) = run(
            &tree,
            &cargo_args(&selector, &["--", "--exact", &resolved_name]),
        );
        assert_eq!(
            code,
            Some(0),
            "`{name}` fails on the restored tree, so the mutated run's failure may have had nothing to do \
             with the mutation:\n{log}"
        );
        assert!(
            ran_exactly_one(&log),
            "the restored-tree run for `{name}` did not run exactly one test:\n{log}"
        );
    }

    eprintln!(
        "pin bites ok ({} declared mutation(s) covering {} of {} cited test(s)) — the uncovered remainder is \
         the point: a gate reporting only the mutations it ran would be the reads-as-coverage failure it \
         exists to end, one level up",
        records.len(),
        records
            .iter()
            .map(|r| r.name.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        cited.len()
    );
}
