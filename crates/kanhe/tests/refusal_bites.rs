//! Self-governance reaction: every refusal **site** is distinguished by some direction, in both of its
//! contracts.
//!
//! `rust-repository-reactions` requires a reaction's directions to *assert which outcome a shape produces*.
//! Nothing held that requirement. A review sweep at `22ec98e` counted 24 of 60 construction sites as
//! surviving both a kind swap and a message replacement, in front of `cargo publish`, which is irreversible
//! and where the kind is what an operator acts on. That figure conflates two facts: a perturbation kills
//! nothing both when no direction distinguishes the site and when no direction reaches it. Separating them is
//! this reaction's first job, and on the current tree it measures **zero undistinguished** among reached
//! sites.
//!
//! A site carries **two independent contracts**. Its kind is what an operator acts on; its message is what
//! tells them where to look. Both are perturbed, and some direction must die under each. The message
//! perturbation **replaces** rather than prefixes, which is what finds *shadowing* — two sites producing one
//! needle, where no assertion can say which fired.
//!
//! Every site falls into exactly one of five classes and three of them fail, so there is no category left for
//! a coverage report to absorb:
//!
//! | site | verdict |
//! |---|---|
//! | reached, both perturbations kill a direction | defended |
//! | reached, some perturbation kills nothing | **undistinguished** |
//! | never reached, declared out of reach | declared, and counted in the residual |
//! | never reached, not declared | **unreachable and unclaimed** |
//! | declared out of reach, but reached | **stale exemption** |
//!
//! **It is gated behind `TIANHENG_REFUSAL_BITES`** and named on its own line in the Definition of Done and in
//! CI. It runs every judged test binary once per site per perturbation; nothing is rebuilt between runs, but
//! the runs themselves are not free.
//!
//! A target's failure is read as *the site was distinguished*. A panic from the instrument is not that, and a
//! sweep that could not tell them apart would report a site as defended on the strength of its own
//! malfunction — a false negative. Every instrument panic carries a marker, and a run carrying it is refused
//! rather than concluded from.

use kanhe::refusal;

use kanhe::refusal_sites as sites;

use kanhe::refusal_exemptions as exemptions;

use exemptions::Exemption;
use sites::{Corpus, Site, Target};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// What one run of a judged target came to.
#[derive(Debug, PartialEq, Eq)]
enum Run {
    /// Every direction passed.
    Green,
    /// Some direction failed — under a perturbation, that is the site being distinguished.
    Died,
    /// The instrument itself failed, which is not the same fact and must never be read as the site being
    /// distinguished.
    Instrument(String),
}

/// Run a built test binary directly.
///
/// Directly rather than through `cargo test`: nothing is being rebuilt between perturbations, and cargo's own
/// overhead would be paid roughly a hundred times.
fn run_target(root: &Path, target: &Target, mutant: Option<&str>, record: Option<&Path>) -> Run {
    let mut command = Command::new(&target.executable);
    command
        .current_dir(root)
        .env_remove(refusal::MUTANT)
        .env_remove(refusal::RECORD)
        // This reaction is itself a judged target — it compiles the shared vocabulary — so a child that
        // inherited the gate variable would run this sweep again, and again. The child is asked for its
        // directions, never for its own sweep.
        .env_remove("TIANHENG_REFUSAL_BITES")
        // A gate asked for only where it can answer must not be asked for here.
        .env_remove("TIANHENG_PUBLISH_SOURCE")
        .env("TIANHENG_WORKSPACE_TESTS", "1");
    if let Some(mutant) = mutant {
        command.env(refusal::MUTANT, mutant);
    }
    if let Some(record) = record {
        command.env(refusal::RECORD, record);
    }
    let out = command.output().unwrap_or_else(|err| {
        panic!(
            "cannot run the built test binary {}: {err}",
            target.executable.display()
        )
    });
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if log.contains(refusal::INSTRUMENT_PANIC) {
        return Run::Instrument(log);
    }
    if out.status.success() {
        Run::Green
    } else {
        Run::Died
    }
}

/// Whether a recorded line names a site: a path, a colon, a line number, and nothing else.
fn parses_as_a_site(line: &str) -> bool {
    line.rsplit_once(':')
        .is_some_and(|(file, number)| !file.is_empty() && number.parse::<u32>().is_ok())
}

/// The sites each judged target actually constructs, per target.
///
/// Per target rather than merged, because the controls below need a target that **reaches** a site, not one
/// that merely compiles one. Measured, not assumed: `publish_source_integrity` compiles 25 sites and
/// constructs none — it exercises only the accepted path — so poisoning every site in it changes nothing and
/// a control aimed there fails for a reason that has nothing to do with the injection.
fn reached(root: &Path, corpus: &Corpus) -> Vec<(String, BTreeSet<String>)> {
    let scratch =
        std::env::temp_dir().join(format!("tianheng-refusal-reach-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("the scratch root is writable");

    let mut per_target = Vec::new();
    for target in corpus.judged() {
        // One file per target run, so no two processes interleave their appends — and **created here**, by
        // the parent, so that a later read failing means something went wrong rather than meaning the child
        // constructed nothing. Reading it with a default on error was the hole: an unreadable record is
        // indistinguishable from an empty one, and an empty one is a legal answer.
        let record = scratch.join(format!("{}.reach", target.name));
        std::fs::write(&record, "")
            .expect("the record file is creatable before the run that appends to it");
        match run_target(root, target, None, Some(&record)) {
            Run::Green => {}
            Run::Instrument(log) => panic!(
                "the instrument failed while recording {}'s reach, so what it constructs cannot be read:\n{log}",
                target.name
            ),
            Run::Died => panic!(
                "{} does not pass unperturbed, so no failure under a perturbation could be attributed to one",
                target.name
            ),
        }
        let text = std::fs::read_to_string(&record).unwrap_or_else(|err| {
            panic!(
                "the reach record for {} cannot be read ({err}); this file was created before the run, so a \
                 failed read is a lost record rather than a run that constructed nothing — and a lost record \
                 is not self-announcing, since a site it drops looks legally unreached",
                target.name
            )
        });
        let mut seen = BTreeSet::new();
        for line in text.lines() {
            assert!(
                parses_as_a_site(line),
                "the reach record for {} carries the unparseable line {line:?}; a lost or malformed record is \
                 not self-announcing, because a site that declares itself out of reach then looks legally \
                 unreached and the run reports clean",
                target.name
            );
            seen.insert(line.to_string());
        }
        per_target.push((target.name.clone(), seen));
    }
    let _ = std::fs::remove_dir_all(&scratch);
    per_target
}

/// Where a site stands. Five classes, and three of them fail — so there is no category left over.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    Defended,
    Undistinguished(Vec<&'static str>),
    UnreachedAndUnclaimed,
    /// Unreached and declared out of reach: accepted, and counted in the residual.
    DeclaredOutOfReach,
    /// Declared out of reach and reached anyway. The declaration is stale, and retiring it is decided by
    /// re-running the observation it claimed was impossible rather than by the sentence still reading well.
    StaleExemption,
}

/// The classification, as a function of produced inputs, so it can be shown a case rather than trusted.
fn classify(reached: bool, declared: bool, killed_by: &[&'static str]) -> Verdict {
    match (reached, declared) {
        (true, true) => Verdict::StaleExemption,
        (false, true) => Verdict::DeclaredOutOfReach,
        (false, false) => Verdict::UnreachedAndUnclaimed,
        (true, false) => {
            let missing: Vec<&'static str> = ["kind", "message"]
                .into_iter()
                .filter(|mode| !killed_by.contains(mode))
                .collect();
            if missing.is_empty() {
                Verdict::Defended
            } else {
                Verdict::Undistinguished(missing)
            }
        }
    }
}

/// Every edge of the slug ↔ registry ↔ bound join, required in both directions.
///
/// One direction alone is not enough anywhere. A registry entry naming a slug no site carries is a dead
/// exemption; a site whose slug no entry covers is an unexcused one; and the registry-to-bound edge is a
/// **biconditional**, because with one exemption-class bound "registry non-empty implies bound declared"
/// alone lets the last exemption disappear while the bound survives as permanent residue — a declared false
/// negative over a set with no members, which reads as a limit the reaction still has.
fn join_offences(
    sites: &[Site],
    registry: &[Exemption],
    declared_bounds: &BTreeSet<String>,
) -> Vec<String> {
    let mut offences = Vec::new();
    let mut carried: BTreeSet<&str> = BTreeSet::new();
    for site in sites.iter().filter(|s| s.declares_out_of_reach()) {
        let slug = site.slug.as_deref().unwrap_or_default();
        if !carried.insert(slug) {
            offences.push(format!(
                "  {}:{} carries the slug {slug:?}, which another site also carries; an exemption naming a \
                 set rather than a site excuses whichever member happened to be looked at",
                site.file, site.line
            ));
        }
        if !registry.iter().any(|e| e.slug == slug) {
            offences.push(format!(
                "  {}:{} declares itself out of reach under {slug:?}, which no exemption entry covers",
                site.file, site.line
            ));
        }
    }
    for entry in registry {
        if !carried.contains(entry.slug) {
            offences.push(format!(
                "  the exemption {:?} names no site; a dead exemption outlives the thing it excused",
                entry.slug
            ));
        }
        if !declared_bounds.contains(entry.bound) {
            offences.push(format!(
                "  the exemption {:?} names the bound {:?}, which the live declaration set does not hold",
                entry.slug, entry.bound
            ));
        }
    }
    // The other half of the biconditional.
    let bound_is_declared = declared_bounds.contains(exemptions::OUT_OF_REACH_BOUND);
    if registry.is_empty() && bound_is_declared {
        offences.push(format!(
            "  no site is declared out of reach while {:?} is still declared; a false negative over a set \
             with no members reads as a limit this reaction still has",
            exemptions::OUT_OF_REACH_BOUND
        ));
    }
    if !registry.is_empty() && !bound_is_declared {
        offences.push(format!(
            "  {} site(s) are declared out of reach while {:?} is not declared; the exemptions rest on a \
             bound that does not exist",
            registry.len(),
            exemptions::OUT_OF_REACH_BOUND
        ));
    }
    offences
}

/// The names of the targets a run recorded as constructing one site.
fn reached_by(per_target: &[(String, BTreeSet<String>)], key: &str) -> BTreeSet<String> {
    per_target
        .iter()
        .filter(|(_, sites)| sites.contains(key))
        .map(|(name, _)| name.clone())
        .collect()
}

/// Which perturbations of this site kill some direction.
///
/// Only the targets **measured** to construct this site are run. Compiling a file is not reaching a site in
/// it, and since a library's sources belong to every target of its package, `observers` names each of them —
/// perturbing one that never constructed the site cannot kill anything, and the unperturbed recording already
/// says which did. Sound in both directions: a target that constructed the site takes the same path under a
/// perturbation, and one that did not cannot be made to by swapping a kind or replacing a message.
fn perturbations_that_kill(
    root: &Path,
    corpus: &Corpus,
    site: &Site,
    reached_by: &BTreeSet<String>,
) -> Vec<&'static str> {
    let compiled = corpus.observers(&site.file);
    assert!(
        !compiled.is_empty(),
        "no target compiles {}, yet a run recorded constructing a refusal there; the corpus and the run \
         disagree about what was built",
        site.file
    );
    let observers: Vec<_> = compiled
        .into_iter()
        .filter(|target| reached_by.contains(&target.name))
        .collect();
    assert!(
        !observers.is_empty(),
        "a run recorded constructing a refusal at {}, yet no target is recorded as having reached it; the \
         recording and the classification disagree about the same run",
        site.key()
    );
    let mut killed = Vec::new();
    for mode in ["kind", "message"] {
        let selector = format!("{}:{}:{mode}", site.file, site.line);
        for target in &observers {
            match run_target(root, target, Some(&selector), None) {
                Run::Died => {
                    killed.push(mode);
                    break;
                }
                Run::Green => {}
                Run::Instrument(log) => panic!(
                    "the instrument failed under {selector} in {}, so this run says nothing about the site — \
                     reading it as the site being distinguished would be a false negative:\n{log}",
                    target.name
                ),
            }
        }
    }
    killed
}

#[test]
fn every_refusal_site_is_distinguished_in_both_its_contracts() {
    let Some(root) = workspace_root() else {
        return;
    };
    if std::env::var_os("TIANHENG_REFUSAL_BITES").is_none() {
        eprintln!(
            "refusal bites: skipped — set TIANHENG_REFUSAL_BITES=1 to run it. It is named on its own line in \
             the Definition of Done and in CI, so skipping here is a cost decision rather than a hole."
        );
        return;
    }

    let corpus = sites::build(&root);
    assert!(
        corpus.offences.is_empty(),
        "the enumeration this reaction rests on is not sound:\n{}",
        corpus.offences.join("\n")
    );
    assert!(
        !corpus.sites.is_empty(),
        "no refusal site was enumerated; every property of zero sites holds, and reporting that as a clean \
         run is the vacuity direction"
    );
    assert!(
        !corpus.judged().is_empty(),
        "no target compiles the shared refusal vocabulary, so nothing could be perturbed"
    );

    // The recording pass doubles as the first control: every judged target must pass unperturbed, or a
    // failure under a perturbation could not be attributed to one.
    let per_target = reached(&root, &corpus);
    let seen: BTreeSet<String> = per_target
        .iter()
        .flat_map(|(_, sites)| sites.iter().cloned())
        .collect();

    // The injection controls, on a target measured to **reach** the most sites — not merely to compile one.
    let probe_name = per_target
        .iter()
        .max_by_key(|(_, sites)| sites.len())
        .filter(|(_, sites)| !sites.is_empty())
        .map(|(name, _)| name.clone())
        .expect("some judged target constructs a refusal, or there is nothing to perturb");
    let probe = corpus
        .targets
        .iter()
        .find(|t| t.name == probe_name)
        .expect("the probe was chosen from the built targets");
    assert_eq!(
        run_target(&root, probe, Some("ALL:kind"), None),
        Run::Died,
        "poisoning every site in {probe_name} changed nothing, so the injection is not reached at all and \
         every per-site verdict below would be vacuous"
    );
    assert_eq!(
        run_target(
            &root,
            probe,
            Some(&format!("{}:999999:kind", sites::SHARED)),
            None
        ),
        Run::Green,
        "a selector naming no site made {probe_name} fail, so the poison fires where it was not aimed and no \
         per-site attribution holds"
    );

    // The `#[track_caller]` chain, checked against what a real run recorded rather than against a fixture.
    // Were the location read inside the shared module's own helper, every construction would report a line
    // in that module and the sweep would enumerate 58 sites while intercepting one — reporting clean over
    // the rest. A run that recorded anything must therefore have recorded it somewhere else.
    let inside_the_shared_module = seen
        .iter()
        .filter(|key| key.starts_with(&format!("{}:", sites::SHARED)))
        .count();
    assert!(
        !seen.is_empty() && inside_the_shared_module == 0,
        "{} of {} recorded constructions report a location inside {}, so the caller location is being read \
         outside the #[track_caller] chain and every site looks like one site",
        inside_the_shared_module,
        seen.len(),
        sites::SHARED
    );

    let enumerated: BTreeSet<String> = corpus.sites.iter().map(Site::key).collect();
    let unenumerated: Vec<&String> = seen.difference(&enumerated).collect();
    assert!(
        unenumerated.is_empty(),
        "a run constructed refusals the enumeration does not name, so the scan is missing sites: {unenumerated:?}"
    );

    let registry = exemptions::exemptions();
    let declared_bounds: BTreeSet<String> = tianheng::observation_bounds()
        .iter()
        .map(|bound| bound.id().as_str().to_string())
        .collect();
    let join = join_offences(&corpus.sites, &registry, &declared_bounds);
    assert!(
        join.is_empty(),
        "the join between a site's slug, the exemption covering it, and the bound that bound rests on is \
         broken:\n{}",
        join.join("\n")
    );

    let mut undistinguished = Vec::new();
    let mut unclaimed = Vec::new();
    let mut stale = Vec::new();
    let mut defended = 0usize;
    let mut declared = 0usize;
    for site in &corpus.sites {
        let is_reached = seen.contains(&site.key());
        let killed = if is_reached && !site.declares_out_of_reach() {
            perturbations_that_kill(&root, &corpus, site, &reached_by(&per_target, &site.key()))
        } else {
            Vec::new()
        };
        match classify(is_reached, site.declares_out_of_reach(), &killed) {
            Verdict::Defended => defended += 1,
            Verdict::DeclaredOutOfReach => declared += 1,
            Verdict::Undistinguished(missing) => undistinguished.push(format!(
                "  {}:{} `{}` — no direction dies when its {} is perturbed",
                site.file,
                site.line,
                site.constructor,
                missing.join(" or its ")
            )),
            Verdict::UnreachedAndUnclaimed => unclaimed.push(format!(
                "  {}:{} `{}` — no direction constructs it at all",
                site.file, site.line, site.constructor
            )),
            Verdict::StaleExemption => stale.push(format!(
                "  {}:{} declares itself out of reach under {:?}, and a run reached it",
                site.file,
                site.line,
                site.slug.as_deref().unwrap_or_default()
            )),
        }
    }

    eprintln!(
        "refusal sites: {} enumerated, {defended} defended, {declared} declared out of reach, {} \
         undistinguished, {} unreached and unclaimed, {} stale",
        corpus.sites.len(),
        undistinguished.len(),
        unclaimed.len(),
        stale.len()
    );
    assert!(
        undistinguished.is_empty() && unclaimed.is_empty() && stale.is_empty(),
        "a refusal can change kind or message with nothing noticing:\n{}\n{}\n{}",
        undistinguished.join("\n"),
        unclaimed.join("\n"),
        stale.join("\n")
    );
}

#[cfg(test)]
mod records {
    use super::*;

    /// A malformed record line is refused rather than absorbed.
    ///
    /// Its own defect is a parser that accepts anything, which this catches in one call: a lost or garbled
    /// record makes a site look legally unreached, and for a site that declares itself out of reach that
    /// reads as a clean run.
    #[test]
    fn a_record_line_that_names_no_site_is_refused() {
        assert!(parses_as_a_site("crates/a/tests/support/gate.rs:12"));
        for malformed in [
            "",
            "no-colon-at-all",
            "crates/a.rs:",
            "crates/a.rs:twelve",
            ":12",
        ] {
            assert!(
                !parses_as_a_site(malformed),
                "{malformed:?} was accepted as a recorded site"
            );
        }
    }
}

#[cfg(test)]
mod classification {
    use super::*;

    /// The classifier's own pair: it must name the undefended case **and** not name the defended one.
    ///
    /// A classifier that always answers the same way passes one of these and fails the other. Run against its
    /// own defect rather than against a blanket perturbation, because disabling the injection would leave
    /// this untouched while making the "injection is wired" control fail — one negative run that only some
    /// guards notice reports the rest as exercised when nothing tested them.
    #[test]
    fn the_classifier_names_the_undefended_case_and_only_it() {
        assert_eq!(
            classify(true, false, &["kind", "message"]),
            Verdict::Defended
        );
        assert_eq!(
            classify(true, false, &["kind"]),
            Verdict::Undistinguished(vec!["message"])
        );
        assert_eq!(
            classify(true, false, &["message"]),
            Verdict::Undistinguished(vec!["kind"])
        );
        assert_eq!(
            classify(true, false, &[]),
            Verdict::Undistinguished(vec!["kind", "message"])
        );
        assert_eq!(classify(false, false, &[]), Verdict::UnreachedAndUnclaimed);
        assert_eq!(classify(false, true, &[]), Verdict::DeclaredOutOfReach);
        assert_eq!(classify(true, true, &[]), Verdict::StaleExemption);
    }

    /// A site killed by one perturbation is not defended.
    ///
    /// The kind and the message are independent contracts; accepting either would let a site be observed in
    /// one and rot in the other — a message that has become a sentence about something else, or a kind that
    /// has silently inverted, with the suite green.
    #[test]
    fn one_perturbation_is_not_enough() {
        for killed in [vec!["kind"], vec!["message"]] {
            assert_ne!(
                classify(true, false, &killed),
                Verdict::Defended,
                "a site distinguished only by its {killed:?} was accepted as defended"
            );
        }
    }
}

/// `rust-repository-reactions/whether-a-declared-out-of-reach-refusal-is-genuinely-unconstructible-is-not-observed-a-stated-bound`
///
/// `OutOfReach`, owned by the engine. A declaration says *no environment the suite runs in can produce this
/// precondition*; the reaction can only see that no direction reached the site. The two agree today and the
/// reaction cannot tell them apart, so a declaration whose reason has quietly become false still passes.
///
/// The direction below shows exactly that: a site that a fixture *could* construct, declared out of reach and
/// simply never constructed, is accepted. Reaching further would mean building the environment the
/// declaration calls unbuildable, which is the reason this is a bound and not a check.
#[test]
fn a_site_declared_out_of_reach_is_only_observed_to_be_unreached() {
    assert_eq!(
        classify(false, true, &[]),
        Verdict::DeclaredOutOfReach,
        "an unreached declaration is accepted on the strength of not being reached"
    );
    // The one half that *is* observed: a declaration the run contradicts.
    assert_eq!(
        classify(true, true, &[]),
        Verdict::StaleExemption,
        "a declaration a run contradicts must fail; that half is observed, and it is the only half"
    );
    // What is not observed: nothing here distinguishes a precondition that is genuinely unbuildable from one
    // nobody has built yet. Both are `false, true`, and both are accepted.
    let genuinely_unbuildable = classify(false, true, &[]);
    let merely_unbuilt = classify(false, true, &[]);
    assert_eq!(
        genuinely_unbuildable, merely_unbuilt,
        "the two are one value, which is the bound: the reason a declaration gives is prose the reaction \
         never reads"
    );
}

/// `rust-repository-reactions/a-refusal-vocabulary-under-different-names-is-not-observed-a-stated-bound`
///
/// `OutOfReach`, owned by the engine. The scan recognises the shared vocabulary's exact names. A reaction
/// declaring the same contract as `Decision { Disagrees, Unreadable }` carries every property this family
/// cares about and matches none of the needles — and widening the scan toward intent is a judgement over
/// source, the instrument this repository has measured and rejected. The reaction's own sources are exempt for
/// the same reason in reverse: a scan over text holds the text it recognises.
#[test]
fn a_refusal_vocabulary_under_other_names_is_not_observed() {
    let probe = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/refusal_scan/a_vocabulary_under_other_names.rs.txt"),
    )
    .expect("the probe is readable");
    let (found, offences) = sites::scan_for_tests("some/other_gate.rs", &probe, false, true);
    assert!(
        offences.is_empty(),
        "a contract declared under other names was refused, so this bound has been closed and its \
         declaration is stale: {offences:?}"
    );
    assert!(
        found.is_empty(),
        "a contract declared under other names produced sites, which the sweep could not perturb: {found:?}"
    );
}
