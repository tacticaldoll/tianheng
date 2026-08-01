# proposal: Louke Observes cfg_if Arm Contents

## Why

圭表 has read `cfg_if!` arms as real code since 0.2.3; 渾儀 joined it in this release
(`1fe062f`). 漏刻 has not, and its probe audit skips a `cfg_if!` body wholesale like any foreign
macro — `foreign_macro_body_end` is called in **two** independent passes, module-declaration
collection and probe scanning, and neither knows the macro's arms hold human-authored code.

Measured on ordinary, compilable source (one declared seam, one probe; controls clean):

| fixture | 漏刻 today |
| --- | --- |
| the seam's only `assert_boundary!` sits inside a `cfg_if!` arm | **reports the seam unprobed** |
| the identical probe at module top level (control) | clean |
| `pub mod net;` declared only inside an arm, `net.rs` holds the probe | **reports the seam unprobed** |
| a **typo'd** seam name probed inside an arm | **clean** — probed-but-undeclared never fires |
| an **un-auditable** probe (non-literal seam argument) inside an arm | **clean** — never reported |

Two of the audit's three reaction directions are broken inside an arm, in both error directions:

- **False negatives.** A typo'd seam and an un-auditable probe both escape. The un-auditable case
  contradicts a documented claim in `audit_probe_coverage`'s own contract — "never a silent skip (a
  silent skip would be a false negative)".
- **A false alarm that blocks.** A seam whose real, production probe lives in an arm is reported
  unprobed, so an adopter who uses `cfg_if!` for platform branching fails CI over coverage they
  actually have. Fail-loud rather than silent, but wrong on source that compiles.

The current spec **mandates** the broken behavior: `runtime-origin-assertion`'s CI-face requirement
says a probe inside "the body of any macro invocation `ident! (…)/{…}/[…]` other than the
`assert_boundary!` probe itself … SHALL NOT count as coverage". Adding transparency without
modifying that sentence would leave the spec contradicting itself, so this is a MODIFIED delta, not
an addition beside it.

`cfg_if_transparency_conformance.rs` currently pins **two of three** dimensions and says so in its
own module doc. This change closes that gap and 漏刻 joins those same tests.

## What Changes

- Recognize the one transparent control-flow macro by **name** in 漏刻's own byte scanner
  (reimplemented louke-locally — 三儀 ⊥ 三儀 forbids importing 圭表's), matching the name test the
  other two dimensions apply.
- **Probe pass** (`scan_source_with_markers`): do not skip a transparent invocation's body — scan
  into it as ordinary code, so a probe, a typo'd seam, and an un-auditable probe inside an arm are
  all observed exactly as at top level.
- **Module pass** (`collect_scope_modules`): treat each arm as a **transparent scope** — recurse
  into the arm's brace group with the enclosing bases unchanged (an arm adds no directory component,
  unlike an inline `mod`), so an arm-declared `mod` enters the reachable corpus and its file's probes
  are counted.
- Treat an arm-declared module as **cfg-conditional** for absent-file purposes, the rule 圭表 settled
  in `a567211` and 渾儀 adopted in `1fe062f`, so an absent conventional file (or `#[path]` target) is
  tolerated rather than a constitution error — while a resolution ambiguity stays an error under
  every gate.
- MODIFY the two `runtime-origin-assertion` requirements this touches, and state the same bounds the
  other dimensions state: only `cfg_if` is transparent, and observation stays cfg-blind.

## Capabilities

### Modified Capabilities

- `runtime-origin-assertion`: the CI-face macro-body exclusion gains the transparent-macro carve-out,
  and the root-aware module walker gains arm-declared modules plus arm membership as a
  cfg-conditional source.

## Impact

- `crates/louke/src/audit/scan.rs`: the transparent-name test, the probe pass's `!` branch, the
  module pass's `!` branch and its arm recursion, and the cfg-conditional threading.
- `crates/louke/src/audit/tests.rs`: the five measured shapes as regression coverage, plus the
  controls and the nested/else-if/paren shapes.
- `crates/tianheng/tests/cfg_if_transparency_conformance.rs`: 漏刻 joins the existing fixtures; the
  module doc's "deliberately absent" paragraph retires.
- `CHANGELOG.md`: `[Unreleased]` → `### Fixed`.
- Non-breaking: no public API or wire-format change. Adopters see one fewer false alarm and two
  closed false negatives; a newly caught typo'd or un-auditable probe inside an arm is a real finding
  and absorbable by baseline.
