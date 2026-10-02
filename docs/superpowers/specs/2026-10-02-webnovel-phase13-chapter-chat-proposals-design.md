# Phase 13: Chapter Chat and Manuscript Proposals

## Status

Approved implementation slice on `phase13/implementation`.

## Goal

Connect a chapter-scoped conversation to the manuscript workspace while keeping
AI output outside canonical prose until the user explicitly promotes a draft
revision.

## Scope

- chapter ownership for `chapter_chat` conversations;
- a Chapter Chat send command that includes bounded chat history and the current
  manuscript as temporary working context;
- a typed manuscript proposal lifecycle (`draft`, `accepted`, `rejected`);
- proposal promotion through the same transactional manuscript revision path;
- stale-base conflict protection for proposal promotion;
- Chapter Chat and proposal controls in the React workspace;
- Rust, frontend and command-boundary tests.

## Non-goals

- automatic AI writes to canonical manuscript content;
- streaming chat presentation, tool calls or automatic memory extraction;
- semantic retrieval, chapter/arc planning or multi-user collaboration;
- line-level merge algorithms for prose;
- provider-specific prompt formats.

## Data contract

`Conversation.chapter_id` is set for `chapter_chat` and must reference a chapter
in the same project. Developer Chat remains project-scoped. Chapter Chat sends
the most recent bounded messages plus the current manuscript body as transient
working memory; neither becomes canonical memory.

`ManuscriptProposal` stores a complete proposed body, content format, rationale
and the manuscript revision it was based on. Creating a proposal is non-
canonical. Promotion checks that the manuscript is still at the proposal base
revision, writes a new immutable manuscript revision and marks the proposal
accepted in one transaction. A stale base returns `conflict`; reject is explicit
and never changes the manuscript.

## Validation

- migration is idempotent and enforces chapter-chat ownership in the service;
- Rust tests cover chapter conversation ownership, bounded manuscript context,
  proposal lifecycle and stale promotion conflicts;
- frontend tests cover Chapter Chat persistence, proposal actions and conflict
  messaging;
- existing Rust, frontend, lint and build checks remain required.
