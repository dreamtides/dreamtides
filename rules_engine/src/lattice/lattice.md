---
lattice-id: LCEWQN
name: lattice
description: |-
  Root document for the Lattice crate - a unified knowledge base and task
  tracking system built on markdown files with SQLite indexing.
created-at: 2026-01-19T05:15:00.000000Z
updated-at: 2026-01-19T05:15:00.000000Z
---

# Lattice

Lattice is a unified knowledge base and task tracking system built on markdown
files stored in git repositories, with SQLite providing an ephemeral index for
query performance.

## Overview

The core innovation of Lattice is treating markdown documents as first-class
database entities while maintaining full git compatibility and human
readability. Documents can exist anywhere in a project hierarchy, colocated
with relevant code, and are identified by their `lattice-id` YAML annotation
rather than filesystem location.

## Key Features

- Document atomicity with 500-line soft limit
- Rich cross-referencing with bidirectional link tracking
- Task tracking integrated with knowledge base documents
- Full-text search via SQLite FTS5
- Git as single source of truth (SQLite is just a cache)
- Claude Skill integration for AI workflows

## Design Documentation

See the main design document at `rules_engine/docs/lattice/lattice_design.md`
for complete technical specifications.

## Implementation

The implementation is in `rules_engine/src/lattice/src/` with modules for:

- `cli` - Command-line interface
- `index` - SQLite indexing and queries
- `document` - Document parsing and manipulation
- `git` - Git integration
- `format` - Document formatting
- `link` - Link parsing and normalization
- `lint` - Document validation
- `id` - Lattice ID generation
