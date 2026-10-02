# Phase 9: Narrative Orchestrator Core

## Status

Approved implementation slice on `phase9/implementation`.

## Goal

Add a provider-independent Narrative Orchestrator that turns a logical
writing task and quality mode into a deterministic execution plan, routes each
step through `ModelRouter`, compiles only the requested context, and executes
provider calls through the existing `AIProvider` contract.

## Scope

- typed logical-agent steps and deterministic FAST/BALANCED/DEEP plans;
- sequential execution with explicit working-memory handoff between steps;
- model routing and context compilation for every step;
- safe provider response/result types and a typed `orchestrator_run` command;
- Rust and TypeScript command contracts plus mock-provider tests;
- architecture and phase documentation.

## Non-goals

- Developer Chat, manuscript/editor UI or streaming presentation;
- automatic canonical-memory writes, memory extraction or continuity mutation;
- retries, billing, provider failover, background jobs or persistent run history;
- semantic retrieval, sync, export or new provider adapters.

## Execution contract

`OrchestrationRequest` supplies the project/context selection, user prompt,
logical task, quality mode and routing constraints. The planner creates a
small deterministic sequence:

- FAST: planner, writer, and memory-curator summary;
- BALANCED: planner, writer, continuity critic, and memory curator;
- DEEP: planner, writer, continuity critic, style critic, revision, and
  memory curator.

Planning-only and helper tasks use a single task-appropriate step instead of
inventing a writer pipeline. Each step routes independently, compiles its
context with the selected model budget, and receives previous step outputs as
temporary working memory. Outputs are returned to the caller only; no
canonical entity is changed.

The orchestrator is sequential in this phase. A future execution engine may
batch compatible steps or stream responses without changing the typed plan
boundary.

## Validation

- Rust tests cover plan shapes, task mapping, context handoff, provider call
  ordering and safe error propagation.
- Command tests verify the typed `orchestrator_run` helper.
- Frontend tests verify snake_case serialization for the new command contract.
- Existing Rust, frontend and build checks remain required.
