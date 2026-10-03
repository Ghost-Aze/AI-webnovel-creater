import type { ModelProfile, ModelTier, ProviderCapabilities } from "../../types/provider";

export interface ProviderPreset {
  id: string;
  displayName: string;
  baseUrl: string;
  credentialId: string;
  description: string;
  models: ModelProfile[];
}

const chatCapabilities: ProviderCapabilities = {
  streaming: true,
  embeddings: false,
  tools: false,
  vision: false,
  structured_output: false,
  prompt_caching: false,
};

function model(
  providerId: string,
  modelId: string,
  displayName: string,
  tier: ModelTier = "medium",
  contextWindowTokens = 131072,
  defaultOutputTokens = 16384,
): ModelProfile {
  return {
    provider_id: providerId,
    model_id: modelId,
    display_name: displayName,
    context_window_tokens: contextWindowTokens,
    default_output_tokens: defaultOutputTokens,
    strengths: [],
    weaknesses: [],
    strategy: [],
    tier,
    capabilities: chatCapabilities,
  };
}

export const providerPresets: ProviderPreset[] = [
  {
    id: "openai",
    displayName: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    credentialId: "openai-primary",
    description: "GPT and reasoning models for general writing workflows.",
    models: [
      model("openai", "gpt-5", "gpt-5", "large"),
      model("openai", "gpt-5-mini", "gpt-5-mini"),
      model("openai", "gpt-4.1", "gpt-4.1", "large"),
      model("openai", "gpt-4o", "gpt-4o"),
      model("openai", "gpt-4o-mini", "gpt-4o-mini", "small"),
      model("openai", "o3-mini", "o3-mini", "large"),
    ],
  },
  {
    id: "azure",
    displayName: "Azure",
    baseUrl: "https://YOUR-RESOURCE-NAME.openai.azure.com/openai/v1",
    credentialId: "azure-primary",
    description: "Azure OpenAI deployments with a resource endpoint preset.",
    models: [
      model("azure", "gpt-4o", "gpt-4o", "large"),
      model("azure", "gpt-4o-mini", "gpt-4o-mini", "small"),
    ],
  },
  {
    id: "anthropic",
    displayName: "Anthropic",
    baseUrl: "https://api.anthropic.com/v1",
    credentialId: "anthropic-primary",
    description: "Claude models for long-form reasoning and revision.",
    models: [
      model("anthropic", "claude-sonnet-4-5", "claude-sonnet-4-5", "large"),
      model("anthropic", "claude-haiku-4-5", "claude-haiku-4-5", "small"),
      model("anthropic", "claude-opus-4-1", "claude-opus-4-1", "large"),
    ],
  },
  {
    id: "openrouter",
    displayName: "OpenRouter",
    baseUrl: "https://openrouter.ai/api/v1",
    credentialId: "openrouter-primary",
    description: "One key for a broad catalog of routed models.",
    models: [
      model(
        "openrouter",
        "deepseek/deepseek-v4-flash-0731",
        "DeepSeek V4 Flash 0731",
        "medium",
        1310720,
        393216,
      ),
      model("openrouter", "openrouter/auto", "OpenRouter Auto", "large"),
      model("openrouter", "qwen/qwen3-30b-a3b:free", "Qwen3 30B A3B"),
      model("openrouter", "moonshotai/kimi-k3", "Kimi K3", "large"),
    ],
  },
  {
    id: "mistral",
    displayName: "Mistral",
    baseUrl: "https://api.mistral.ai/v1",
    credentialId: "mistral-primary",
    description: "Mistral and Codestral models for writing and coding.",
    models: [
      model("mistral", "mistral-large-latest", "Mistral Large", "large"),
      model("mistral", "mistral-small-latest", "Mistral Small", "small"),
      model("mistral", "codestral-latest", "Codestral"),
    ],
  },
  {
    id: "groq",
    displayName: "Groq",
    baseUrl: "https://api.groq.com/openai/v1",
    credentialId: "groq-primary",
    description: "Fast hosted open models for quick assistant turns.",
    models: [
      model("groq", "llama-3.3-70b-versatile", "Llama 3.3 70B", "large"),
      model("groq", "openai/gpt-oss-120b", "GPT OSS 120B", "large"),
      model("groq", "qwen/qwen3-32b", "Qwen3 32B"),
    ],
  },
  {
    id: "xai",
    displayName: "xAI",
    baseUrl: "https://api.x.ai/v1",
    credentialId: "xai-primary",
    description: "Grok models for fast, broad-context conversations.",
    models: [
      model("xai", "grok-4", "Grok 4", "large"),
      model("xai", "grok-3-mini", "Grok 3 Mini"),
    ],
  },
  {
    id: "gemini",
    displayName: "Gemini",
    baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai",
    credentialId: "gemini-primary",
    description: "Gemini models through the OpenAI-compatible endpoint.",
    models: [
      model("gemini", "gemini-2.5-pro", "Gemini 2.5 Pro", "large"),
      model("gemini", "gemini-2.5-flash", "Gemini 2.5 Flash"),
    ],
  },
  {
    id: "minimax",
    displayName: "MiniMax",
    baseUrl: "https://api.minimax.io/v1",
    credentialId: "minimax-primary",
    description: "MiniMax text models for story drafting and expansion.",
    models: [
      model("minimax", "MiniMax-M1", "MiniMax M1", "large"),
      model("minimax", "MiniMax-Text-01", "MiniMax Text 01"),
    ],
  },
  {
    id: "huggingface",
    displayName: "Hugging Face",
    baseUrl: "https://router.huggingface.co/v1",
    credentialId: "huggingface-primary",
    description: "Open models served through the Hugging Face router.",
    models: [
      model(
        "huggingface",
        "meta-llama/Llama-3.3-70B-Instruct",
        "Llama 3.3 70B Instruct",
        "large",
      ),
      model("huggingface", "Qwen/Qwen3-32B", "Qwen3 32B"),
    ],
  },
  {
    id: "nvidia-nim",
    displayName: "NVIDIA NIM",
    baseUrl: "https://integrate.api.nvidia.com/v1",
    credentialId: "nvidia-nim-primary",
    description: "NVIDIA hosted inference endpoints for open models.",
    models: [
      model(
        "nvidia-nim",
        "meta/llama-3.3-70b-instruct",
        "Llama 3.3 70B Instruct",
        "large",
      ),
      model(
        "nvidia-nim",
        "deepseek-ai/deepseek-v3.1",
        "DeepSeek V3.1",
        "large",
      ),
    ],
  },
];

export const providerPresetDescriptors = providerPresets.map((preset) => ({
  id: preset.id,
  display_name: preset.displayName,
}));

export function providerPresetFor(id: string) {
  return providerPresets.find((preset) => preset.id === id) ?? null;
}

export function mergeProviderModels(
  configuredModels: ModelProfile[],
  presetModels: ModelProfile[],
) {
  const keys = new Set(
    configuredModels.map((model) => `${model.provider_id}:${model.model_id}`),
  );
  return [
    ...configuredModels,
    ...presetModels.filter(
      (model) => !keys.has(`${model.provider_id}:${model.model_id}`),
    ),
  ];
}
