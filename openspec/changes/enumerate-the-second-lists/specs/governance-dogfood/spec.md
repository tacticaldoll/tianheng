## ADDED Requirements

### Requirement: The example suite's declared set SHALL equal the tracked example directories

The dogfood suite SHALL hold its declared example list against the tracked contents of `examples/`, in both
directions. An example present on disk and absent from the list is exercised by **neither** of the suite's
directions nor by the workflow job that runs them, which is a false negative in the gate that runs the product
against itself — the one gate whose silence is least likely to be questioned.

The enumeration SHALL come from tracked content rather than the working directory, so an untracked scratch
directory neither fails the reaction nor is mistaken for an example.

#### Scenario: An example is added and not declared

- **WHEN** a directory under `examples/` carries a manifest and no entry in the declared list names it
- **THEN** the reaction fails, naming the directory, because it would otherwise be exercised by nothing

#### Scenario: A declared example no longer exists

- **WHEN** the declared list names a directory the tracked tree does not carry
- **THEN** the reaction fails, naming the entry, because a declaration that outlived its subject reads as
  coverage while defending nothing
