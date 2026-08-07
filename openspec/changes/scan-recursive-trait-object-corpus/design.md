## Context

`composition_introduces_no_trait_object` currently enumerates only `.rs` files directly under `src/`, then tries
to prove nested directories irrelevant by requiring their module declarations to be non-public. That premise is
insufficient because `pub use private_module::Item` can expose a nested item without making the module public.

## Goals / Non-Goals

**Goals:**

- make directory nesting irrelevant to the lexical corpus;
- retain the existing one-line recognizer and its declared continuation-line bound;
- fail loud if any traversed directory or Rust source cannot be read;
- keep the non-vacuity assertion over inspected files.

**Non-Goals:**

- resolve Rust visibility or re-export reachability;
- replace the lexical recognizer with a semantic scanner;
- close the existing multi-line signature observation bound.

## Decisions

### Traverse every directory below `src`

A small helper will walk directories iteratively, count each `.rs` file, and run the existing recognizer through
the Rust executed-source region. This deliberately over-approximates private nested items in the safe direction,
matching the existing policy.

### Delete the top-level soundness premise

Once the corpus is recursive, module visibility no longer decides whether a source file is read. Retaining the
premise check would add an unrelated restriction and preserve a false explanation of the reaction.

## Risks / Trade-offs

Private nested `pub` items containing trait objects can now fail the guard even if they are not re-exported. This
is the same declared safe-direction over-approximation already accepted for top-level private modules.

## Verification

A temporary nested source tree will contain a one-line public trait-object signature. The helper must report that
offender; temporarily restoring top-level-only traversal must make the same guard fail.
