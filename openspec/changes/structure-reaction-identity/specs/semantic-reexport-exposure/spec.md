## ADDED Requirements

### Requirement: Re-export identity is the exported public seam

A re-export exposure fact SHALL encode its forbidden subject and exported module/name path as
separate structured roles. Private source path spelling and human rendering SHALL NOT define the
public seam identity.

#### Scenario: Two exported names stay distinct
- **WHEN** the same subject is re-exported under two public names
- **THEN** the exported-path fields produce two identities
