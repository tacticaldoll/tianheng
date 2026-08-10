## ADDED Requirements

### Requirement: Repository governance vocabulary SHALL preserve product ownership

Live project prose and self-descriptive source comments SHALL use **product** only for crates whose manifests permit publication. **Reaction** SHALL name observable boundary behavior implemented by those product crates: observation, structured outcome or report, process exit class, or runtime event.

An unpublished Rust judgement or test over this repository SHALL be called a **repository check** or **gate**. A Shengmo gate MAY invoke product reactions as dogfood, but the gate itself SHALL NOT be described as a separate reaction. Shell scripts and CI SHALL be described as **workflow orchestration** that invokes gates or irreversible commands and SHALL NOT be assigned a verdict of their own.

#### Scenario: A Kanhe test judges repository records

- **WHEN** prose describes a Kanhe integration test comparing this repository's documents, code, or workflow registration
- **THEN** it calls that executable a repository check or gate, not a product reaction

#### Scenario: Shengmo runs the delivered product against this workspace

- **WHEN** a Shengmo test invokes Tianheng's published observation and outcome path against this repository
- **THEN** prose calls the test a dogfood gate and calls the behavior it invokes the product reaction

#### Scenario: A shell wrapper invokes a Rust judgement

- **WHEN** a shell script sequences a Kanhe gate before merge or publish
- **THEN** prose assigns the judgement and verdict to the Rust gate and calls the shell wrapper workflow orchestration

#### Scenario: Product specifications describe boundary behavior

- **WHEN** a publishable crate observes a governed shape and produces an outcome, report, exit class, or runtime event
- **THEN** its specification retains reaction vocabulary

### Requirement: The repository-check capability SHALL carry its accurate identity

The capability governing unpublished Rust checks over repository records SHALL be named `repository-checks`. Its main spec path, declared bound ids, citations, fixtures, generated projections, and live cross-references SHALL use that identity. The retired `rust-repository-reactions` identity SHALL NOT remain in those live surfaces.

#### Scenario: A renamed bound is stale on one surface

- **WHEN** a declaration, spec citation, or generated projection retains the retired capability prefix
- **THEN** the bound-model/register join or full-file vocabulary sweep fails

#### Scenario: The capability directory and proposal accounting diverge

- **WHEN** the renamed main spec or a change touching its subject uses a different capability name
- **THEN** capability-subject accounting fails rather than silently filing the work under the retired identity
