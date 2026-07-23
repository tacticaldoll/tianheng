# Design: `ScanDepth` Enum & DSL Integration

## Architectural Concept

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ScanDepth Enum & Layering Design                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  [xuanji] (底 / Reaction Model)                                              │
│  pub enum ScanDepth {                                                       │
│      #[default]                                                             │
│      Shallow,   // Default: current module / signature level (<10ms)       │
│      Subtree,   // Deep: recursive submodule & private item traversal       │
│      Audit,     // Cross-check: static AST + runtime probe coverage          │
│  }                                                                          │
│                                                                             │
│  [guibiao / hunyi / louke / tianheng::prelude]                              │
│  .depth(ScanDepth) ──▶ Set explicit depth                                    │
│  .including_submodules() ──▶ Ergonomic wrapper around .depth(Subtree)       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Detailed Design

### 1. `xuanji` Base Model Addition

Add `ScanDepth` to `crates/xuanji/src/model.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScanDepth {
    /// Default shallow scan: current module/signature level (<10ms).
    #[default]
    Shallow,
    /// Deep subtree scan: recursive submodule and private implementation traversal.
    Subtree,
    /// Audit scan: static AST observation coupled with runtime probe coverage cross-check.
    Audit,
}

impl ScanDepth {
    /// Returns true if this depth is Shallow (used for serde skip_serializing_if).
    pub fn is_shallow(&self) -> bool {
        matches!(self, Self::Shallow)
    }
}
```

Re-export `ScanDepth` in `xuanji::pub use` and `tianheng::prelude::*`.

### 2. DSL Builder Integration

Extend boundary builders (`ModuleBoundary`, `ImplTraitBoundary`, `AsyncExposureBoundary`, etc.):

- Add `.depth(mut self, depth: ScanDepth) -> Self`.
- Retain `.including_submodules(self) -> Self` by mapping it internally to `.depth(ScanDepth::Subtree)`.
- Ensure default construction initializes `depth: ScanDepth::Shallow` (or current default).

### 3. Adversarial Boundaries & Wire Compatibility

- **Static vs. Runtime Boundary**: Static dimensions (`guibiao`, `hunyi`) interpret `Audit` as max-visibility + private item inspection, while runtime probe coverage remains strictly owned by `louke`.
- **Wire Format Zero-Breakage**: Serialized structures use `#[serde(default, skip_serializing_if = "ScanDepth::is_shallow")]` so un-annotated baseline JSON snapshots and SARIF outputs parse without error or schema migration.
