# Phase 15: Project Memory Core and Project Bible Foundation

## Status

Design approved in conversation; written spec is awaiting user review before
implementation planning.

## Goal

Add the first structured Project Memory slice so a project can store durable
story facts and canon rules independently from chats and manuscript text. The
slice must preserve explicit canonical writes, revision safety, provider
independence and bounded context compilation.

The Project Bible remains a derived user-facing view of structured project
memory. This phase establishes its backend foundation and typed read/write
boundary; it does not build the final visual Project Bible screen.

## Scope

- typed `StoryFact` and `CanonRule` project entities;
- idempotent SQLite migration `0009_project_memory.sql`;
- archive status, canon status and optimistic revision checks;
- revision history for the new entities through the existing revision boundary;
- typed AI proposals with explicit user-controlled promotion;
- typed Tauri and TypeScript CRUD/command contracts;
- explicit selection of project-memory records for Context Compiler input;
- Rust, integration and command-contract tests;
- durable architecture and development-context documentation.

## Non-goals

- Project Bible visual redesign or a new React Project Bible screen;
- relationships, locations, factions, arcs, plot threads, scenes, secrets or
  foreshadowing entities;
- semantic search, vector indexes or relationship graphs;
- automatic memory extraction from chapters or chats;
- continuity AI, streaming, retry/failover or provider-specific workflows;
- authentication, cloud sync, collaboration or export;
- automatic canonical writes by an AI provider.

## Canonical entities

### StoryFact

`StoryFact` stores a durable project statement such as a premise detail,
world fact or established story truth.

- `id TEXT PRIMARY KEY`;
- `project_id TEXT NOT NULL REFERENCES projects(id)`;
- `title TEXT NOT NULL`;
- `content TEXT NOT NULL`;
- `status TEXT NOT NULL CHECK (status IN ('active', 'archived'))`;
- `canon_status TEXT NOT NULL CHECK (canon_status IN ('canon', 'locked_canon'))`;
- `revision INTEGER NOT NULL CHECK (revision > 0)`;
- `created_at TEXT NOT NULL`;
- `updated_at TEXT NOT NULL`.

Titles and content are trimmed and may not be blank. New facts start at
revision 1 with `active` and `canon` status. Archiving is idempotent. Archived
facts remain readable but cannot be edited or promoted.

### CanonRule

`CanonRule` stores a durable constraint such as a world, style or plot rule.

- `id TEXT PRIMARY KEY`;
- `project_id TEXT NOT NULL REFERENCES projects(id)`;
- `title TEXT NOT NULL`;
- `rule TEXT NOT NULL`;
- `scope TEXT NOT NULL`;
- `status TEXT NOT NULL CHECK (status IN ('active', 'archived'))`;
- `canon_status TEXT NOT NULL CHECK (canon_status IN ('canon', 'locked_canon'))`;
- `revision INTEGER NOT NULL CHECK (revision > 0)`;
- `created_at TEXT NOT NULL`;
- `updated_at TEXT NOT NULL`.

Titles, rule text and scope are trimmed and may not be blank. New rules start
at revision 1 with `active` and `canon` status. Scope remains typed non-empty
text in this phase so future rule categories do not require a migration.

Both entities are project-scoped. Writes require an active project; reads may
list archived records when explicitly requested.

## Migration and revision compatibility

Migration `0009_project_memory.sql` creates `story_facts`, `canon_rules` and
their project/status indexes. It also extends the existing revision and
proposal entity constraints to accept `story_fact` and `canon_rule` while
preserving all existing Character and CharacterState rows. SQLite table rebuild
steps must run inside the migration transaction: create replacement tables,
copy every existing column and row, recreate the existing indexes with their
current names plus the new constraints, then replace the old tables. No
existing revision or proposal row may be dropped or rewritten.

The Rust `MemoryEntityType` wire enum gains `StoryFact` and `CanonRule`. The
existing `memory_revisions` table remains the shared append-only log. New
entity updates write the canonical row and its revision record in one
transaction with actor, base revision, before/after values and source metadata.

## Domain, repository and service boundaries

`src-tauri/src/domain/project_memory.rs` owns serializable entities, update
inputs, status values and normalization rules. `src-tauri/src/project_memory`
owns SQL, row mapping and service policy. Business logic stays out of React.

The service exposes typed operations equivalent to:

- `create_story_fact(project_id, input)`;
- `list_story_facts(project_id, filter)`;
- `get_story_fact(id)`;
- `update_story_fact(id, input, expected_revision)`;
- `archive_story_fact(id, expected_revision)`;
- the corresponding five `canon_rule` operations.

Updates, archives and restores reject archived projects, archived entities, locked canon
and stale revisions with existing safe error codes. The existing
`memory_restore` command is extended to `story_fact` and `canon_rule`; it
creates a new revision from the selected historical snapshot and never moves
the current revision number backward. A missing record maps to `not_found`;
invalid input maps to `validation`; storage details never cross the command
boundary.

## Proposal boundary

The existing `memory_proposals` path is extended with typed validation for the
two new entity types.

- AI create proposals use `entity_id = null` and `base_revision = 0`. Promotion
  generates the canonical entity id inside the same transaction, stores that id
  in the accepted proposal record, and returns the create revision for the new
  entity.
- AI update proposals include the existing entity id and its loaded revision.
- Payloads are validated against the selected entity type before storage.
- Promotion checks project status, entity status, canon status and base
  revision in one transaction.
- Successful promotion writes the canonical row, the next revision record and
  accepted proposal state atomically.
- Rejection changes only proposal status.

AI may propose a fact or rule, but never applies the canonical change directly.
The existing `memory_history_list`, `memory_restore`,
`memory_set_canon_status` and proposal lifecycle commands are extended with the
new entity enum values.

## Context Compiler integration

`ContextCompileRequest` gains an explicit `project_memory_refs` collection.
Each reference contains a `MemoryEntityType` (`story_fact` or `canon_rule`)
and an entity id. The compiler loads only those records through
`ContextSource`; it does not scan all project memory.

`ContextBlockKind` gains `StoryFact` and `CanonRule`. Blocks are ordered after
the project block and before character, character-state and working-memory
blocks. Within the selected references, request order is preserved; fact and
rule blocks use the same project-memory priority and are truncated or omitted
by the existing model budget policy. Missing or cross-project references return
safe `not_found` errors. No provider is called during compilation.

Existing character selection and working-memory behavior remains unchanged.

## Command and client contracts

Rust registers:

- `story_fact_create`, `story_fact_list`, `story_fact_get`,
  `story_fact_update`, `story_fact_archive`;
- `canon_rule_create`, `canon_rule_list`, `canon_rule_get`,
  `canon_rule_update`, `canon_rule_archive`.

The client adds serde-compatible types and snake_case wrappers. Update and
archive wrappers always send `expected_revision`. No settings or Project Bible
screen is added in this phase.

## Verification

Rust tests cover migration idempotence/data preservation, entity normalization,
project/archive/lock policy, stale updates, revision history, proposal payload
validation and atomic promotion. Context tests cover selected fact/rule blocks,
cross-project rejection, budget truncation and omission reporting. Command
tests assert snake_case names, typed payloads and safe error mapping.

The full repository gate remains:

```text
npm run typecheck
npm run lint
npm test -- --run
npm run build
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Future compatibility

`StoryFact` and `CanonRule` are the first typed Project Memory records, not a
generic JSON memory bucket. Later entities can reuse the revision/proposal and
Context Compiler boundaries without changing the existing Project, Character,
Chapter or Manuscript contracts. Relationships, timeline events, plot threads,
semantic retrieval and continuity checks remain separately designed phases.
