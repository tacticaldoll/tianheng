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

/// Days from 1970-01-01 to `y-m-d`, proleptic Gregorian, by Howard Hinnant's civil-calendar algorithm.
///
/// Arithmetic rather than a dependency: this crate takes `serde_json` for cargo's message stream and nothing
/// else, and a date library would be a dependency added for one comparison. The algorithm is closed-form and
/// exercised below on the boundaries that make it wrong when transcribed carelessly — a leap day, a century
/// that is not a leap year, and the epoch itself.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let year_of_era = y - era * 400;
    let shifted_month = (m + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + d - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
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

/// What the workflow's declaration and its pin say together, or why they cannot be read.
///
/// Every refusal names what to do about the state it met rather than asserting something about Node, for the
/// reason the module header gives.
fn support_window(workflow: &str, today: i64) -> Result<(), String> {
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

    let parts: Vec<i64> = date
        .split('-')
        .filter_map(|part| part.parse::<i64>().ok())
        .collect();
    let [year, month, day] = parts.as_slice() else {
        return Err(format!(
            "the support window's date reads `{date}` and this expects `YYYY-MM-DD`"
        ));
    };
    if !(1..=12).contains(month) || !(1..=31).contains(day) {
        return Err(format!(
            "the support window's date reads `{date}`, which names no day"
        ));
    }
    if today >= days_from_civil(*year, *month, *day) {
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

/// The date arithmetic, on the values that make a careless transcription wrong.
#[test]
fn the_calendar_arithmetic_holds_at_its_awkward_days() {
    for (y, m, d, expected) in [
        (1970, 1, 1, 0),
        (1970, 1, 2, 1),
        (1972, 2, 29, 789),
        (2000, 2, 29, 11016),
        (2100, 3, 1, 47541),
        (2028, 4, 30, 21304),
    ] {
        assert_eq!(
            days_from_civil(y, m, d),
            expected,
            "{y}-{m}-{d} is not {expected} days from the epoch"
        );
    }
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
    let day = days_from_civil(2026, 8, 24);
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
        let reacted = support_window(&workflow(declaration, pin), day).is_err();
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
        support_window(&two, day).is_err(),
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
    if let Err(refusal) = support_window(&workflow, today()) {
        panic!("{refusal}");
    }
}
