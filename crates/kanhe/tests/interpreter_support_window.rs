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
//!
//! **The pin is read where it takes effect.** It is the `node-version` input of the one `actions/setup-node`
//! step, found through the workflow's structure rather than by matching `node-version:` at any depth, and the
//! window must be declared inside that same step's lines. A comment is not part of YAML's structure, so the
//! declaration is still read as text; what the structure adds is where it has to stand.

mod support;

use std::path::PathBuf;

use kanhe::reading;
use support::workflow;

/// The `engines.node` range the tree's own pin requires, as `package.json` writes it.
///
/// A literal in a direction rather than a read of the file, so a row moves one thing: the live tree is held
/// by `the_pinned_interpreter_is_within_its_declared_support_window`, which reads the real manifest.
const ENGINES_FOR_24: &str = "{\"engines\": {\"node\": \">=24 <25\"}}";

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
    let parsed = workflow::parse(workflow).map_err(|why| {
        format!("the workflow cannot be read, so its interpreter pin cannot be: {why}")
    })?;
    let setup_node: Vec<&workflow::Step> = parsed
        .jobs
        .iter()
        .flat_map(|job| &job.steps)
        .filter(|step| {
            step.uses
                .as_ref()
                .is_some_and(|uses| uses.value.starts_with("actions/setup-node@"))
        })
        .collect();
    let [step] = setup_node.as_slice() else {
        return Err(format!(
            "{} `actions/setup-node` steps are present, and a window declared beside one pin cannot speak \
             for several. Bound each pin where it stands",
            setup_node.len()
        ));
    };

    let declarations: Vec<(usize, &str, &str)> = workflow
        .lines()
        .enumerate()
        .filter_map(|(index, text)| {
            text.trim()
                .strip_prefix("# NOT-BEYOND:")
                .map(|declared| (index + 1, text, declared))
        })
        .collect();
    // A line spelled as the declaration is a declaration only where the grammar drops it: inside any value — a
    // literal or folded block, a quoted scalar spanning lines — YAML reads it as data.
    for (line, _, _) in &declarations {
        if !workflow::is_comment_line(workflow, *line)? {
            return Err(format!(
                "line {line} is spelled as the support window, and YAML reads it as part of a value rather \
                 than a comment. Keep that text out of values, and declare the window as a comment"
            ));
        }
    }
    let declaration = match declarations.as_slice() {
        [(line, text, one)] if step.holds_comment_at(*line, text) => one.trim(),
        [(line, _, _)] => {
            return Err(format!(
                "the support window is declared on line {line}, outside the `actions/setup-node` step it \
                 bounds — lines {}–{}, at the step's depth or deeper — so nothing ties the date to the pin. \
                 Move it into that step",
                step.first_line, step.last_line
            ));
        }
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
    // **A major is a number, read once.** Compared as text, a word agreeing with itself across the three legs
    // — `x` declared, `x` pinned, `>=x <x` in the manifest — was a major they all agreed on, because the
    // successor fell back to the text it was given. Parsed here, every leg is compared as a number and the
    // successor is computed or refused.
    let major: u32 = major.parse().map_err(|_| {
        format!(
            "the support window's major is `{major}`, which is not a number. Write the major the pin runs, as \
             digits, so it is compared with the pin as a number"
        )
    })?;
    let successor = major.checked_add(1).ok_or_else(|| {
        format!("the support window's major `{major}` has no successor, so no range can be bounded above it")
    })?;

    let Some(pin) = step.with.iter().find(|input| input.key == "node-version") else {
        return Err(
            "the `actions/setup-node` step pins no `node-version`, so the window beside it bounds nothing. \
             Pin the interpreter in that step's `with:`"
                .into(),
        );
    };
    let pin = pin.value.as_str();
    let pinned_major = pin.split_once('.').map_or(pin, |(major, _)| major);
    let pinned_major: u32 = pinned_major.parse().map_err(|_| {
        format!(
            "the pin `{pin}` has no numeric major, so which major the window bounds cannot be compared. Pin \
             a version whose first field is the major"
        )
    })?;
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
    let expected_engines = format!(">={major} <{successor}");
    // **Read as JSON, not as lines.** `engines.node` is a member of one object, and a line reader took any
    // `"node":` in the file — a `metadata` object carrying one satisfied it with `engines` absent. A key written
    // twice is refused rather than resolved, since the range is one declaration.
    let manifest: Strict = serde_json::from_str(engines).map_err(|error| {
        format!("`package.json` cannot be read as the manifest this holds: {error}")
    })?;
    let declared = match manifest.0.get("engines").map(|engines| engines.get("node")) {
        Some(Some(serde_json::Value::String(range))) => range.as_str(),
        Some(Some(other)) => {
            return Err(format!(
                "`package.json` declares `engines.node` as `{other}`, which is not a range. Write it as a string"
            ));
        }
        Some(None) | None => {
            return Err(format!(
                "`package.json` declares no `engines.node`, so the pin's major is held against nothing on the \
                 npm side. Declare `\"engines\": {{\"node\": \"{expected_engines}\"}}`"
            ));
        }
    };
    if declared != expected_engines {
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

/// A JSON document read with every object's keys held unique.
///
/// `serde_json::Value` keeps the last of a key written twice, which would let a manifest carry two ranges and
/// have one of them silently govern; this refuses it instead, naming the key.
struct Strict(serde_json::Value);

impl<'de> serde_core::Deserialize<'de> for Strict {
    fn deserialize<D: serde_core::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(StrictVisitor).map(Strict)
    }
}

struct StrictVisitor;

impl<'de> serde_core::de::Visitor<'de> for StrictVisitor {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(serde_json::Value::Null)
    }

    fn visit_seq<A: serde_core::de::SeqAccess<'de>>(
        self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while let Some(Strict(item)) = seq.next_element()? {
            items.push(item);
        }
        Ok(serde_json::Value::Array(items))
    }

    fn visit_map<A: serde_core::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> Result<Self::Value, A::Error> {
        let mut object = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            let Strict(value) = map.next_value()?;
            if object.insert(key.clone(), value).is_some() {
                return Err(serde_core::de::Error::custom(format!(
                    "`{key}` is written twice in one object, so which value applies is not decidable"
                )));
            }
        }
        Ok(serde_json::Value::Object(object))
    }
}

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join(".github/workflows/ci.yml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// A workflow whose one `actions/setup-node` step carries `declaration` beside the `pin` it bounds.
fn step_pinning(declaration: &str, pin: &str) -> String {
    format!(
        "jobs:\n  j:\n    steps:\n      - name: the validator's interpreter\n        {declaration}\n        uses: actions/setup-node@abc # v7.0.0\n        with:\n          node-version: '{pin}'\n"
    )
}

/// Every shape the declaration can be in, including the ones that must not react.
///
/// **The date direction is constructed rather than waited for.** A bound whose only demonstration is the
/// calendar reaching it is a bound nobody has seen work, and this one is meant to sit dormant for years.
#[test]
fn the_window_reader_decides_every_shape_of_the_declaration() {
    let workflow = |declaration: &str, pin: &str| step_pinning(declaration, pin);
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
    // The window and the pin have to be where the structure says they take effect.
    for (label, document) in [
        (
            "the window outside the step it bounds",
            "# NOT-BEYOND: 24 2028-04-30\njobs:\n  j:\n    steps:\n      - name: node\n        uses: actions/setup-node@abc\n        with:\n          node-version: '24.16.0'\n".to_string(),
        ),
        (
            "the window in a sibling step",
            "jobs:\n  j:\n    steps:\n      - name: other\n        # NOT-BEYOND: 24 2028-04-30\n        run: echo\n      - name: node\n        uses: actions/setup-node@abc\n        with:\n          node-version: '24.16.0'\n".to_string(),
        ),
        (
            "a node-version on another action",
            "jobs:\n  j:\n    steps:\n      - name: other\n        # NOT-BEYOND: 24 2028-04-30\n        uses: some/other-action@abc\n        with:\n          node-version: '24.16.0'\n".to_string(),
        ),
        (
            "the window beside a job key written after steps",
            "jobs:\n  j:\n    steps:\n      - name: node\n        uses: actions/setup-node@abc\n        with:\n          node-version: '24.16.0'\n    # NOT-BEYOND: 24 2028-04-30\n    timeout-minutes: 5\n".to_string(),
        ),
        (
            "declaration-shaped text inside a value of the step",
            "jobs:\n  j:\n    steps:\n      - name: node\n        env:\n          NOTE: |\n            # NOT-BEYOND: 24 2028-04-30\n        uses: actions/setup-node@abc\n        with:\n          node-version: '24.16.0'\n".to_string(),
        ),
        (
            "declaration-shaped text inside a folded value of the step",
            "jobs:\n  j:\n    steps:\n      - name: node\n        env:\n          NOTE: >\n            harmless\n            # NOT-BEYOND: 24 2028-04-30\n        uses: actions/setup-node@abc\n        with:\n          node-version: '24.16.0'\n".to_string(),
        ),
        (
            "a setup-node step with no pin",
            "jobs:\n  j:\n    steps:\n      - name: node\n        # NOT-BEYOND: 24 2028-04-30\n        uses: actions/setup-node@abc\n".to_string(),
        ),
        (
            "two setup-node steps",
            format!("{}{}", step_pinning("# NOT-BEYOND: 24 2028-04-30", "24.16.0"), "      - uses: actions/setup-node@abc\n        with:\n          node-version: '22'\n"),
        ),
    ] {
        assert!(
            support_window(&document, ENGINES_FOR_24, day).is_err(),
            "{label}: must refuse, since the date then binds no pin"
        );
    }
    // A major is a number: a word agreeing with itself across the three legs is not a major the three agree on,
    // and a major with no successor has no range to be bounded by.
    for (major, why) in [
        ("x", "not a number"),
        ("4294967295", "no successor to bound the range"),
    ] {
        let engines = format!("{{\"engines\": {{\"node\": \">={major} <{major}\"}}}}");
        assert!(
            support_window(
                &step_pinning(&format!("# NOT-BEYOND: {major} 2028-04-30"), major),
                &engines,
                day
            )
            .is_err(),
            "a declared major `{major}` must refuse — {why}"
        );
    }
    // `engines.node` is a member of one object, not any `"node"` in the file.
    let elsewhere = "{\n  \"metadata\": {\n    \"node\": \">=24 <25\"\n  }\n}\n";
    assert!(
        support_window(
            &step_pinning("# NOT-BEYOND: 24 2028-04-30", "24.16.0"),
            elsewhere,
            day
        )
        .is_err(),
        "a `node` key outside `engines` is not the range the pin is held against"
    );
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
    let workflow = step_pinning("# NOT-BEYOND: 24 2028-04-30", "24.16.0");
    let day = reading::date("the supplied day", "2026-08-24")
        .expect("a real day")
        .days_from_epoch();

    for (engines, reacts, because) in [
        (
            r#"{"engines": {"node": ">=24 <25"}}"#,
            false,
            "the range admits exactly the pinned major",
        ),
        (
            r#"{"engines": {"node": ">=24"}}"#,
            true,
            "no upper bound admits 25, which CI does not run",
        ),
        (
            r#"{"engines": {"node": ">=24 <26"}}"#,
            true,
            "an upper bound one major too high admits the same",
        ),
        (
            r#"{"engines": {"node": ">=22 <25"}}"#,
            true,
            "a lower bound below the pin admits a major CI does not run",
        ),
        (
            r#"{"engines": {"node": ">=25 <26"}}"#,
            true,
            "the range names a different major than the pin",
        ),
        (
            r#"{"engines": {"node": "24"}}"#,
            true,
            "an exact version is not the range this holds",
        ),
        ("{}", true, "no range declared at all"),
        (
            r#"{"engines": {"node": ">=24 <25", "node": ">=24 <25"}}"#,
            true,
            "two ranges, and a pin bounded by one cannot be held against several",
        ),
        (
            r#"{"engines": {"node": 24}}"#,
            true,
            "a range that is not a string",
        ),
        (
            "{\n  \"metadata\": {\n    \"node\": \">=24 <25\"\n  }\n}\n",
            true,
            "a `node` key outside `engines`",
        ),
        ("not json", true, "a manifest that is not JSON"),
    ] {
        let reacted = support_window(&workflow, engines, day).is_err();
        assert_eq!(
            reacted,
            reacts,
            "`{engines}` should {} — {because}",
            if reacts { "react" } else { "pass" }
        );
    }
}
