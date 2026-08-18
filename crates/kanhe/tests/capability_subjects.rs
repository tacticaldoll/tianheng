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
    Declared, declaration_offences, join_offences, proposal_capabilities, subject_globs,
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

/// The tracked paths each capability's subject claims, or the read that could not be made.
///
/// A glob whose listing cannot be read is **not** a glob claiming nothing. Swallowing the failure silently
/// shrinks the claimed set, and the filing join downstream then reports a change clean over the very capability
/// whose subject it touched — a false negative in the enforcement floor, arrived at by a read nobody was told
/// had failed. The sibling direction in this file already refuses rather than under-claims; this one now agrees
/// with it instead of diverging by one `if let`.
fn claimed(
    root: &Path,
    specs: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, BTreeSet<String>>, Refusal> {
    let mut claimed = BTreeMap::new();
    for (capability, spec) in specs {
        let mut paths = BTreeSet::new();
        // The same rule this function's own comment states, one step earlier: a bullet that cannot be read
        // is not a bullet claiming nothing. `unwrap_or_default()` collapsed *both* of the reader's non-glob
        // answers into an empty claim, so an unparseable bullet shrank the claimed set exactly as an
        // unresolved glob would have — the case `claimed` already refuses, arrived at one line sooner.
        let globs = match subject_globs(spec) {
            Declared::Globs(globs) => globs,
            // A capability with no `## Subject` claims nothing *here* and is reported by the sibling
            // direction, which is where that fact belongs.
            Declared::Absent => Vec::new(),
            Declared::Unreadable(bullet) => {
                return Err(cannot_judge(format!(
                    "`{capability}` lists the subject bullet `{bullet}`, which cannot be read as one \
                     backticked glob; reading past it would shrink the claimed set by exactly that bullet \
                     and let a change touching this capability's subject read as filed"
                )));
            }
            // The same refusal one level up: reading the first section would shrink the claimed set by every
            // glob the others list, which is the identical hole the bullet arm above closes.
            Declared::SeveralSections(count) => {
                return Err(cannot_judge(format!(
                    "`{capability}` carries {count} `## Subject` sections; reading the first would shrink \
                     the claimed set by every glob the others list, and let a change touching this \
                     capability's subject read as filed"
                )));
            }
        };
        for glob in globs {
            let listing = git(root, &["ls-files", "--", &glob]).map_err(|err| {
                cannot_judge(format!(
                    "`{capability}` declares the subject glob `{glob}` and it could not be resolved ({err}); \
                     an unresolved glob is not a glob claiming nothing, and treating it as one would let a \
                     change touching this capability's subject read as filed"
                ))
            })?;
            paths.extend(lines(&listing));
        }
        claimed.insert(capability.clone(), paths);
    }
    Ok(claimed)
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
        //
        // **Under the declared OpenSpec mode this branch is always taken, and saying so is the point.**
        // `PROJECT.md` records that this project uses OpenSpec's `specs` half and not its `changes` half,
        // so `openspec/changes/` holds nothing but `archive/.gitkeep` and the corpus above can never be
        // non-empty. This direction therefore observes nothing today — it is held for the same optionality
        // the placeholder is, and it starts working the day a change directory exists.
        //
        // The class it was built from is **currently defended by review alone**: a change filing a
        // wrapper's requirement under the wrong capability, which is live because `scripts/publish.sh` has
        // two claimants. That is recorded in `BACKLOG.md` rather than left for a reader to infer from an
        // early return, because an inert reaction reads as coverage — which is the failure this whole file
        // exists to refuse one level up.
        //
        // Re-pointing it at what `specs` mode does produce was considered and rejected: the join compares a
        // proposal's *declared* capability set against the subjects a diff touches, and reading that set
        // from the touched spec paths instead is near-tautological — touching a spec is naming its
        // capability. There is no second, independent declaration to compare against.
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
    let claimed = claimed(&root, &specs).unwrap_or_else(|refusal| {
        panic!("capability subjects (cannot judge): {}", refusal.message)
    });

    let mut offences = Vec::new();
    for proposal_path in changes {
        let change = proposal_path
            .trim_start_matches("openspec/changes/")
            .trim_end_matches("/proposal.md")
            .to_string();
        let proposal = std::fs::read_to_string(root.join(&proposal_path))
            .unwrap_or_else(|err| panic!("cannot read {proposal_path}: {err}"));
        // Several `## Capabilities` sections is a fact about the proposal this reader may not resolve:
        // reading the first drops the capabilities the others name, and the join would then report the
        // change as having accounted for one it never listed.
        let listed = proposal_capabilities(&proposal).unwrap_or_else(|count| {
            panic!(
                "capability subjects (cannot judge): {proposal_path} carries {count} `## Capabilities` \
                 sections, so which one lists what the change touches is decided by file order"
            )
        });
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

/// `repository-checks/files-no-capability-claims-a-stated-bound`
///
/// Subjects are declared where a capability has something to say, and requiring them to tile the repository
/// would buy coverage with a claim per capability nobody could defend. The blindness is declared so a clean report
/// is not read as a complete one — and it is reported rather than left silent.
#[test]
fn files_no_capability_claims_are_reported_rather_than_implied_judged() {
    let Some(root) = workspace_root() else {
        return;
    };
    let specs = specs(&root);
    let claimed = claimed(&root, &specs).unwrap_or_else(|refusal| {
        panic!("capability subjects (cannot judge): {}", refusal.message)
    });
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
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");
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
    let claimed = claimed(&root, &specs(&root)).unwrap_or_else(|refusal| {
        panic!("capability subjects (cannot judge): {}", refusal.message)
    });
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

/// Building the claimed set refuses over a subject it cannot read, rather than claiming less.
///
/// Both of this reader's non-glob answers, because they shrink the claimed set by different amounts and for
/// the same reason: a bullet it cannot parse drops that bullet's paths, and a second `## Subject` section
/// drops every glob the others list. A claimed set quietly one glob short lets a change touching that
/// capability's subject read as filed — which is the whole fact this file exists to decide.
///
/// The three live call sites pass the repository's own specs, which are well formed, so neither branch had
/// ever been reached. `subject_globs` is the shared reader and its own states are exercised beside it; these
/// are this consumer's answers to them.
///
/// Negative run: with each arm replaced by `Vec::new()` — the `unwrap_or_default()` shape this reader's own
/// comment records as the defect it replaced — the claimed set came back one capability short and the
/// direction failed.
#[test]
fn a_subject_the_claimed_set_cannot_read_is_refused_rather_than_shrunk() {
    let root = std::env::temp_dir().join(format!("kanhe-claimed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("create");

    for (name, spec, needle) in [
        (
            "an unreadable bullet",
            "# Probe\n\n## Subject\n\n- not a backticked glob\n",
            "cannot be read as one backticked glob",
        ),
        (
            "two subject sections",
            "# Probe\n\n## Subject\n\n- `crates/a/**`\n\n## Subject\n\n- `crates/b/**`\n",
            "reading the first would shrink",
        ),
    ] {
        let specs: BTreeMap<String, String> = [("probe".to_string(), spec.to_string())]
            .into_iter()
            .collect();
        let refusal = claimed(&root, &specs)
            .expect_err("a subject this reader cannot read is not a subject claiming less");
        assert_eq!(
            refusal.kind,
            Kind::CannotJudge,
            "{name}: {}",
            refusal.message
        );
        assert!(
            refusal.message.contains(needle),
            "{name}: the refusal must name what it could not read, got: {}",
            refusal.message
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}
