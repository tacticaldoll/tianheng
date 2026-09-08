//! The probe environment every hermetic-`git` builder is proven in — parsed, validated and built once.
//!
//! **Three runners held the same job and drifted at it.** `kanhe::hermetic_git`'s builder and the two
//! `shengmo` test targets that cannot reach it each parsed the case inventory, each checked it their own
//! way, and each assembled the child's environment by hand. What drifted was not the builders — it was the
//! evidence: one runner's baseline missed a variable another's did, and the count each accepted stood in for
//! a set none of them held.
//!
//! This module is that job. It lives here because `shengmo` ships in no package and is the one member both
//! consumers reach: `kanhe` depends on it, and the two copies are its own test targets. A caller supplies
//! only what is irreducibly its own — the builder, which runs in the child.
//!
//! **Two sets, not one.** The inventory's `case` rows name channels to attack; its `baseline` rows name
//! variables to clear before any case runs. Deriving the second from the first left `GIT_CONFIG_NOSYSTEM`
//! inherited — a variable the builder sets and no case attacks, which suppresses the system-config channel
//! and so changes what the `GIT_CONFIG_SYSTEM` case's control reads.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// The tracked inventory, relative to the workspace root.
///
/// **Beside its owner.** Held under `crates/kanhe/tests/fixtures/`, this module reached into a crate that
/// depends on it for the evidence it is the owner of — so the owner was not one: `kanhe` could move or
/// delete the file this module is built around, and a consumer that cannot reach `kanhe` at all was reading
/// a path inside it.
pub const INVENTORY: &str = "crates/shengmo/tests/fixtures/hermetic_channels.tsv";

/// One channel to attack: how it is injected, the observation it moves, and what an isolated command reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    /// The environment variable this case injects.
    pub channel: String,
    /// How it is injected — `git-dir`, `work-tree`, `index`, `config-file`, `literal:…` or `indexed:k=v`.
    pub injection: String,
    /// The `git` reading this channel moves.
    pub observation: String,
    /// What an isolated command reads, with `(empty)` decoded to the empty string.
    pub isolated: String,
}

/// The inventory: what may be attacked, and what must be emptied first.
#[derive(Debug, Clone)]
pub struct Inventory {
    cases: Vec<Case>,
    /// **A set, because the contract says one.** Held as a `Vec`, a name repeated in the inventory was
    /// accepted and kept — a state *two sets, not one* calls impossible, spellable anyway. The type is what
    /// refuses it now, and ingestion says so rather than deduplicating in silence.
    baseline: BTreeSet<String>,
}

impl Inventory {
    /// The channels to attack, one case each.
    pub fn cases(&self) -> &[Case] {
        &self.cases
    }
}

/// Read the inventory under `root`, refusing a set that is not one.
///
/// **A row count is not a set.** Held at a minimum, a row could be replaced by a second row for a channel
/// already listed: the count intact, the channel it displaced asked about by nobody. Identity is what the
/// matrix is, so a repeated channel is refused — and every attacked channel must be in the baseline, since
/// a channel worth injecting is one worth clearing first.
pub fn read(root: &Path) -> Inventory {
    let text = std::fs::read_to_string(root.join(INVENTORY))
        .unwrap_or_else(|err| panic!("the channel inventory is readable: {err}"));
    let mut cases = Vec::new();
    let mut baseline = BTreeSet::new();
    for line in text.lines() {
        if line.trim_start().starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.first().copied() {
            Some("case") => {
                assert_eq!(
                    fields.len(),
                    5,
                    "a case row carries the kind and four fields: {line:?}"
                );
                cases.push(Case {
                    channel: fields[1].to_string(),
                    injection: fields[2].to_string(),
                    observation: fields[3].to_string(),
                    // `(empty)` rather than a blank field: a trailing tab is whitespace this repository
                    // refuses, and a sentinel a reader decodes is clearer than an absence to infer.
                    isolated: if fields[4] == "(empty)" {
                        String::new()
                    } else {
                        fields[4].to_string()
                    },
                });
            }
            Some("baseline") => {
                assert_eq!(
                    fields.len(),
                    2,
                    "a baseline row carries the kind and one variable: {line:?}"
                );
                assert!(
                    baseline.insert(fields[1].to_string()),
                    "the inventory clears {} twice; a baseline is a set, and a repeated row is a row \
                     nobody can act on differently",
                    fields[1]
                );
            }
            other => panic!("the inventory carries a row of an unknown kind: {other:?}"),
        }
    }

    let mut seen = BTreeSet::new();
    for case in &cases {
        assert!(
            seen.insert(case.channel.clone()),
            "the inventory names {} twice; a repeated row makes the count without making the case, so the \
             channel it displaced is asked about by nobody",
            case.channel
        );
    }
    for case in &cases {
        assert!(
            baseline.contains(case.channel.as_str()),
            "the inventory attacks {} and does not clear it first, so what a case demonstrates could be \
             an inherited channel rather than the injected one",
            case.channel
        );
    }
    assert!(
        cases.len() >= 8,
        "the inventory collapsed to {} case(s); a matrix that shrinks is one this check stops asking about",
        cases.len()
    );
    assert!(
        !baseline.is_empty(),
        "the inventory clears nothing, so no case starts from a baseline"
    );

    Inventory { cases, baseline }
}

/// The `git` arguments an observation is made with.
pub fn arguments(observation: &str) -> Vec<&'static str> {
    match observation {
        "log" => vec!["log", "-1", "--format=%s"],
        "ls-files" => vec!["ls-files"],
        "status" => vec!["status", "--porcelain"],
        "config-probe-marker" => vec!["config", "--default", "isolated", "--get", "probe.marker"],
        other => panic!("the inventory names an observation no probe makes: {other}"),
    }
}

/// Clear the baseline and inject exactly one case into `probe`'s environment.
///
/// **One channel per case is a baseline, not an addition.** Injected onto the environment a test binary
/// inherited, a case ran under whatever `GIT_*` the host already carried, so what its control demonstrated
/// was *a* channel rather than *the* channel.
pub fn prepare(
    probe: &mut Command,
    inventory: &Inventory,
    case: &Case,
    decoy: &Path,
    config: &Path,
) {
    for variable in &inventory.baseline {
        probe.env_remove(variable);
    }
    match case.injection.as_str() {
        "git-dir" => {
            probe.env(&case.channel, decoy.join(".git"));
        }
        "work-tree" => {
            probe.env(&case.channel, decoy);
        }
        "index" => {
            probe.env(&case.channel, decoy.join(".git/index"));
        }
        "config-file" => {
            probe.env(&case.channel, config);
        }
        other => {
            if let Some(value) = other.strip_prefix("literal:") {
                probe.env(&case.channel, value);
            } else if let Some(pair) = other.strip_prefix("indexed:") {
                // Three variables spelling one channel: the count, and the key and value at index zero.
                let (key, value) = pair
                    .split_once('=')
                    .unwrap_or_else(|| panic!("an indexed injection is `key=value`, got {pair:?}"));
                probe
                    .env(&case.channel, "1")
                    .env("GIT_CONFIG_KEY_0", key)
                    .env("GIT_CONFIG_VALUE_0", value);
            } else {
                panic!("the inventory names an injection no probe makes: {other}");
            }
        }
    }
}

/// One observation, under a builder and under a bare `Command`, with each exit status.
#[derive(Debug, Clone)]
pub struct Reading {
    /// The builder's exit code.
    pub builder_status: i32,
    /// What the builder read.
    pub builder: String,
    /// What the builder said on stderr.
    pub builder_stderr: String,
    /// A bare `Command`'s exit code.
    pub bare_status: i32,
    /// What a bare `Command` read — the control.
    pub bare: String,
    /// What a bare `Command` said on stderr.
    pub bare_stderr: String,
}

/// The line a probe child prints, so both halves spell the report the same way.
pub fn report(builder: (i32, String, String), bare: (i32, String, String)) -> String {
    format!(
        "PROBE_BEGIN\nbuilder-status={}\nbuilder={}\nbuilder-stderr={}\nbare-status={}\nbare={}\nbare-stderr={}\nPROBE_END",
        builder.0, builder.1, builder.2, bare.0, bare.1, bare.2
    )
}

/// The child's report, or a refusal naming the channel it was taken for.
pub fn reading(stdout: &str, channel: &str) -> Reading {
    let reported = stdout
        .split_once("PROBE_BEGIN\n")
        .and_then(|(_, rest)| rest.split_once("PROBE_END"))
        .map(|(body, _)| body.to_string())
        .unwrap_or_else(|| panic!("the probe child produced no reading for {channel}:\n{stdout}"));
    let line = |key: &str| {
        reported
            .lines()
            .find_map(|line| line.strip_prefix(key))
            .unwrap_or_else(|| {
                panic!("the probe reported no `{key}` line for {channel}:\n{reported}")
            })
            .to_string()
    };
    let code = |key: &str| {
        line(key).parse::<i32>().unwrap_or_else(|err| {
            panic!("the probe reported an unreadable `{key}` for {channel}: {err}")
        })
    };
    Reading {
        builder_status: code("builder-status="),
        builder: line("builder="),
        builder_stderr: line("builder-stderr="),
        bare_status: code("bare-status="),
        bare: line("bare="),
        bare_stderr: line("bare-stderr="),
    }
}

/// Hold one reading against its case: the control first, then the isolation.
pub fn judge(case: &Case, reading: &Reading) {
    for (who, code, stderr) in [
        ("builder", reading.builder_status, &reading.builder_stderr),
        ("bare", reading.bare_status, &reading.bare_stderr),
    ] {
        assert_eq!(
            code, 0,
            "the {who} `git` under {} exited {code}, so its reading is a failure rather than an answer: \
             {stderr}",
            case.channel
        );
    }
    // The control first: this channel does move this reading, so the assertion after it is a difference
    // rather than an environment that never arrived.
    assert_ne!(
        reading.bare, case.isolated,
        "a bare `Command` read the same under {} as without it, so this case demonstrates no channel and \
         the assertion below would hold for the wrong reason",
        case.channel
    );
    assert_eq!(
        reading.builder, case.isolated,
        "this builder followed {}, so a verdict behind it is about a tree or a configuration nobody asked \
         for",
        case.channel
    );
}
