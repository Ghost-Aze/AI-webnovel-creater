# Phase 15 Project Memory Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add revision-safe, provider-independent StoryFact and CanonRule project memory with explicit proposal promotion and explicit Context Compiler selection.

**Architecture:** Keep canonical Project Memory rows in structured SQLite tables owned by a Rust project_memory repository and service. Reuse the shared revision and proposal boundaries, adding typed entity handling and a dedicated archived_memory error. Extend ContextSource only with explicit fact/rule reference loading; do not add automatic extraction, semantic retrieval or a React Project Bible screen.

**Tech Stack:** Rust, rusqlite, serde, Tauri commands, SQLite migrations, React/TypeScript typed invoke wrappers, Vitest, Cargo test/clippy/fmt.

**Spec:** docs/superpowers/specs/2026-10-03-webnovel-phase15-project-memory-core-design.md

## Global Constraints

- Work only on phase13/implementation; do not merge main and do not begin another phase.
- Preserve the provider-independent Rust boundary, structured SQLite data and explicit user-controlled canonical writes.
- Keep SQL and row mapping inside src-tauri/src/project_memory and keep business logic out of React.
- New facts and rules start at revision 1, status active, canon status canon; archive is idempotent and archived rows remain readable but are not writable.
- AI proposals remain draft until explicit promotion; an AI provider never writes a canonical fact or rule directly.
- Create proposals use entity_id = null and base_revision = 0; promotion generates and records the canonical entity id atomically.
- Context compilation loads only explicitly selected project_memory_refs, preserves request order, and never scans all project memory.
- Do not add relationships, semantic/vector search, automatic memory extraction, continuity AI, sync, collaboration, export, authentication, settings UI or a Project Bible React screen.
- Do not commit API keys, credentials, SQLite runtime data, node_modules, Rust targets or generated Tauri output.
- Run the repository checks after source changes: npm run typecheck, npm run lint, npm test -- --run, npm run build, cargo fmt --all -- --check, cargo clippy --all-targets -- -D warnings, and cargo test.

## Review Focus

- Legacy revision/proposal rows, columns and indexes survive migration 0009 and a second migration run is a no-op. Test this in the migration task.
- Cross-project, archived-project, archived-memory, locked-canon and stale-revision writes return safe typed errors without partial rows or revision entries. Test these in the CRUD/revision task.
- Create and update proposals validate their entity-specific payloads; promotion is atomic, idempotence-safe and records the generated id for create proposals. Test this in the proposal task.
- Restore and canon-status operations create forward revisions and never silently revive or rewrite archived/locked records. Test this in the revision task.
- Context selection loads only requested fact/rule references, rejects missing or cross-project ids, preserves order and reports budget omissions. Test this in the context task.

### Task 1: Migration, domain contracts and shared error/enums

**Files:**
- Create: migrations/0009_project_memory.sql
- Create: src-tauri/src/domain/project_memory.rs
- Modify: src-tauri/src/domain/mod.rs
- Modify: src-tauri/src/domain/revision.rs
- Modify: src-tauri/src/error.rs
- Modify: src-tauri/src/db.rs
- Modify: src-tauri/src/revisions/repository.rs
- Test: src-tauri/src/db.rs migration tests and src-tauri/src/domain/project_memory.rs unit tests

**Interfaces:**
- Produces ProjectMemoryStatus::{Active, Archived}, ProjectMemoryListFilter, StoryFact, CanonRule, CreateStoryFactInput, UpdateStoryFactInput, CreateCanonRuleInput, UpdateCanonRuleInput, build_*/update_*_fields normalization helpers, and MemoryEntityType::{StoryFact, CanonRule}.
- Produces serialized AppError::ArchivedMemory with code archived_memory and a safe frontend-facing message.
- Keeps existing MemoryRevision, MemoryProposal, Character and CharacterState wire values backward compatible.

- [ ] Step 1: Write failing migration and domain tests. Add tests for deterministic migration idempotence, creation of the two tables and indexes, preservation of seeded legacy memory_revisions/memory_proposals rows and existing index names during the table rebuild, trimming/rejection of blank titles/content/rule/scope, default revision/status/canon values, and serde round trips for story_fact and canon_rule.
- [ ] Step 2: Run the focused Rust tests and verify they fail for the missing migration/domain contracts.

  Run: cargo test db::tests -- --nocapture && cargo test project_memory -- --nocapture

  Expected: FAIL because migration 0009, the new domain module, enum variants and archived_memory error do not yet exist.
- [ ] Step 3: Implement the domain and migration contract. Define typed entities and normalization with new_id()/now_utc(). Add 0009_project_memory.sql with story_facts and canon_rules, project/status indexes, and transactional replacement tables for memory_revisions and memory_proposals that add the two entity values while copying every existing column and row and recreating existing indexes. Register the migration in db::MIGRATIONS, expose the domain module, add the error variant/code, and extend revision enum serialization/parsing.
- [ ] Step 4: Run the focused tests and inspect the schema.

  Run: cargo test db::tests -- --nocapture && cargo test project_memory -- --nocapture

  Expected: PASS, with no dropped legacy rows, one _migrations record for 0009_project_memory.sql, and normalized domain values.
- [ ] Step 5: Commit the task.

  ~~~bash
  git add migrations/0009_project_memory.sql src-tauri/src/db.rs src-tauri/src/domain src-tauri/src/error.rs src-tauri/src/revisions/repository.rs
  git commit -m "feat: add project memory schema and contracts"
  ~~~

### Task 2: Project Memory CRUD repository and service

**Files:**
- Create: src-tauri/src/project_memory/mod.rs
- Create: src-tauri/src/project_memory/repository.rs
- Create: src-tauri/src/project_memory/service.rs
- Modify: src-tauri/src/lib.rs
- Test: src-tauri/src/project_memory/service.rs unit tests and src-tauri/tests/project_memory_flow.rs

**Interfaces:**
- ProjectMemoryRepository::new(connection: SharedConnection) -> Self owns SQL, row mapping and transactional revision inserts.
- ProjectMemoryService::new(repository: ProjectMemoryRepository, projects: ProjectRepository) -> Self owns active-project, archive and locked-canon policy.
- Typed methods: create_story_fact, list_story_facts, get_story_fact, update_story_fact, archive_story_fact, plus the corresponding five canon_rule methods; update/archive take expected_revision: u64.
- Repository create/update/archive operations append MemoryRevision rows with Create, Update or Archive, actor User, source manual, and before/after snapshots in the same transaction.

- [ ] Step 1: Write failing CRUD/revision tests. Cover create normalization and revision 1, list filtering with archived records excluded by default, get/list reads from an archived project, update revision increments, archive idempotence, active-project enforcement, archived_memory, archived_project, locked_canon, conflict, not_found, and revision history snapshots for both entity types.
- [ ] Step 2: Run the focused integration test and verify the new service is absent/failing.

  Run: cargo test --test project_memory_flow -- --nocapture

  Expected: FAIL because project_memory and its typed operations are not implemented.
- [ ] Step 3: Implement repository SQL and row mapping. Mirror existing Character/Manuscript repository patterns, use ProjectMemoryStatus and CanonStatus parsers, preserve archived reads, and guard every update/archive with WHERE id = ? AND revision = ? inside an explicit transaction.
- [ ] Step 4: Implement service policy and wire the module. Normalize inputs before repository calls, require an active project for writes, allow explicit archived listing, reject archived memory and locked canon writes with their typed errors, and expose the module from src-tauri/src/lib.rs.
- [ ] Step 5: Run the focused tests and verify all CRUD, policy and revision assertions pass.
- [ ] Step 6: Commit the task.

  ~~~bash
  git add src-tauri/src/project_memory src-tauri/src/lib.rs src-tauri/tests/project_memory_flow.rs
  git commit -m "feat: add project memory CRUD service"
  ~~~

### Task 3: Shared revision restore and canon-status support

**Files:**
- Modify: src-tauri/src/revisions/service.rs
- Modify: src-tauri/src/project_memory/repository.rs
- Modify: src-tauri/src/project_memory/service.rs
- Modify: src-tauri/tests/revision_flow.rs
- Test: src-tauri/tests/revision_flow.rs and project-memory revision tests

**Interfaces:**
- RevisionService receives a ProjectMemoryRepository and matches MemoryEntityType::StoryFact/CanonRule in restore and set_canon_status.
- Project-memory restore methods accept (entity_id, expected_revision, target_revision) and create a new forward Restore revision; canon-status methods accept (entity_id, status, expected_revision) and create a CanonStatus revision.
- Restore and canon-status operations use the same active-project, archived-memory, locked-canon and optimistic-revision rules as CRUD writes.

- [ ] Step 1: Add failing revision tests. Assert history contains create/update/restore or canon-status entries, restore increments from the current revision instead of reusing the target number, stale restore/status returns Conflict, and archived or locked records are not changed.
- [ ] Step 2: Run cargo test --test revision_flow -- --nocapture and verify the new enum branches fail.
- [ ] Step 3: Implement typed project-memory restore/status repository operations and extend RevisionService matches without changing Character/CharacterState behavior.
- [ ] Step 4: Run cargo test --test revision_flow -- --nocapture and the project-memory flow tests; verify all pass.
- [ ] Step 5: Commit the task.

  ~~~bash
  git add src-tauri/src/revisions/service.rs src-tauri/src/project_memory src-tauri/tests/revision_flow.rs
  git commit -m "feat: extend project memory revision lifecycle"
  ~~~

### Task 4: Typed AI proposals and atomic promotion

**Files:**
- Modify: src-tauri/src/revisions/service.rs
- Modify: src-tauri/src/project_memory/repository.rs
- Modify: src-tauri/src/lib.rs
- Modify: src-tauri/tests/proposal_flow.rs
- Test: src-tauri/tests/proposal_flow.rs and project-memory integration tests

**Interfaces:**
- ProposalService receives ProjectMemoryRepository in addition to the existing revision/character dependencies.
- create accepts only typed StoryFact/CanonRule Create proposals with entity_id = None, base_revision = 0, or typed Update proposals with an owned entity id and matching positive base revision; all other new-entity shapes return InvalidProposal.
- promote atomically creates or updates the canonical row, appends the corresponding MemoryRevision, sets proposal status to accepted, and for create proposals sets the generated entity_id before commit.

- [ ] Step 1: Write failing proposal tests. Cover valid create/update payloads for both types, malformed payloads, wrong operation/entity-id/base combinations, cross-project ids, stale bases, archived/locked targets, atomic acceptance, generated create ids, and idempotent reject/second-promote behavior.
- [ ] Step 2: Run cargo test --test proposal_flow -- --nocapture and verify new proposal cases fail.
- [ ] Step 3: Implement typed validation in ProposalService. Deserialize payloads into the exact domain input types, validate project ownership/status, and preserve Character proposal behavior.
- [ ] Step 4: Implement repository promotion transactions. Lock the draft/current row by querying within one transaction, guard current revision/status/canon, write canonical data and revision, update proposal id/status, and roll back all changes on any conflict or invalid lifecycle state.
- [ ] Step 5: Run proposal, revision and project-memory tests; verify no canonical row changes on failed promotion.
- [ ] Step 6: Commit the task.

  ~~~bash
  git add src-tauri/src/revisions/service.rs src-tauri/src/project_memory src-tauri/src/lib.rs src-tauri/tests/proposal_flow.rs
  git commit -m "feat: add project memory proposal promotion"
  ~~~

### Task 5: Tauri command boundary and application wiring

**Files:**
- Modify: src-tauri/src/commands.rs
- Modify: src-tauri/src/lib.rs
- Modify: src-tauri/src/commands.rs tests
- Test: command helper tests in src-tauri/src/commands.rs

**Interfaces:**
- Add helpers and registered Tauri commands: story_fact_create/list/get/update/archive and canon_rule_create/list/get/update/archive.
- Create/list/get signatures carry project_id, typed input/id and ProjectMemoryListFilter; update/archive always carry expected_revision.
- Existing memory_history_list, memory_restore, memory_set_canon_status and memory_proposal_* commands accept the extended enum without changing snake_case names or safe error mapping.
- AppState owns one shared ProjectMemoryService; startup constructs it from the application database and passes it to RevisionService, ProposalService and later ServiceContextSource.

- [ ] Step 1: Add failing command contract tests. Assert every new command helper delegates to the exact command name and snake_case payload, expected revisions are present, and archived_memory serializes without SQL/provider details.
- [ ] Step 2: Run the focused command tests and verify missing helpers/registrations fail.
- [ ] Step 3: Implement command helpers, Tauri adapters, AppState construction and generate_handler! registrations. Keep adapters thin and return AppResult values from services.
- [ ] Step 4: Run cargo test commands::tests -- --nocapture and verify command assertions pass.
- [ ] Step 5: Commit the task.

  ~~~bash
  git add src-tauri/src/commands.rs src-tauri/src/lib.rs
  git commit -m "feat: expose project memory commands"
  ~~~

### Task 6: Explicit Context Compiler integration

**Files:**
- Modify: src-tauri/src/context/types.rs
- Modify: src-tauri/src/context/compiler.rs
- Modify: src-tauri/src/context/source.rs
- Modify: src-tauri/src/orchestration/mod.rs
- Modify: src-tauri/src/lib.rs
- Modify: src-tauri/src/commands.rs context tests
- Test: src-tauri/tests/context_flow.rs and src-tauri/src/context/compiler.rs

**Interfaces:**
- Add ProjectMemoryRef { entity_type: MemoryEntityType, entity_id: String } and project_memory_refs: Vec<ProjectMemoryRef> to ContextCompileRequest.
- Add ContextBlockKind::{StoryFact, CanonRule} and ContextSource::load_story_fact(project_id: &str, entity_id: &str)/load_canon_rule(project_id: &str, entity_id: &str) methods that enforce project ownership.
- ServiceContextSource::new receives ProjectMemoryService and loads only the requested records.
- Compiler priority is 70 for both project-memory kinds, after the project block and before character/character-state/working-memory blocks; selected reference order is preserved and content is formatted as typed fact/rule blocks.
- Existing orchestration context request builders populate an empty reference list so current chat/orchestration behavior remains unchanged unless a caller explicitly selects project-memory refs.

- [ ] Step 1: Add failing compiler/source tests. Assert selected fact/rule blocks, request order, cross-project/missing rejection, no unselected memory loading, budget truncation and omission reporting, plus serde payload shape.
- [ ] Step 2: Run cargo test context -- --nocapture and cargo test --test context_flow -- --nocapture; verify the new types/source methods are missing.
- [ ] Step 3: Implement source loading and compiler candidates. Load each ref through typed service methods, preserve the existing budget algorithm, and return NotFound for missing or cross-project refs before provider resolution.
- [ ] Step 4: Update every Rust ContextCompileRequest literal and orchestration builder with the new field; run context, orchestration and conversation tests.
- [ ] Step 5: Commit the task.

  ~~~bash
  git add src-tauri/src/context src-tauri/src/orchestration/mod.rs src-tauri/src/commands.rs src-tauri/tests/context_flow.rs
  git commit -m "feat: compile explicitly selected project memory"
  ~~~

### Task 7: TypeScript contracts, command wrappers and client tests

**Files:**
- Create: apps/client/src/types/project-memory.ts
- Modify: apps/client/src/types/revision.ts
- Modify: apps/client/src/types/context.ts
- Modify: apps/client/src/lib/commands.ts
- Modify: apps/client/src/lib/commands.test.ts
- Modify: apps/client/src/lib/command-error.ts

**Interfaces:**
- Export serde-compatible ProjectMemoryStatus, ProjectMemoryListFilter, StoryFact, CanonRule, create/update inputs and ProjectMemoryRef types.
- Extend MemoryEntityType to "story_fact" | "canon_rule" and ContextBlockKind to "story_fact" | "canon_rule".
- Add typed wrappers matching the Rust commands and use expected_revision for update/archive/restore/status/promotion calls.
- Add archived_memory to CommandErrorCode and map it to a safe user message.

- [ ] Step 1: Add failing TypeScript command/type tests. Assert all new invoke names, exact snake_case arguments, context refs, extended memory entity values and archived_memory normalization.
- [ ] Step 2: Run npm test -- --run src/lib/commands.test.ts and verify the new imports/types/wrappers fail.
- [ ] Step 3: Implement the types, wrappers and safe error mapping without adding a React screen or provider-specific client logic.
- [ ] Step 4: Run the focused Vitest file, then npm run typecheck and npm run lint; verify all pass.
- [ ] Step 5: Commit the task.

  ~~~bash
  git add apps/client/src/types apps/client/src/lib/commands.ts apps/client/src/lib/commands.test.ts apps/client/src/lib/command-error.ts
  git commit -m "feat: add project memory client contracts"
  ~~~

### Task 8: Architecture/development documentation and full verification

**Files:**
- Modify: docs/ARCHITECTURE.md
- Modify: docs/DEVELOPMENT_CONTEXT.md
- Modify: docs/ARCHITECTURE_DECISIONS.md
- Test: repository verification commands and git diff --check

**Interfaces:**
- Documentation records Phase 15 entities, migration 0009, archived_memory, revision/proposal/context boundaries, explicit selection and all deferred non-goals.
- Documentation keeps the approved branch and Phase 14/Phase 15 boundary accurate; no UI implementation is added.

- [ ] Step 1: Update the architecture command list, domain inventory, context boundary, error list and development continuation state. Record that Phase 15 is implemented only after source work is complete; until then keep the plan/spec state accurate.
- [ ] Step 2: Run git diff --check and inspect the diff for generated files, secrets, SQLite data or scope creep.
- [ ] Step 3: Run the complete required gate.

  ~~~text
  npm run typecheck
  npm run lint
  npm test -- --run
  npm run build
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  ~~~

  Expected: all required checks pass. If the repository-wide Prettier check is run separately, report its known pre-existing drift without mass-formatting unrelated files.
- [ ] Step 4: Commit the documentation and verification-ready source state.

  ~~~bash
  git add docs/ARCHITECTURE.md docs/DEVELOPMENT_CONTEXT.md docs/ARCHITECTURE_DECISIONS.md
  git commit -m "docs: record phase 15 project memory architecture"
  ~~~

- [ ] Step 5: Confirm branch and worktree hygiene. Verify git branch --show-current is phase13/implementation, git status --short contains only intentionally ignored generated Tauri output, and no main merge or unrelated phase work exists.
