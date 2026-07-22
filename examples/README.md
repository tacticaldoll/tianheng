# Runnable examples

The focused examples are the adoption surface:

| Example | Reaction owner |
|---|---|
| `guibiao-standalone` | static crate/module adoption, severity, baseline, and identity metadata |
| `hunyi-standalone` | signature exposure and visibility |
| `composed` | staged static + semantic + runtime funnel |
| `sans-io-pure` | composed inline-clock + async-exposure profile |
| `unsafe-confinement` | unsafe-confinement in a crate that legitimately contains unsafe |

`capability-catalog` is different: it is a deliberately dense contract-coverage fixture for the
published families that have no honest home above—dependency-source metadata,
external-crate confinement, trait-impl locality, forbidden markers, dyn-trait exposure, and
impl-trait exposure. It is not an architecture recommendation or a starting tutorial.

Together these owners execute the published family set through real evaluators. Dimension unit
tests remain responsible for individual modifiers and scanner edge cases; a future public builder
method still requires an explicit OpenSpec/API decision rather than being magically enumerated here.
The examples gate separately derives every immediate workspace in this directory and requires its
owner to finish both quality and declared-reaction assertions, so adding a directory without wiring
it into the gate fails rather than silently shrinking this adoption surface.
