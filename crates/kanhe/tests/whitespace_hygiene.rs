//! Repository check: whitespace hygiene across every tracked text file.
//!
//! No tracked text file carries trailing whitespace on a line, ends with a blank line, or is missing its
//! final newline.
//!
//! **This was the last judgement in this crate still answering with a bare `assert!`.** Every sibling returns
//! [`Refusal`], which separates *the source disagrees* from *the source could not be read* — and this one
//! collapsed the two in the direction that reports clean: a tracked file it could not open was `continue`d,
//! so a file nobody could read counted as hygienic. `census::sweep` refuses the identical condition, in its
//! own words, because *an unread document is not a document without one*, and `reference_integrity` states it
//! as a rule: a file this check claims to have inspected must have been read.

use std::path::{Path, PathBuf};
use std::process::Command;

use kanhe::refusal::{Kind, Refusal, cannot_judge, violation};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Every hygiene offence in `listing`, and how many files were actually opened and read.
///
/// **The count is returned rather than folded into the verdict**, because `offences.is_empty()` is satisfied
/// by a corpus that collapsed to nothing exactly as it is by a clean one, and those are opposite facts. Three
/// paths below leave a file uninspected. One of them depends on `git ls-files --eol` writing exactly one tab
/// before the path — measured today, 389 lines, every one with exactly two tab-separated fields — so a change
/// in that format sends every line down it, and without the caller's guard this check would assert nothing
/// over nothing and report clean.
fn offences(root: &Path, listing: &str) -> (Vec<Refusal>, usize) {
    let mut offences = Vec::new();
    let mut inspected = 0usize;

    for line in listing.lines() {
        let Some((eol_info, path_str)) = line.split_once('\t') else {
            offences.push(cannot_judge(format!(
                "`git ls-files --eol` produced `{line}`, which carries no tab before a path. A line this \
                 reader cannot parse is a tracked file it did not inspect, not a file without offences"
            )));
            continue;
        };

        // Binary as **git** classifies it, which is the same classification `cargo publish` and every other
        // tool here defers to. Not a file this check failed to read — a file whitespace is not a property of.
        // Measured: no tracked path carries `i/-text` today, so this branch is unexercised and the caller's
        // corpus guard is what keeps it from quietly becoming the whole corpus.
        if eol_info.starts_with("i/-text") {
            continue;
        }

        let content = match std::fs::read(root.join(path_str)) {
            Ok(content) => content,
            Err(err) => {
                offences.push(cannot_judge(format!(
                    "{path_str} is tracked and could not be read ({err}), so whether it is hygienic is \
                     unknown — an unread file is not a file without offences"
                )));
                continue;
            }
        };
        // Counted here: the file was opened and its bytes are in hand. An empty one is inspected and clean,
        // not skipped — it has no last line to end badly and no missing newline to be missing.
        inspected += 1;
        if content.is_empty() {
            continue;
        }

        if content.last() != Some(&b'\n') {
            offences.push(violation(format!("{path_str}: missing final newline")));
        }

        let text = String::from_utf8_lossy(&content);
        for (index, line_text) in text.lines().enumerate() {
            let normalized = line_text.strip_suffix('\r').unwrap_or(line_text);
            if normalized.ends_with(' ') || normalized.ends_with('\t') {
                offences.push(violation(format!(
                    "{path_str}:{}: trailing whitespace",
                    index + 1
                )));
            }
        }

        if text.ends_with("\n\n") || text.ends_with("\r\n\r\n") {
            offences.push(violation(format!("{path_str}: blank line at end of file")));
        }
    }

    (offences, inspected)
}

#[test]
fn whitespace_hygiene_across_tracked_text_files() {
    let Some(root) = workspace_root() else {
        return;
    };

    // The enumeration itself is an input like any other: a `git` that could not run leaves this check with no
    // corpus, which is not a corpus without offences.
    let output = Command::new("git")
        .args(["ls-files", "--eol"])
        .current_dir(&root)
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "CannotJudge: could not run `git ls-files --eol` ({err}), so no \
                                      tracked file was inspected"
            )
        });
    assert!(
        output.status.success(),
        "CannotJudge: `git ls-files --eol` failed, so no tracked file was inspected: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let listing = String::from_utf8_lossy(&output.stdout);
    let (offences, inspected) = offences(&root, &listing);

    // **Before the verdict, not after.** The shape every sibling here uses — `census::sweep`'s vacuity
    // direction, `tracked_specs`'s "would report clean without reading anything", `gate_exit_classes`'s
    // `examined > 0`. This check had none, so every path that drops a line dropped it into a silence
    // indistinguishable from cleanliness.
    assert!(
        inspected > 0,
        "no tracked file was inspected, so this check would report clean over nothing — the vacuity \
         direction. {} line(s) of `git ls-files --eol` were read",
        listing.lines().count()
    );

    assert!(
        offences.is_empty(),
        "{inspected} tracked file(s) inspected; a file carries whitespace this repository does not keep, or \
         a tracked file could not be read:\n{}",
        offences
            .iter()
            .map(|refusal| format!("  {:?}: {}", refusal.kind, refusal.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// An unreadable tracked file is a cannot-judge, not a file without offences.
///
/// The direction the silent `continue` hid. Constructed rather than declared: the listing names a path that
/// does not exist under the root, which is exactly what `read` meets when a tracked file cannot be opened.
#[test]
fn an_unreadable_tracked_file_is_refused_rather_than_skipped() {
    let scratch = std::env::temp_dir().join(format!(
        "tianheng-whitespace-unreadable-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");

    let listing = "i/lf\tzzz_absent_whitespace_probe.md\n";
    let (offences, inspected) = offences(&scratch, listing);
    let _ = std::fs::remove_dir_all(&scratch);

    assert_eq!(
        offences.len(),
        1,
        "an unreadable tracked file must produce exactly one refusal, got {offences:?}"
    );
    assert_eq!(
        offences[0].kind,
        Kind::CannotJudge,
        "an unread file is not a file without offences, so it is not a violation"
    );
    assert!(
        offences[0]
            .message
            .contains("zzz_absent_whitespace_probe.md"),
        "the refusal must name the file it could not read, got {:?}",
        offences[0].message
    );
    assert_eq!(
        inspected, 0,
        "a file that could not be read was not inspected, so it must not be counted as one"
    );
}

/// A listing line this reader cannot parse is refused, not passed over.
///
/// The path that would swallow the whole corpus: it depends on `git ls-files --eol` writing exactly one tab
/// before the path, and a format change sends every line down it. Skipping silently, the check would then
/// assert `offences.is_empty()` over zero files and report clean.
#[test]
fn a_listing_line_without_a_path_separator_is_refused() {
    let (offences, inspected) =
        offences(Path::new("/nonexistent"), "i/lf w/lf attr/ no-tab-here\n");
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(offences[0].kind, Kind::CannotJudge);
    assert_eq!(inspected, 0);
}

/// The judgement can see each offence it exists for, or its silence over this repository says nothing.
///
/// Every assertion in the live direction is a *silence*, and silence has more than one cause. These are the
/// three shapes, each shown to the reader and each required to be named — and each as a **violation**, so the
/// two kinds are held apart in the direction that matters as well as in the one above.
#[test]
fn each_offence_shape_is_named_when_it_is_shown() {
    let scratch = std::env::temp_dir().join(format!(
        "tianheng-whitespace-offences-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");

    for (name, body, expected) in [
        (
            "trailing.md",
            "a line with a space \n",
            "trailing whitespace",
        ),
        ("nonewline.md", "no final newline", "missing final newline"),
        ("blankend.md", "text\n\n", "blank line at end of file"),
    ] {
        std::fs::write(scratch.join(name), body).expect("write the probe");
        let (offences, inspected) = offences(&scratch, &format!("i/lf\t{name}\n"));
        assert_eq!(inspected, 1, "{name} was not inspected");
        assert!(
            offences.iter().any(|refusal| {
                refusal.kind == Kind::Violation && refusal.message.contains(expected)
            }),
            "{name} must be named as a violation carrying {expected:?}, got {offences:?}"
        );
    }

    // The control: a file with none of the three is silent, so the assertions above are about the offences
    // rather than about a judgement that reports everything.
    std::fs::write(scratch.join("clean.md"), "text\n").expect("write the control");
    let (clean, inspected) = offences(&scratch, "i/lf\tclean.md\n");
    let _ = std::fs::remove_dir_all(&scratch);
    assert_eq!(inspected, 1);
    assert!(
        clean.is_empty(),
        "a hygienic file must be silent, got {clean:?}"
    );
}
