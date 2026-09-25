//! The workflow model and the argv tokenizer the `ci.yml` checks that judge its structure read through, held
//! shape by shape. The scans that judge its shell read it as text, and `repository-checks` states why.
//!
//! Each row is a shape a line reader got wrong or could not see, and the answer the grammar gives. The checks
//! built on these readers inherit exactly what is decided here, so a row that moves is a check that moves.

mod support;

use std::collections::BTreeMap;

use support::shell::{Argv, RESERVED_WORDS, SHELL_OWN_WORDS, script_commands, words};
use support::workflow::{Workflow, is_comment_line, parse};

fn read(text: &str) -> Workflow {
    parse(text).unwrap_or_else(|why| panic!("expected a readable workflow, got: {why}\n{text}"))
}

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

/// The same env name in two jobs is two values in two scopes, not a doubled declaration.
#[test]
fn one_name_in_two_jobs_is_two_scopes() {
    let workflow = read(
        "jobs:\n  a:\n    env:\n      CARGO_TERM_COLOR: always\n    steps:\n      - run: x\n  b:\n    env:\n      CARGO_TERM_COLOR: never\n    steps:\n      - run: y\n",
    );
    let [a, b] = workflow.jobs.as_slice() else {
        panic!("two jobs")
    };
    assert_eq!(
        workflow.env_for(a, &a.steps[0])["CARGO_TERM_COLOR"],
        "always"
    );
    assert_eq!(
        workflow.env_for(b, &b.steps[0])["CARGO_TERM_COLOR"],
        "never"
    );
}

/// A name written twice in one mapping is refused: which value applies is not decidable.
#[test]
fn one_name_twice_in_one_mapping_is_refused() {
    let refused = parse("jobs:\n  a:\n    env:\n      A: \"1\"\n      A: \"2\"\n");
    assert!(refused.is_err_and(|why| why.contains("written twice")));
}

/// A `with:` value is handed to an action rather than expanded, so a null there is not refused.
#[test]
fn a_null_with_value_is_read() {
    let workflow =
        read("jobs:\n  j:\n    steps:\n      - uses: a@v1\n        with:\n          x: ~\n");
    assert_eq!(workflow.jobs[0].steps[0].with[0].key, "x");
}

/// A quoted `null` is the text it spells, and only a plain one is YAML's no-value.
#[test]
fn a_quoted_null_env_value_is_its_text() {
    let workflow = read("env:\n  A: \"null\"\n  B: '~'\njobs:\n  j:\n    steps:\n      - run: x\n");
    let job = &workflow.jobs[0];
    let seen = workflow.env_for(job, &job.steps[0]);
    assert_eq!(seen.get("A").map(String::as_str), Some("null"));
    assert_eq!(seen.get("B").map(String::as_str), Some("~"));
}

/// A value is in force only where it is declared: job `a`'s pin does not reach job `b`.
#[test]
fn a_job_env_does_not_reach_another_job() {
    let workflow = read(
        "jobs:\n  a:\n    env:\n      MSRV: \"1.85\"\n    steps:\n      - run: x\n  b:\n    steps:\n      - run: cargo \"+$MSRV\" test\n",
    );
    let b = &workflow.jobs[1];
    let seen = workflow.env_for(b, &b.steps[0]);
    assert!(!seen.contains_key("MSRV"));
    assert_eq!(words(&b.steps[0].run.as_ref().unwrap().value, &seen), None);
}

/// Workflow, job and step env layer in that order, each overriding the one before.
#[test]
fn env_layers_workflow_then_job_then_step() {
    let workflow = read(
        "env:\n  A: w\n  B: w\n  C: w\njobs:\n  j:\n    env:\n      B: j\n      C: j\n    steps:\n      - env:\n          C: s\n        run: x\n",
    );
    let job = &workflow.jobs[0];
    assert_eq!(
        workflow.env_for(job, &job.steps[0]),
        env(&[("A", "w"), ("B", "j"), ("C", "s")])
    );
}

/// Flow-form `with:` and `env:` are the same mappings as their block form.
#[test]
fn flow_form_mappings_are_read() {
    let workflow = read(
        "jobs:\n  j:\n    env: {A: b}\n    steps:\n      - {uses: EmbarkStudios/cargo-deny-action@v2, with: {command: check}}\n",
    );
    let job = &workflow.jobs[0];
    assert_eq!(job.env[0].value, "b");
    assert_eq!(job.steps[0].with[0].value, "check");
}

/// A blank line or a comment inside a block ends nothing.
#[test]
fn blank_lines_and_comments_inside_a_block_are_nothing() {
    let workflow = read(
        "jobs:\n  j:\n    env:\n      A: a\n\n      # a comment: b\n      B: b\n    steps: []\n",
    );
    let names: Vec<&str> = workflow.jobs[0]
        .env
        .iter()
        .map(|e| e.key.as_str())
        .collect();
    assert_eq!(names, ["A", "B"]);
}

/// A trailing comment on a job's header, and a quoted key, are still a job and a key.
#[test]
fn trailing_comments_and_quoted_keys_are_read() {
    let workflow = read("jobs:\n  alpha:  # the first job\n    \"if\": x\n    runs-on: y\n");
    let job = &workflow.jobs[0];
    assert_eq!(job.id, "alpha");
    assert!(job.keys.iter().any(|(key, line)| key == "if" && *line == 3));
}

/// Shapes the model cannot hold are refused by name, never read past.
#[test]
fn shapes_the_model_cannot_hold_are_refused() {
    for (shape, text, says) in [
        ("an anchor and alias", "a: &x 1\nb: *x\n", "anchor"),
        (
            "a merge key",
            "base: {c: 1}\njob:\n  <<: {c: 2}\n",
            "merge key",
        ),
        ("a tag", "a: !custom 1\n", "tag"),
        ("two documents", "a: 1\n---\nb: 2\n", "2 documents"),
        ("a non-mapping root", "- a\n", "mapping"),
        (
            "a null env value",
            "env:\n  A: ~\njobs:\n  j:\n    steps:\n      - run: x\n",
            "null",
        ),
        (
            "an empty env value",
            "jobs:\n  j:\n    env:\n      A:\n    steps:\n      - run: x\n",
            "null",
        ),
        (
            "jobs as a sequence",
            "jobs: [a]\n",
            "`jobs` is not a mapping",
        ),
        (
            "steps as a mapping",
            "jobs:\n  j:\n    steps: {a: b}\n",
            "`steps` is not a sequence",
        ),
        (
            "a nested env value",
            "jobs:\n  j:\n    env:\n      A: {b: c}\n",
            "not a scalar",
        ),
        // A `defaults` or `run` that is not a mapping holds no shell this can read, which is not the same fact as
        // a workflow declaring none.
        (
            "workflow defaults as a scalar",
            "defaults: sh\n",
            "`defaults` is not a mapping",
        ),
        (
            "a job's defaults.run as a scalar",
            "jobs:\n  j:\n    defaults: {run: sh}\n",
            "`defaults.run` is not a mapping",
        ),
    ] {
        let refused = parse(text);
        assert!(
            refused.as_ref().is_err_and(|why| why.contains(says)),
            "{shape}: expected a refusal naming `{says}`, got {refused:?}"
        );
    }
}

/// A folded block scalar is read with YAML's own folding, and a literal one line by line.
#[test]
fn block_scalars_read_as_yaml_reads_them() {
    let workflow = read(
        "jobs:\n  j:\n    steps:\n      - run: >\n          cargo\n          test\n      - run: |\n          cargo\n          test\n",
    );
    let steps = &workflow.jobs[0].steps;
    assert_eq!(steps[0].run.as_ref().unwrap().value, "cargo test\n");
    assert_eq!(steps[1].run.as_ref().unwrap().value, "cargo\ntest\n");
    // The content line, not the indicator's, for both block styles.
    assert_eq!(steps[0].run.as_ref().unwrap().line, 5);
    assert_eq!(steps[1].run.as_ref().unwrap().line, 8);
}

/// A step spans from its first key to the line before the next step, so a comment inside it is its own.
#[test]
fn a_step_spans_the_lines_up_to_the_next() {
    let workflow = read(
        "jobs:\n  j:\n    steps:\n      - name: one\n        # inside one\n        uses: a\n      - name: two\n        uses: b\n",
    );
    let steps = &workflow.jobs[0].steps;
    assert_eq!((steps[0].first_line, steps[0].last_line), (4, 6));
    assert_eq!((steps[1].first_line, steps[1].last_line), (7, 8));
}

/// The last step ends where `steps` does, not where its job does: a job key written after `steps` is the
/// job's, and so is a comment beside it.
#[test]
fn the_last_step_ends_where_steps_does() {
    let workflow = read(
        "jobs:\n  j:\n    steps:\n      - name: one\n        uses: a\n    # beside the job's key\n    timeout-minutes: 5\n  k:\n    steps: []\n",
    );
    let step = &workflow.jobs[0].steps[0];
    assert_eq!(step.first_line, 4);
    // Line 6 is inside the step's line span only by position; it is indented as the job's key, not the step's.
    assert!(!step.holds_comment_at(6, "    # beside the job's key"));
    assert!(
        step.last_line < 7,
        "the job key on line 7 is not the step's: {step:?}"
    );
}

/// A comment belongs to a step when it sits within the step's lines at the step's own depth or deeper.
#[test]
fn a_comment_inside_a_step_belongs_to_it() {
    let workflow =
        read("jobs:\n  j:\n    steps:\n      - name: one\n        # inside one\n        uses: a\n");
    let step = &workflow.jobs[0].steps[0];
    assert!(step.holds_comment_at(5, "        # inside one"));
}

/// The tokenizer's answers: argv where the shell decides it now, `None` where it decides it at run time.
#[test]
fn the_tokenizer_reads_argv_or_declines() {
    let msrv = env(&[("MSRV", "1.85"), ("EMPTY", "")]);
    for (line, expected) in [
        (
            "cargo \"+$MSRV\" test",
            Some(vec!["cargo", "+1.85", "test"]),
        ),
        ("cargo +${MSRV} test", Some(vec!["cargo", "+1.85", "test"])),
        // A longer name is another name, not `$MSRV` followed by `X`.
        ("cargo +$MSRVX test", None),
        (
            "RUSTDOCFLAGS=\"-D warnings\" cargo doc",
            Some(vec!["RUSTDOCFLAGS=-D warnings", "cargo", "doc"]),
        ),
        // Quotes elsewhere on the line are the shell's, and stay as the shell reads them.
        (
            "cargo \"+$MSRV\" test --features 'a b'",
            Some(vec!["cargo", "+1.85", "test", "--features", "a b"]),
        ),
        (
            "cargo test # a trailing comment",
            Some(vec!["cargo", "test"]),
        ),
        ("echo a#b", Some(vec!["echo", "a#b"])),
        ("x=$(cargo metadata)", None),
        ("echo `date`", None),
        ("cargo test | head", None),
        ("a && b", None),
        ("ls *.rs", None),
        ("cd ~/x", None),
        ("echo $1", None),
        ("echo ${MSRV:-1}", None),
        ("echo $UNDEFINED", None),
        ("echo 'unterminated", None),
        // Brace expansion is the shell's to perform, so an unquoted brace is declined; a quoted one is text.
        ("printf '%s' {a,b}", None),
        ("printf '%s' '{a,b}'", Some(vec!["printf", "%s", "{a,b}"])),
        ("echo a{1..3}", None),
        // An unquoted expansion to nothing removes its word; a quoted one is an empty word.
        ("echo $EMPTY x", Some(vec!["echo", "x"])),
        ("echo \"$EMPTY\" x", Some(vec!["echo", "", "x"])),
        ("echo a$EMPTY", Some(vec!["echo", "a"])),
        // A tilde is expanded after `=` and `:` too, not only at a word's start.
        ("x=~/y cmd", None),
    ] {
        let expected: Option<Vec<String>> =
            expected.map(|w| w.into_iter().map(String::from).collect());
        assert_eq!(
            words(line, &msrv).map(|argv| argv.words),
            expected,
            "`{line}`"
        );
    }
}

/// Which leading words are assignments is read from how each is written, before anything is expanded.
#[test]
fn an_assignment_is_one_as_written() {
    let prefix = env(&[("PREFIX", "X=1"), ("EMPTY", "")]);
    for (line, assignments) in [
        ("X=1 cargo", 1),
        ("X=1 Y+=2 cargo", 2),
        ("X=\"a b\" cargo", 1),
        ("X= cargo", 1),
        // Expanding to an assignment is not being one: the shell runs a command by that name.
        ("\"$PREFIX\" cargo", 0),
        ("$PREFIX cargo", 0),
        // Quoting the name or its `=` makes the word an argument.
        ("\"X=1\" cargo", 0),
        ("X\\=1 cargo", 0),
        // The prefix ends at the first word that is not one, even a word that expands to nothing.
        ("$EMPTY X=1 cargo", 0),
        ("cargo X=1", 0),
    ] {
        assert_eq!(
            words(line, &prefix).map(|argv| argv.assignments),
            Some(assignments),
            "`{line}`"
        );
    }
}

/// Whether a line is a comment is the grammar's answer: data in every scalar form, a comment where YAML drops it.
#[test]
fn a_line_is_a_comment_only_where_the_grammar_drops_it() {
    let text = "a: 1\n# a comment\n\nliteral: |\n  # data\nfolded: >\n  first\n  # data\nquoted: \"one\n  # data\"\n";
    for (line, comment, what) in [
        (2, true, "a comment line"),
        (3, true, "a blank line"),
        (5, false, "a literal block's content"),
        (
            8,
            false,
            "a folded block's content, which folding joins to the line before",
        ),
        (10, false, "a quoted scalar continued onto the line"),
    ] {
        assert_eq!(
            is_comment_line(text, line),
            Ok(comment),
            "line {line}: {what}"
        );
    }
}

/// A script is read whole: one line the tokenizer declines makes the script contribute no commands.
#[test]
fn a_script_is_read_whole_or_not_at_all() {
    let none = env(&[]);
    assert_eq!(
        script_commands(
            "cargo build\n# a comment\n\ncargo test \\\n  --workspace\n",
            &none
        ),
        Some(vec![
            Argv {
                assignments: 0,
                words: vec!["cargo".to_string(), "build".to_string()],
            },
            Argv {
                assignments: 0,
                words: vec![
                    "cargo".to_string(),
                    "test".to_string(),
                    "--workspace".to_string()
                ],
            },
        ])
    );
    // The continuation rule is the lexer's, as bash's: an escaped backslash ends its line, and a backslash
    // inside single quotes joins nothing.
    let argv = |words: &[&str]| Argv {
        assignments: 0,
        words: words.iter().map(|word| word.to_string()).collect(),
    };
    assert_eq!(
        script_commands("cargo a\\\\\ncargo test --workspace\n", &none),
        Some(vec![
            argv(&["cargo", "a\\"]),
            argv(&["cargo", "test", "--workspace"])
        ])
    );
    // A comment after a backslash-newline is a comment, not words a Definition of Done line could match.
    assert_eq!(
        script_commands("cargo build \\\n#cargo test --workspace\n", &none),
        Some(vec![argv(&["cargo", "build"])])
    );
    assert_eq!(
        script_commands("cargo 'a\\\nb'\n", &none),
        Some(vec![argv(&["cargo", "a\\\nb"])])
    );
    assert_eq!(script_commands("x=$(date)\ncargo test\n", &none), None);
    assert_eq!(
        script_commands("cat <<'EOF'\ncargo test\nEOF\n", &none),
        None
    );
    // The shell's own words decide what runs next, so a script using any is not read as a list of commands.
    for script in [
        "exit 0\ncargo test\n",
        "exec true\ncargo test\n",
        "builtin exit 0\ncargo test\n",
        "set -n\ncargo test\n",
        "source ./other.sh\ncargo test\n",
        "A=1 exit 0\ncargo test\n",
        "A=2\ncargo test\n",
        "A+=2\ncargo test\n",
        "A[0]=2\ncargo test\n",
    ] {
        assert_eq!(script_commands(script, &none), None, "{script:?}");
    }
}

/// The shell's own words are bash's, held against bash both ways.
///
/// **A declared set beside its producer**, which `AGENTS.md` admits where something downstream filters on the
/// claim: a script is declined on a word from this list, so a builtin missing from it would let a script that
/// ends early count as a witness, and a word listed that bash does not own would decline a script for nothing.
#[test]
fn the_shell_words_are_the_ones_bash_owns() {
    let output = support::bash::bash()
        .args(["-c", "compgen -b; compgen -k"])
        .output()
        .expect("bash runs — the scripts these words are about are run by it");
    assert!(
        output.status.success(),
        "compgen answered {:?}",
        output.status
    );
    let owned: std::collections::BTreeSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    let declared: std::collections::BTreeSet<String> = SHELL_OWN_WORDS
        .iter()
        .map(|word| (*word).to_string())
        .collect();
    assert_eq!(
        declared,
        owned,
        "missing from the declaration: {:?}; declared and not bash's: {:?}",
        owned.difference(&declared).collect::<Vec<_>>(),
        declared.difference(&owned).collect::<Vec<_>>()
    );
}

/// The reserved words are bash's, held against `compgen -k` both ways: a word missing would leave a command after it
/// unread as one, and a word listed that bash does not reserve would read an argument as a command.
#[test]
fn the_reserved_words_are_the_ones_bash_owns() {
    let output = support::bash::bash()
        .args(["-c", "compgen -k"])
        .output()
        .expect("bash runs");
    let reserved: std::collections::BTreeSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    let declared: std::collections::BTreeSet<String> = RESERVED_WORDS
        .iter()
        .map(|word| (*word).to_string())
        .collect();
    assert_eq!(declared, reserved);
}
