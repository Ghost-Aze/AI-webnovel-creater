# Phase 10: Developer Chat and Memory Tools

## Status

Approved implementation slice on `phase10/implementation`.

## Goal

Provide the durable Developer Chat foundation that can sit above the Phase 9
orchestrator. Conversation history is stored separately from canonical memory,
and AI memory actions enter the existing proposal/revision path instead of
mutating entities directly.

## Scope

- SQLite migrations for project conversations and ordered messages;
- typed conversation/message domain, repository and service operations;
- a `developer_chat_send` command that persists the user turn, invokes the
  orchestrator with recent chat history as temporary working memory, and
  persists only the final assistant turn;
- a typed `memory_tool_propose` boundary for character updates that creates a
  draft proposal through `ProposalService`;
- TypeScript command/type contracts and Rust/frontend tests.

## Non-goals

- chat UI, streaming presentation or editor/manuscript documents;
- direct AI writes to canonical Character or CharacterState records;
- automatic proposal promotion, memory extraction or semantic retrieval;
- sync, conflict resolution, new provider adapters or persistent orchestration
  run history.

## Data contract

`Conversation` belongs to a project and has a typed kind. Phase 10 creates
`developer_chat`; `arc_chat` and `chapter_chat` are reserved for later phases.
`ConversationMessage` stores an ordered role (`system`, `user`, `assistant` or
`tool`) and content, with optional provider/model metadata for assistant turns.

`developer_chat_send` appends the user message before execution. Previous
messages are passed as bounded working-memory blocks to Phase 9. On success,
the final orchestrator response is appended as an assistant message. If the
provider fails, the user turn remains durable and no partial assistant turn is
created.

`memory_tool_propose` accepts a typed character update action and creates a
`DRAFT` proposal with `actor_type = ai`. Promotion remains an explicit user
operation and still enforces revision conflicts and locked canon rules.

## Validation

- migration and repository tests cover ordering, project ownership and archive
  behavior;
- chat tests cover durable user/assistant turns, bounded history handoff and
  provider failure behavior;
- memory-tool tests prove proposal creation without canonical mutation;
- existing Rust, frontend and build checks remain required.
