use crate::wrapper_parser::{Arm, parser_arms, value_guard};

/// The scan reads the parser's `case` and no other `case` in the script.
///
/// **The dependency this removes was on someone else's formatting.** `parser_arms` read every
/// `)`-terminated line in the file, and was correct over `scripts/merge-pr.sh` only because its inner
/// `case $conclusion in` writes each body on its pattern line (`SUCCESS) ;;`), so no line there ends in `)`.
/// Reformatting that inner case onto separate lines gives it a `*)` that opens an arm, and `BTreeMap::insert`
/// then drops one of the two — a dropped catch-all being the arm every refusal rests on.
///
/// Given as a constructed script rather than as a perturbation of the wrapper, because the wrapper's own two
/// catch-alls happen to carry identical properties today: the collision is real and its consequence there is
/// not, so a perturbation would report green and prove nothing. Here the second `case`'s arms are given
/// properties that differ from the parser's, which is what makes the boundary observable.
#[test]
fn the_scan_reads_the_parsers_case_and_no_other() {
    let script = "\
#!/usr/bin/env bash
require_value() {
    if (($1 < 2)); then
        refuse \"$2\"
    fi
}
while (($#)); do
    case $1 in
    --subject)
        require_value \"$#\" \"$1\" \"${2-}\"
        subject=$2
        shift 2
        ;;
    --admin)
        passthrough+=(\"$1\")
        shift
        ;;
    *)
        refuse \"$1\" \"not admitted\"
        ;;
    esac
done
for name in $names; do
    case $conclusion in
    --subject)
        echo \"a different meaning for the same token\"
        shift 2
        ;;
    *)
        disagreeing+=\"$2\"
        shift 2
        ;;
    esac
done
";
    let arms = parser_arms(script, "require_value");

    assert_eq!(
        arms.keys().cloned().collect::<Vec<_>>(),
        vec![
            "*".to_string(),
            "--admin".to_string(),
            "--subject".to_string()
        ],
        "the second `case`'s arms must not enter the map — its `--subject` and `*` would overwrite the \
         parser's own, and the properties they carry are not the parser's"
    );
    assert_eq!(
        arms["--subject"],
        Arm {
            guards: true,
            guards_with_value: true,
            consumes: true
        },
        "the parser's `--subject` guards with a value and consumes one; the second `case`'s namesake \
         consumes without guarding, and reading it would report this arm unguarded"
    );
    assert_eq!(
        arms["*"],
        Arm {
            guards: false,
            guards_with_value: false,
            consumes: false
        },
        "the parser's catch-all takes no value; the second `case`'s does, and reading it would report the \
         arm every refusal rests on as one that consumes"
    );
}

/// Several differently-named guards in one wrapper is a finding, not a name silently picked.
///
/// **`.next()` over the call lines was the first form**, which is the habit [`crate::selection`] exists to
/// end. One guard called from several arms stays the ordinary shape; two guards is what refuses.
#[test]
fn several_value_guards_in_one_wrapper_refuse_and_one_called_twice_does_not() {
    let one = "\
    case $1 in
    --subject)
        require_value \"$#\" \"$1\" \"${2-}\"
        ;;
    --body-file)
        require_value \"$#\" \"$1\" \"${2-}\"
        ;;
    esac
";
    assert_eq!(
        value_guard(one, "one.sh").map_err(|r| r.message),
        Ok("require_value".to_string()),
        "one guard called from two arms is the ordinary shape"
    );

    let two = "\
    case $1 in
    --subject)
        require_value \"$#\" \"$1\" \"${2-}\"
        ;;
    --body-file)
        require_a_value \"$#\" \"$1\" \"${2-}\"
        ;;
    esac
";
    let refusal = value_guard(two, "two.sh").expect_err(
        "two differently-named guards means one would be picked and the other's arms read as unguarded",
    );
    crate::refusal::expect("repository-checks#the-only-found-several", &refusal);

    let none = "    case $1 in\n    --subject)\n        subject=$2\n        ;;\n    esac\n";
    let refusal = value_guard(none, "none.sh")
        .expect_err("a wrapper taking a value with no guard hands nothing to be judged");
    crate::refusal::expect("repository-checks#the-only-found-none", &refusal);
}
