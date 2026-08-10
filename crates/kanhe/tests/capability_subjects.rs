//! Collation: what a capability says it governs, against what a change actually touched.
//!
//! Which capability a requirement belongs to is decided once, in a proposal, and was checked by nothing. It
//! went wrong twice in one window — a requirement about `scripts/publish.sh` filed under a capability whose
//! subject is repository checks, and a member filled by the one criterion that says where a check must *not*
//! live — and a reader caught both.
//!
//! The touched set is **produced**: the change's diff against its base. Reading it from the change's own
//! prose would compare the capability list against something written by the same decision, which is a
//! comparison of a value with itself.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use kanhe::capability_subjects::{
    declaration_offences, join_offences, proposal_capabilities, subject_globs,
};
use kanhe::refusal::{Kind, Refusal, cannot_judge};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("openspec/specs").is_dir(),
        shengmo::workspace::marker_set(),
    )
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| format!("cannot run git {args:?}: {err}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn lines(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// Every capability spec, keyed by capability.
fn specs(root: &Path) -> BTreeMap<String, String> {
    let listing = git(root, &["ls-files", "openspec/specs/*/spec.md"])
        .expect("the capability specs are enumerable; a failed enumeration is not an empty set");
    let mut specs = BTreeMap::new();
    for path in lines(&listing) {
        let capability = path
            .trim_start_matches("openspec/specs/")
            .trim_end_matches("/spec.md")
            .to_string();
        let text = std::fs::read_to_string(root.join(&path))
            .unwrap_or_else(|err| panic!("cannot read {path}: {err}"));
        specs.insert(capability, text);
    }
    assert!(
        !specs.is_empty(),
        "no capability spec was enumerated, so every property of this check holds while proving nothing"
    );
    specs
}

/// The tracked paths each capability's subject claims.
fn claimed(root: &Path, specs: &BTreeMap<String, String>) -> BTreeMap<String, BTreeSet<String>> {
    let mut claimed = BTreeMap::new();
    for (capability, spec) in specs {
        let mut paths = BTreeSet::new();
        for glob in subject_globs(spec).unwrap_or_default() {
            if let Ok(listing) = git(root, &["ls-files", "--", &glob]) {
                paths.extend(lines(&listing));
            }
        }
        claimed.insert(capability.clone(), paths);
    }
    claimed
}

/// Every capability declares a subject, and every glob it declares resolves.
#[test]
fn every_capability_declares_the_subject_it_governs() {
    let Some(root) = workspace_root() else {
        return;
    };
    let offences = declaration_offences(&specs(&root), |glob| {
        git(&root, &["ls-files", "--", glob]).map(|listing| lines(&listing))
    });
    assert!(
        offences.is_empty(),
        "a capability's declared subject does not hold:\n{}",
        offences
            .iter()
            .map(|refusal| format!("  ({:?}) {}", refusal.kind, refusal.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The base this branch's change is measured against.
///
/// Unresolvable is a **cannot-judge**, never an empty diff: reading it as "nothing was touched" would report
/// clean over every change, which is the direction this check exists to close.
fn base(root: &Path) -> Result<String, Refusal> {
    let mut candidates = vec![];
    if let Ok(upstream) = git(root, &["rev-parse", "--abbrev-ref", "@{upstream}"]) {
        candidates.push(upstream.trim().to_string());
    }
    if let Ok(refs) = git(
        root,
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/remotes/origin/release/*",
            "refs/remotes/origin/main",
        ],
    ) {
        candidates.extend(lines(&refs));
    }
    let mut best: Option<(usize, String)> = None;
    for candidate in candidates {
        let Ok(merge_base) = git(root, &["merge-base", "HEAD", &candidate]) else {
            continue;
        };
        let merge_base = merge_base.trim().to_string();
        let Ok(count) = git(
            root,
            &["rev-list", "--count", &format!("{merge_base}..HEAD")],
        ) else {
            continue;
        };
        let Ok(count) = count.trim().parse::<usize>() else {
            continue;
        };
        if best.as_ref().is_none_or(|(seen, _)| count < *seen) {
            best = Some((count, merge_base));
        }
    }
    best.map(|(_, merge_base)| merge_base).ok_or_else(|| {
        cannot_judge(
            "the base this branch departs from cannot be resolved from its upstream or from the tracked \
             release and main refs, so what the change touches cannot be produced — reading that as an \
             empty diff would report clean over every change",
        )
    })
}

/// A change's proposal names a capability claiming each file the change touches.
#[test]
fn a_change_names_every_capability_whose_subject_it_touches() {
    let Some(root) = workspace_root() else {
        return;
    };
    let changes = git(&root, &["ls-files", "openspec/changes/*/proposal.md"])
        .expect("the active changes are enumerable");
    let changes = lines(&changes);
    if changes.is_empty() {
        // No filing decision is in front of this check. An ordinary checkout is asking no such question,
        // and refusing one would be noise rather than governance.
        return;
    }

    let base = match base(&root) {
        Ok(base) => base,
        Err(refusal) => {
            assert_eq!(refusal.kind, Kind::CannotJudge);
            panic!("{}", refusal.message);
        }
    };
    let diff = git(&root, &["diff", "--name-only", &format!("{base}...HEAD")])
        .expect("the change's diff is readable once its base resolves");
    let touched_all = lines(&diff);

    let specs = specs(&root);
    let claimed = claimed(&root, &specs);

    let mut offences = Vec::new();
    for proposal_path in changes {
        let change = proposal_path
            .trim_start_matches("openspec/changes/")
            .trim_end_matches("/proposal.md")
            .to_string();
        let proposal = std::fs::read_to_string(root.join(&proposal_path))
            .unwrap_or_else(|err| panic!("cannot read {proposal_path}: {err}"));
        let listed = proposal_capabilities(&proposal);
        // The change's own directory is what a proposal is, not what it governs.
        let own = format!("openspec/changes/{change}/");
        let touched: Vec<String> = touched_all
            .iter()
            .filter(|path| !path.starts_with(&own))
            .cloned()
            .collect();
        offences.extend(join_offences(&change, &touched, &listed, &claimed));
    }
    assert!(
        offences.is_empty(),
        "a change touches a capability's subject without naming it:\n{}",
        offences
            .iter()
            .map(|refusal| format!("  {}", refusal.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// `repository-checks/a-tracked-file-no-capability-claims-is-not-judged-a-stated-bound`
///
/// Subjects are declared where a capability has something to say, and requiring them to tile the repository
/// would buy coverage with thirty-six claims nobody could defend. The blindness is declared so a clean report
/// is not read as a complete one — and it is reported rather than left silent.
#[test]
fn files_no_capability_claims_are_reported_rather_than_implied_judged() {
    let Some(root) = workspace_root() else {
        return;
    };
    let specs = specs(&root);
    let claimed = claimed(&root, &specs);
    let every: BTreeSet<String> = claimed.values().flatten().cloned().collect();
    let tracked = lines(&git(&root, &["ls-files"]).expect("the tracked set is enumerable"));
    let unclaimed = tracked.len() - every.len();
    assert!(
        every.len() < tracked.len(),
        "every tracked path is claimed by some capability's subject, which would retire this bound — its \
         scenario should be removed rather than left asserting a blindness that no longer exists"
    );
    eprintln!(
        "capability subjects: {} of {} tracked paths claimed, {unclaimed} unclaimed and therefore unjudged \
         by the filing join",
        every.len(),
        tracked.len()
    );
}

/// A branch whose base cannot be resolved refuses rather than reading as an empty diff.
///
/// Constructed rather than declared: a repository with a commit, no upstream, and no `origin/release/*` or
/// `origin/main` is exactly the shape, and it is one `git init` away.
#[test]
fn a_branch_with_no_resolvable_base_cannot_be_judged() {
    let scratch = std::env::temp_dir().join(format!("kanhe-no-base-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("the scratch root is writable");
    for args in [
        vec!["init", "-q", "."],
        vec![
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "one",
        ],
    ] {
        let out = Command::new("git")
            .args(&args)
            .current_dir(&scratch)
            .output()
            .expect("run git in the scratch repository");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let refusal =
        base(&scratch).expect_err("a repository with no upstream and no origin refs has no base");
    let _ = std::fs::remove_dir_all(&scratch);
    assert_eq!(refusal.kind, Kind::CannotJudge);
    assert!(
        refusal.message.contains("cannot be resolved"),
        "{}",
        refusal.message
    );
}

/// The defect the join was written from, against the **declared** subjects rather than a constructed map.
///
/// The unit matrix exercises the rule; this exercises the rule *on this repository*. Both are needed, and
/// only this one can say whether the claim "it would have caught that filing" is true — measured, under the
/// first rule it was false, because `repository-checks` claims `scripts/*.sh` and naming one claimant
/// was enough.
#[test]
fn the_parked_misfiling_is_refused_against_the_declared_subjects() {
    let Some(root) = workspace_root() else {
        return;
    };
    let claimed = claimed(&root, &specs(&root));
    let wrapper = "scripts/publish.sh".to_string();
    let claimants: Vec<String> = claimed
        .iter()
        .filter(|(_, paths)| paths.contains(&wrapper))
        .map(|(capability, _)| capability.clone())
        .collect();
    assert!(
        claimants.len() > 1,
        "this direction is about overlapping subjects, and `{wrapper}` is claimed by {claimants:?} — with \
         one claimant it would pass for a reason unrelated to the rule"
    );

    let named: BTreeSet<String> = ["repository-checks".to_string()].into_iter().collect();
    let offences = join_offences("a-gate-that-matched-no-test", &[wrapper], &named, &claimed);
    assert!(
        offences
            .iter()
            .any(|refusal| refusal.message.contains("publish-source-integrity")),
        "a change touching the publish wrapper while naming only the repository-check capability must be \
         refused, and named for the capability it failed to account for; got: {:?}",
        offences.iter().map(|r| &r.message).collect::<Vec<_>>()
    );
}
