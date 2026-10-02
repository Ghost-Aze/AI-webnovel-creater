export interface ProviderDescriptor {
  id: string;
  display_name: string;
}

export interface ProviderCapabilities {
  streaming: boolean;
  embeddings: boolean;
  tools: boolean;
  vision: boolean;
  structured_output: boolean;
  prompt_caching: boolean;
}

export type ModelTier = "local" | "small" | "medium" | "large";

export interface ModelProfile {
  provider_id: string;
  model_id: string;
  display_name: string;
  context_window_tokens: number;
  default_output_tokens: number;
  strengths: string[];
  weaknesses: string[];
  strategy: string[];
  tier: ModelTier;
  capabilities: ProviderCapabilities;
}

export interface ModelRef {
  provider_id: string;
  model_id: string;
}

export type QualityMode = "fast" | "balanced" | "deep";

export type ModelTask =
  | "story_architecture"
  | "arc_planning"
  | "chapter_planning"
  | "scene_planning"
  | "main_writing"
  | "character_psychology"
  | "major_revision"
  | "plot_analysis"
  | "style_check"
  | "memory_extraction"
  | "summary"
  | "entity_extraction"
  | "timeline_extraction"
  | "metadata"
  | "retrieval_query"
  | "continuity_check";

export type RouteSelectionReason = "preferred" | "policy";

export interface RoutingRequest {
  task: ModelTask;
  quality: QualityMode;
  preferred_model: ModelRef | null;
  required_capabilities: ProviderCapabilities;
  minimum_context_window_tokens: number | null;
}

export interface RouteDecision {
  model: ModelRef;
  profile: ModelProfile;
  quality: QualityMode;
  reason: RouteSelectionReason;
}

export type PromptRole = "system" | "user" | "assistant";

export interface PromptMessage {
  role: PromptRole;
  content: string;
}

export interface GenerateRequest {
  model: ModelRef;
  messages: PromptMessage[];
  max_output_tokens: number;
  temperature: number | null;
}

export interface ProviderUsage {
  input_tokens: number | null;
  output_tokens: number | null;
}

export interface GenerateResponse {
  model: ModelRef;
  text: string;
  usage: ProviderUsage;
}

export interface ProviderConfigureInput {
  descriptor: ProviderDescriptor;
  base_url: string;
  models: ModelProfile[];
  credential_id: string;
  credential_value: string;
}

export interface ProviderConfigureResult {
  descriptor: ProviderDescriptor;
  models: ModelProfile[];
}

export type CredentialStoreKind = "ephemeral" | "platform_secure";

export interface CredentialStoreStatus {
  kind: CredentialStoreKind;
  persistent: boolean;
  available: boolean;
}
