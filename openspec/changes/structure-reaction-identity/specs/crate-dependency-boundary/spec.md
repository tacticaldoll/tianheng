## ADDED Requirements

### Requirement: Dependency observations use semantic reaction identity

Every dependency violation SHALL identify the governed crate, a structured rule key, and a
structured dependency fact whose fields preserve dependency name, dependency kind, source/feature
roles when observed. Human finding text SHALL NOT provide version-1 compatibility or identity.

#### Scenario: Dependency kind remains injective
- **WHEN** the same dependency appears as normal and development edges that both violate a boundary
- **THEN** their structured facts differ by dependency kind without relying on rendered suffixes

#### Scenario: Presentation cannot provide legacy matching
- **WHEN** a numeric or text-identity baseline carries the same displayed dependency
- **THEN** baseline parsing fails rather than matching it to the current fact
