## Context

This repository resolves four kinds of reference — a path, an `--exact` identifier, a `(bound: …)` reference,
and a bare bound id. Each lands on an **identity**. A line number lands on a **position**, and positions move:
the two live coordinates were correct on 2026-08-09 and are now off by whole entries, one of them displaced by
the window that repaired the source-side instances.

`reference-integrity` already states the rule and reacts over every line-comment format. Markdown is outside by
construction — it is the format classified as whole-document prose — with a reason that holds for *phrases*.

## Goals / Non-Goals

**Goals:**

- A coordinate is refused wherever it is written, because the argument against it never depended on the format.
- The reaction reads a shape, never the prose around it.

**Non-Goals:**

- Reopening the declined judgement. A positional *phrase* in Markdown — "the entry above", "86 lines below" —
  stays outside and unread. Nothing here decides what a sentence means.
- Sweeping Markdown for the *other* property the line-comment formats carry. The format-classification rule
  says a format is swept for both properties or for neither; this adds a **third** property whose corpus is
  every tracked format, so the pairing is untouched.

## Decisions

**Refuse the shape, do not resolve it.** The other four reference kinds resolve to an identity and fail when
the identity is absent. A coordinate cannot be resolved that way: `CHANGELOG.md:173` is *valid* — the file has a
line 173 — while naming nothing anyone meant. Validity is the trap, so the answer is refusal rather than
resolution. This is why it is a new property rather than an extension of the path sweep.

**Recognize a backticked path with a line suffix, not any `word:number`.** The discriminator is that the left
side is a **tracked path**, which is produced by the enumeration the sibling sweep already uses. `1:1`, `note:5`
and a time of day are not tracked paths, so they are not coordinates. This is the same construction that made
bare bound ids precise: require the left side to name something the repository enumerates.

**Records are refused too, and that is the deliberate part.** The Markdown exclusion exists because a phrase in
a record narrates a past state. A coordinate in a record narrates nothing — the reader cannot check it, and it
rots exactly as a live one does. So the rule reaches records, and what protects a record is that naming the
entry is always available and never goes stale.

**The empty set is kept.** After the two repairs nothing matches. This repository already keeps a recognizer
asserting its own emptiness rather than pruning it, on the ground that retiring a recognizer and forgetting one
look identical afterwards. The vacuity guard is therefore on the *corpus*, not on the match count.

## Risks / Trade-offs

**A legitimate coordinate in a record** → None was found, and the repair is always available: name the entry.
If one is ever genuinely needed, it arrives with an amendment rather than a silent exemption.

**The corpus grows with the tree** → Every tracked Markdown file is read. The recognizer is one shape test
against a produced path set, so the cost is the read.

**A coordinate written without backticks** → Not matched. That is a declared limit of the shape reading and is
the same limit the bound-id resolver carries: recognition is by structure, and unstructured prose is the
judgement this repository declines. Stated in the requirement rather than left to be discovered.

## Migration Plan

None. Two instances, both repaired in this change, both by naming the entry they meant.

## Open Questions

- **Should the other four reference kinds be swept over Markdown too?** Paths already are. `--exact`
  identifiers and bound ids are source-shaped and no Markdown carries one today. Named so the asymmetry is
  deliberate rather than unnoticed.
