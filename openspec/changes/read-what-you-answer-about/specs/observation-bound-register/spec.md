## ADDED Requirements

### Requirement: The package enumeration SHALL come from tracked content and refuse rather than shorten

The reaction's package enumeration SHALL read tracked content and SHALL refuse when it fails, rather than
producing a short list. A directory listing that emits some entries and then fails leaves a list that reads as
authoritative, and every citation in a package the reaction never enumerated is then reported as one the
harness does not register — a filesystem failure charged to the register it was reading.

This requirement was already written in this capability's prose and held by nothing: the enumeration read the
working directory with `read_dir` and dropped failed entries.

#### Scenario: The tracked package enumeration fails

- **WHEN** the enumeration of tracked package manifests fails
- **THEN** the reaction refuses as a cannot-judge naming the failure, rather than resolving citations against
  a set it could only partly read

#### Scenario: An untracked directory under the crates root

- **WHEN** a directory carrying a manifest exists in the working tree and in no commit
- **THEN** it is not a package this reaction enumerates, because the tracked set is what the repository holds
