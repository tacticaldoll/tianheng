## Why

This is the output of the sweep `BACKLOG.md` filed as *the 天衡 shell's baseline-writing and CLI surface has never
been swept*. That entry's own first instruction was to build the enumeration rather than start guessing, so this
proposal reports what the enumeration measured, and it found two defects — both in the requirement, not in the
code.

**1. `list`'s refusal names no flag, and the requirement that cites it as "the same rule" requires naming.**

Measured, one flag at a time and nothing else on the command line:

| invocation | diagnostic |
| --- | --- |
| `list --warn-uncovered` | `error: list takes only --format; other flags are check-only` |
| `list --disallow-stale` | *(identical)* |
| `list --baseline f` | *(identical)* |
| `list --write-baseline f` | *(identical)* |
| `list --manifest-path f` | *(identical)* |
| `list --format sarif` | `error: list supports --format text|json|markdown; sarif projects the reaction (a check output), not the declared law` |

Every other cell of this surface names what the invocation did. All twenty value-flag cells do — `--baseline
requires a value, but was given an empty one`, `--format was given more than once`. The `--format` *value*
refusal above names the value and explains why. The five check-only flags share one sentence naming none of them.

And the requirement `List rejects flags that only apply to check` asks only that the runner "prints usage guidance
and exits 2, never silently ignored". The requirement covering the same conflict **within** `check` opens by
calling itself "the rule the `list` requirement above states across commands" and then requires "a usage error
**that names the flag**". One of the two is describing the other inaccurately, and the implementation faithfully
satisfies each as written — which is why no test caught it.

The asymmetry runs the wrong way. `--manifest-path` is in `list`'s rejected set and is the flag a user types by
habit; a reader who runs `tianheng list --manifest-path ./Cargo.toml --format json` is told "list takes only
`--format`" while `--format` is the flag they passed correctly.

**2. The requirement enumerates four check-only flags and the code checks five.**

`--manifest-path`, `--baseline`, `--write-baseline`, `--warn-uncovered` are named. `--disallow-stale` is not — it
was added to the runner later, `dispatch_list` rejects it, and the requirement's hand-written list was never
updated. The code is right and the spec under-describes it, which is the same stale-enumeration class this window
has closed three times at a larger scale.

**What the sweep did not find.** The baseline-write surface is **swept and defended**. Its twelve filesystem
operations were enumerated from the code — `canonicalize`, the mode read, the `O_EXCL` temp create, the plant
loop, `fchmod` on the descriptor, `sync_all` before the rename, the rename, the parent-directory flush whose error
is deliberately discarded, the guard's cleanup, and on the create path `create_new`, the `symlink_metadata`
dangling-symlink diagnosis and `read_link` — and each already has a test. Five further adversarial shapes were
probed live (absent parent directory, target is a directory, read-only parent, unreadable target, symlink to a
directory): every one exits 2 naming the path and the OS cause, with no silent success and no misdiagnosis. The
create-versus-overwrite decision takes the create path on `NotFound` **only**; every other read failure is a loud
refusal, so an unreadable existing baseline cannot be misreported as a creation race.

## What Changes

**One requirement tightened, and the implementation follows it.**

- `list`'s refusal SHALL name the flags the invocation actually supplied, making it consistent with the rule the
  `check`-internal requirement already claims to share with it.
- The rejected set SHALL be stated as **derived** — every flag `check` recognizes that `list` does not honor —
  rather than enumerated in prose. That is what stops the next flag from being added to the runner and not to the
  requirement, which is exactly what happened to `--disallow-stale`.
- `dispatch_list` collects the offending flags and names them; a test drives each one.

## Capabilities

### Modified Capabilities

- `cli-check-runner`: the requirement *List rejects flags that only apply to check* gains the naming obligation and
  loses its prose enumeration.

## Impact

- **Modified**: `crates/tianheng/src/runner.rs` (`dispatch_list`'s refusal), and `crates/tianheng/tests/baseline_cli.rs`.
- **Behaviour**: a usage error's *message* changes; its exit code does not. No adopter's build, baseline, or
  invocation changes — an invocation refused before is refused now, more precisely.
- **Not affected**: no public API, no `Constitution`, no baseline format. Version class **PATCH**.
