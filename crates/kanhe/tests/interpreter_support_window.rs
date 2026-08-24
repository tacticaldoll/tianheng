//! Repository check: the pinned interpreter carries a support window, and the window is still open.
//!
//! **A hand-maintained pin rots, and this one had nothing keeping it from rotting.** `node-version` was
//! `'24'` until it became `'24.16.0'` — an exact pin, which is what the step beside it claims to be, at the
//! cost of a version nobody refreshes. `BACKLOG.md` filed that beside the action SHAs, under an entry whose
//! own promotion trigger reads *a second ecosystem arriving whose pinning would want the same answer*. The
//! Node pin is that second ecosystem, and it was added to the entry that says so.
//!
//! **The two halves of the rot close differently, and only one of them has teeth.** Falling behind *within*
//! major 24 is bounded already: `package.json` declares `">=24 <25"` and `.npmrc` sets `engine-strict=true`,
//! so the interpreter cannot silently leave the major the lock's tree was resolved against. What was open is
//! the other half — running an interpreter past the point anyone ships fixes for it, with nothing here
//! reacting, because the only thing that would notice is someone remembering.
//!
//! **So the pin declares a date and this reads it.** The declaration is a *commitment of this repository* —
//! this tree does not run that major beyond that date — rather than an assertion about Node's schedule. The
//! distinction is the one this repository has been repairing all window: a claim about the world needs
//! something holding it, and nothing offline can hold Node's release calendar; a claim about what this tree
//! will do needs only this file. The date is *chosen* with the schedule in view, and that choice is the one
//! unheld thing left — owned as a decision instead of asserted as a fact.
//!
//! The reading is a pure function of the workflow text and the day, so every direction it refuses in is
//! constructed here rather than waiting for a calendar.

use std::path::PathBuf;

use kanhe::reading;

/// The `engines.node` range the tree's own pin requires, as `package.json` writes it.
///
/// A literal in a direction rather than a read of the file, so a row moves one thing: the live tree is held
/// by `the_pinned_interpreter_is_within_its_declared_support_window`, which reads the real manifest.
const ENGINES_FOR_24: &str = "  \"node\": \">=24 <25\"";

/// The major after `major`, as text — `24` yields `25`.
///
/// A range's upper bound names the successor, so holding `engines.node` against the pin needs it. A
/// non-numeric major yields itself, which the declaration's own two-field check has already refused.
fn major_after(major: &str) -> String {
    major
        .parse::<u32>()
        .map_or_else(|_| major.to_string(), |n| (n + 1).to_string())
}

/// Today in the same units, in UTC.
///
/// UTC rather than local time, so the day this refuses on is the same day everywhere. A bound that fires on
/// different dates for different readers is a bound nobody can reason about.
fn today() -> i64 {
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("a clock behind the epoch cannot be compared with a declared date");
    (since_epoch.as_secs() / 86_400) as i64
}

/// What the workflow's declaration, its pin and the package manifest's range say together — or why they
/// cannot be read.
///
/// **`engines` is the third leg, and nothing read it until now.** The module header rested on *`package.json`
/// declares `">=24 <25"`* and the date refusal tells the operator *the three move together*, while this
/// function saw two. Widening `engines.node` to `">=24"` passed every reaction and let a contributor's local
/// Definition of Done run a different Node major than CI's, silently — the local-versus-CI divergence
/// `require_ci_green` exists for, one file over. A claim one word wider than what reacts to it was standing
/// in this check's own header.
///
/// Every refusal names what to do about the state it met rather than asserting something about Node, for the
/// reason the module header gives.
fn support_window(workflow: &str, engines: &str, today: i64) -> Result<(), String> {
    let declarations: Vec<&str> = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("# NOT-BEYOND:"))
        .collect();
    let declaration = match declarations.as_slice() {
        [one] => one.trim(),
        [] => {
            return Err(
                "the interpreter pin declares no support window: add `# NOT-BEYOND: <major> <YYYY-MM-DD>` \
                 beside `node-version`, naming the major it pins and the date beyond which this repository \
                 does not run it"
                    .into(),
            );
        }
        many => {
            return Err(format!(
                "{} support windows are declared and this reads one, so the others bind nothing. Keep the \
                 declaration beside the pin it bounds",
                many.len()
            ));
        }
    };

    // **The field count is answered by `kanhe::reading`, not by a destructure over survivors.** The three
    // `next()` calls this replaces made no claim about how many fields arrived: they read two and checked
    // that a third was absent, which is the same reading spelled longer. What the shared reader adds is that
    // the count is the refusal — and it is the same reader the date below goes through, so the two cannot
    // disagree about what "this input has the wrong number of parts" means.
    //
    // The reader says what arrived; this says what to write. A generic reader cannot know the form its
    // caller wanted, and `repository-checks` requires the refusal to say what to write.
    let [major, date] = reading::fields::<2>(
        "support window",
        declaration,
        reading::Sep::Whitespace,
    )
    .map_err(|refusal| {
        format!(
            "{}. Write `<major> <YYYY-MM-DD>`, so the major it speaks for is compared with the pin \
             rather than assumed",
            refusal.message
        )
    })?;

    let pins: Vec<&str> = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("node-version:"))
        .map(|value| value.trim().trim_matches(['"', '\'']))
        .collect();
    let [pin] = pins.as_slice() else {
        return Err(format!(
            "{} `node-version` pins are present and a window declared for one major cannot speak for \
             several. Bound each pin where it stands",
            pins.len()
        ));
    };
    let pinned_major = pin.split('.').next().unwrap_or(pin);
    if pinned_major != major {
        return Err(format!(
            "the support window is declared for major `{major}` and the pin is `{pin}`, so the date beside \
             the pin bounds a major this workflow does not run. Move both together, or the window outlives \
             what it was chosen for"
        ));
    }

    // **The date goes through `kanhe::reading` too, and that closes two measured defects.** What stood here
    // was `filter_map(|part| part.parse::<i64>().ok())` followed by a destructure of three and a
    // `1..=12`/`1..=31` range check. Both halves were wrong in the same direction — they accepted a date
    // this reader could not read and then answered for it:
    //
    //   `2028--4-30`  the empty field was dropped, three survivors destructured, read as 2028-04-30
    //   `2028-02-31`  in range, off the calendar, and `days_from_civil` answered for it as 2028-03-02
    //
    // Neither is caught by a wider range or a stricter parse alone, which is why the reader is shared rather
    // than repaired in place: the field count and the calendar are one question asked of one input.
    //
    // **The third leg, compared against the same major the other two agree on.** `engines.node` bounds a
    // range rather than pinning a version, so what is held is that the range admits exactly this major: the
    // lower bound names it and the upper bound names its successor. `.npmrc`'s `engine-strict` then makes npm
    // stop rather than warn, which is what turns that declaration into a reaction instead of advice.
    let expected_engines = format!(">={major} <{}", major_after(major));
    let declared: Vec<&str> = engines
        .lines()
        .filter_map(|line| line.trim().strip_prefix("\"node\":"))
        .map(|value| value.trim().trim_matches([',', '"']))
        .collect();
    let [declared] = declared.as_slice() else {
        return Err(format!(
            "`package.json` declares {} `\"node\"` ranges, and a pin bounded by one range cannot be held \
             against several. Declare it once under `engines`",
            declared.len()
        ));
    };
    if *declared != expected_engines {
        return Err(format!(
            "`package.json` declares `engines.node` as `{declared}` while the workflow pins major `{major}` \
             — the same commitment written twice, and only the workflow's half carries the date. Write \
             `{expected_engines}`, or move the pin, the range and the window together"
        ));
    }

    let window = reading::date("support window's date", date).map_err(|refusal| refusal.message)?;
    if today >= window.days_from_epoch() {
        return Err(format!(
            "this repository declared that it does not run Node major `{major}` beyond {date}, and that date \
             has been reached. Move `node-version` in `.github/workflows/ci.yml`, `engines.node` in \
             `package.json`, and this declaration to a major that is still maintained — the three move \
             together or `npm ci` refuses under `engine-strict`, which is the reaction that will say so next"
        ));
    }
    Ok(())
}

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join(".github/workflows/ci.yml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Every shape the declaration can be in, including the ones that must not react.
///
/// **The date direction is constructed rather than waited for.** A bound whose only demonstration is the
/// calendar reaching it is a bound nobody has seen work, and this one is meant to sit dormant for years.
#[test]
fn the_window_reader_decides_every_shape_of_the_declaration() {
    let workflow = |declaration: &str, pin: &str| {
        format!(
            "        {declaration}\n        uses: actions/setup-node@abc # v7.0.0\n        with:\n          node-version: '{pin}'\n"
        )
    };
    let day = reading::date("the supplied day", "2026-08-24")
        .expect("a real day")
        .days_from_epoch();
    for (declaration, pin, reacts, because) in [
        ("# NOT-BEYOND: 24 2028-04-30", "24.16.0", false, "open"),
        (
            "# NOT-BEYOND: 24 2026-08-24",
            "24.16.0",
            true,
            "the day itself is beyond, not within",
        ),
        (
            "# NOT-BEYOND: 24 2026-08-25",
            "24.16.0",
            false,
            "the day before is within",
        ),
        ("# NOT-BEYOND: 24 2020-01-01", "24.16.0", true, "long past"),
        (
            "# unrelated comment",
            "24.16.0",
            true,
            "no declaration at all",
        ),
        (
            "# NOT-BEYOND: 22 2028-04-30",
            "24.16.0",
            true,
            "the window speaks for a major this does not run",
        ),
        (
            "# NOT-BEYOND: 24",
            "24.16.0",
            true,
            "one field, so the major is assumed rather than compared",
        ),
        (
            "# NOT-BEYOND: 24 2028-04-30 extra",
            "24.16.0",
            true,
            "a third field this reader does not define",
        ),
        (
            "# NOT-BEYOND: 24 2028-13-01",
            "24.16.0",
            true,
            "no such month",
        ),
        (
            "# NOT-BEYOND: 24 2028-02-31",
            "24.16.0",
            true,
            "in range and off the calendar — this was ACCEPTED before, and read as 2028-03-02",
        ),
        (
            "# NOT-BEYOND: 24 2028--4-30",
            "24.16.0",
            true,
            "a doubled delimiter is a fourth field — this was ACCEPTED before, and read as 2028-04-30",
        ),
        (
            "# NOT-BEYOND: 24 2028-4-30",
            "24.16.0",
            true,
            "one date has one spelling here; a two-digit month is the declared form",
        ),
        (
            "# NOT-BEYOND: 24 April 2028",
            "24.16.0",
            true,
            "not the declared shape",
        ),
        (
            "# NOT-BEYOND: 24 2028-04-30",
            "24",
            false,
            "a major-only pin still matches its own major",
        ),
    ] {
        // The engines range is held constant at the one matching the tree's pin, so each row moves exactly
        // one thing. The third leg's own directions are below.
        let reacted = support_window(&workflow(declaration, pin), ENGINES_FOR_24, day).is_err();
        assert_eq!(
            reacted,
            reacts,
            "`{declaration}` against pin `{pin}` should {} — {because}",
            if reacts { "react" } else { "pass" }
        );
    }
    // Two declarations bind nothing between them, so the reader refuses rather than picking one.
    let two = format!(
        "{}{}",
        workflow("# NOT-BEYOND: 24 2028-04-30", "24.16.0"),
        "        # NOT-BEYOND: 24 2030-01-01\n"
    );
    assert!(
        support_window(&two, ENGINES_FOR_24, day).is_err(),
        "two declared windows must refuse, since a reader that takes one leaves the other binding nothing"
    );
}

/// The workflow this repository actually runs.
#[test]
fn the_pinned_interpreter_is_within_its_declared_support_window() {
    let Some(root) = workspace_root() else {
        return;
    };
    let workflow = std::fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("read .github/workflows/ci.yml — the pin this bounds is declared in it");
    let engines = std::fs::read_to_string(root.join("package.json"))
        .expect("read package.json — the third leg of the pin this bounds is declared in it");
    if let Err(refusal) = support_window(&workflow, &engines, today()) {
        panic!("{refusal}");
    }
}

/// The third leg refuses in every direction it can be wrong, and passes in the one it can be right.
///
/// **Constructed rather than waited for, like the date half.** Each row moves the `engines` range while the
/// workflow's pin and window stay at the tree's own values, so a refusal is attributable to the leg under
/// test. The widening that motivated this — `">=24"` with no upper bound — is the first row: it satisfies
/// npm, satisfies `engine-strict`, and lets a local run take Node 25 while CI takes 24.16.0.
#[test]
fn the_engines_range_is_held_against_the_major_the_workflow_pins() {
    let workflow = "        # NOT-BEYOND: 24 2028-04-30\n          node-version: '24.16.0'\n";
    let day = reading::date("the supplied day", "2026-08-24")
        .expect("a real day")
        .days_from_epoch();

    for (engines, reacts, because) in [
        (
            r#"  "node": ">=24 <25""#,
            false,
            "the range admits exactly the pinned major",
        ),
        (
            r#"  "node": ">=24""#,
            true,
            "no upper bound admits 25, which CI does not run",
        ),
        (
            r#"  "node": ">=24 <26""#,
            true,
            "an upper bound one major too high admits the same",
        ),
        (
            r#"  "node": ">=22 <25""#,
            true,
            "a lower bound below the pin admits a major CI does not run",
        ),
        (
            r#"  "node": ">=25 <26""#,
            true,
            "the range names a different major than the pin",
        ),
        (
            r#"  "node": "24""#,
            true,
            "an exact version is not the range this holds",
        ),
        ("", true, "no range declared at all"),
        (
            "  \"node\": \">=24 <25\"\n  \"node\": \">=24 <25\"",
            true,
            "two ranges, and a pin bounded by one cannot be held against several",
        ),
    ] {
        let reacted = support_window(workflow, engines, day).is_err();
        assert_eq!(
            reacted,
            reacts,
            "`{engines}` should {} — {because}",
            if reacts { "react" } else { "pass" }
        );
    }
}
