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
