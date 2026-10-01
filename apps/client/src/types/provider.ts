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

export interface ModelProfile {
  provider_id: string;
  model_id: string;
  display_name: string;
  context_window_tokens: number;
  default_output_tokens: number;
  strengths: string[];
  weaknesses: string[];
  strategy: string[];
  capabilities: ProviderCapabilities;
}

export interface ModelRef {
  provider_id: string;
  model_id: string;
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
