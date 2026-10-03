# Webnovel AI Studio Jan-Inspired Client UI

## Status

Design approved in conversation; written spec is approved for planning.

## Goal

Rework the existing React client so the Provider settings and project chat
surfaces have the information hierarchy shown in the supplied Jan references.
The client should feel like one focused AI workspace: provider/model choices are
easy to find, an empty chat invites the first message, and an active chat keeps
the composer available at the bottom of the viewport.

The redesign is presentation and interaction work inside the existing Tauri
client. Rust commands, SQLite boundaries, provider-independent contracts,
secure credential storage and explicit canonical writes remain unchanged.

## Provider settings experience

`/settings/providers` becomes a settings workspace with a secondary settings
navigation and a provider navigation section. The provider list is populated
from `provider_list`; it is not a hardcoded list of Jan providers. The first
configured provider is selected on entry, and OpenRouter remains the default
draft when no provider exists.

The provider navigation shows each configured provider and an add action. The
selected provider owns the content area, which contains:

- provider identity, endpoint and secure credential status;
- a model list with display name, model ID, tier and streaming capability;
- edit, favorite/selection and remove actions for each model or provider where
  the existing command contract supports them;
- an add-model action that keeps the existing validation rules;
- an explicit connection test with a safe success or failure state.

Editing uses `provider_get` and `provider_update`; saving a blank API-key field
keeps the existing secure credential. `provider_test` reports a safe message
without exposing response text or credentials. The screen must continue to
work when metadata exists but the platform credential is unavailable: the
provider remains visible and editable, while model execution stays inactive.

## Chat experience

Developer Chat and Chapter Chat share the same visual conversation shell while
keeping their existing project/chapter scopes and request contracts.

### Empty state

Before the first message, the chat area uses a centered welcome state. The
composer is centered with the prompt, as in the reference image. The composer
contains the message field, assistant selector, provider/model selector,
quality selector and compact tool affordances. It is the primary call to
action, not a separate settings panel.

### Active conversation

After a message exists, the layout changes to a full-height conversation:

- the message list becomes the scrolling region;
- the composer moves to a bottom-pinned position inside the chat surface;
- the header keeps the selected model/provider visible;
- the composer remains available while the user scrolls history;
- sending shows a pending state and prevents duplicate submission.

The transition is state-driven by the message list, not by viewport width or a
timer. It must preserve the draft when loading or command errors occur.

### Error and retry state

Provider failures render an inline error banner in the conversation with a
short safe explanation and a retry action. The banner does not show raw HTTP
bodies, prompts or credentials. Retry repeats the current user attempt through
the existing typed chat command and keeps the composer at the bottom. If no
retryable attempt is available, the action is omitted and the user can submit
the preserved draft again.

Unauthorized or unavailable credentials link to Provider settings. Error
states are visually distinct from an empty chat and do not replace the message
history.

The error explanation is derived from a typed provider failure classification,
not from a generic fallback string. The Rust provider adapter maps the
following outcomes to safe application error codes: unauthorized or forbidden
credentials, missing model or endpoint, rate limiting, provider unavailability
(HTTP 5xx), request timeout or network reachability, and invalid provider
request. The frontend maps those codes to stable copy and retry behavior:
credential and endpoint errors link to Provider settings, rate limits and
temporary provider failures offer retry, and invalid requests ask the user to
review the selected model or chat options. Raw response bodies remain
discarded at the provider boundary, so provider diagnostics never expose
credentials, prompts or arbitrary response text.

The retry action marks the existing failed user attempt through the typed chat
request contract. The conversation service reuses that last user turn instead
of appending a duplicate; a normal new submission continues to append a new
turn.

## Layout and responsive behavior

Desktop keeps the global application rail, a project-scoped navigation group,
the main workspace and the existing context/status areas. Provider settings
adds its own secondary navigation inside the main workspace rather than
duplicating global navigation.

On narrow screens, project/settings navigation becomes a drawer or compact
sheet, the bottom navigation remains the primary route switcher, and the chat
composer respects the on-screen keyboard and safe-area padding. The active chat
message list must remain scrollable without moving the bottom composer offscreen.

## Component boundaries

The implementation should introduce focused client components instead of
moving business logic into layout files:

- `SettingsLayout` and `ProviderNavigation` own settings/provider selection;
- `ProviderModelList` and `ProviderDetailsPanel` render provider metadata and
  actions;
- `ChatHeader`, `ChatEmptyState`, `ChatMessageList`, `ChatComposer` and
  `ChatErrorBanner` own the visual chat states;
- existing typed command wrappers remain the only client/backend boundary.

The chat panels continue to own conversation loading and request state, while
the new visual components receive typed props and callbacks.

## Accessibility and safety

All selectors and actions have visible labels or accessible names. Keyboard
focus moves to the composer when a new empty chat opens, remains recoverable
after an error, and is not trapped by a drawer. Loading, sending and error
states use status/alert semantics. API keys remain password fields and never
appear in rendered text, logs or serialized chat state.

## Validation

The redesign is accepted when:

1. Provider settings can select a provider, inspect its models, edit metadata,
   test a connection and remove it through the existing commands.
2. A new chat renders a centered composer, while a chat with messages renders a
   scrollable history and bottom-pinned composer.
3. Sending, loading, archived/read-only and provider-error states preserve the
   existing behavior and show safe, actionable UI.
4. Desktop and narrow viewport tests cover the layout state transition and the
   provider selection flow.
5. `npm run typecheck`, `npm run lint`, `npm test -- --run`, `npm run build`,
   `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`
   and `cargo test` remain green.

## Out of scope

This redesign does not add browser-mode backend support, streaming protocol
changes, automatic model discovery, new provider adapters, authentication,
cloud sync, project memory extraction or a new product phase.
