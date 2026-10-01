# Phase 8: Cost-Aware Model Routing

## Status

Approved implementation slice on `phase8/implementation`.

## Goal

Add a deterministic, provider-independent model routing policy that selects a
registered `ModelProfile` for a logical task and quality mode. The policy must
be usable by a future Narrative Orchestrator without making an API call or
embedding provider-specific logic in the client.

## Scope

This phase includes:

- explicit `ModelTier` metadata on runtime model profiles;
- typed `ModelTask` and `QualityMode` values matching the product's logical
  agent roles and FAST/BALANCED/DEEP modes;
- capability filtering, explicit preferred-model handling and deterministic
  task/tier scoring;
- a typed `model_route` command and TypeScript command contract;
- routing-tier selection in the provider setup form;
- tests for task affinity, quality-mode preferences, capability constraints,
  preferred models, deterministic ties and safe no-route errors;
- architecture and README documentation for the routing boundary.

## Non-goals

This phase does not include:

- Narrative Orchestrator execution plans or multi-step model calls;
- chat, manuscript editing, streaming UI or prompt compilation changes;
- provider failover, retries, budgets, billing data or telemetry;
- semantic retrieval, memory extraction or continuity analysis;
- persisting model profiles in SQLite or syncing routing preferences.

## Routing contract

`ModelRouter` receives registered model profiles and a `RoutingRequest`:

- `task` describes the logical work (`story_architecture`, `main_writing`,
  `memory_extraction`, `continuity_check`, and the other product roles);
- `quality` is `fast`, `balanced` or `deep`;
- `preferred_model` is an optional explicit user choice;
- `required_capabilities` contains only capabilities that must be true;
- `minimum_context_window_tokens` optionally rejects profiles that cannot fit a
  compiled context.

An explicit preferred model wins when it exists and satisfies the request. If
there is no preference, candidates are filtered by capabilities and context
window, scored by task affinity and quality/tier policy, and resolved by
provider/model ID as a stable tie-break. A quality preference is a policy
hint, not a guarantee that an unavailable tier will be fabricated.

The result contains only the selected `ModelRef`, profile, quality mode and a
typed selection reason. No credential, endpoint, prompt or response content
is returned. If no candidate remains, the command returns the safe
`no_suitable_model` error.

## Tier policy

- `fast` favors `local` and `small` profiles for helper work;
- `balanced` favors `medium` and then `large` profiles while retaining useful
  local/small fallbacks;
- `deep` favors `large` and `medium` profiles for architecture, writing and
  major revision work.

Task affinity is derived from profile `strengths`, `weaknesses` and `strategy`
tags. Tags are normalized case-insensitively and remain metadata, not prompt
text. The scoring table is pure Rust and has no provider SDK dependency.

## Validation

- Rust unit tests cover every routing constraint and deterministic ordering.
- Command tests verify snake_case serialization and safe error conversion.
- Frontend tests verify routing tier metadata is submitted with provider setup.
- Existing provider, context, frontend and build checks must remain green.
- Native Windows/Android builds remain environment-dependent and are not part
  of this phase's cloud proof.
