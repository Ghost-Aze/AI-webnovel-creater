import type {
  CompiledContext,
  ContextBudget,
  WorkingMemoryBlock,
} from "./context";
import type {
  GenerateResponse,
  ModelRef,
  ModelTask,
  ProviderCapabilities,
  QualityMode,
  RouteDecision,
} from "./provider";

export type LogicalAgent =
  | "planner"
  | "writer"
  | "plot_analyst"
  | "continuity_critic"
  | "style_critic"
  | "revision"
  | "memory_curator";

export interface OrchestrationStep {
  id: string;
  agent: LogicalAgent;
  task: ModelTask;
  quality: QualityMode;
  depends_on: string[];
}

export interface OrchestrationPlan {
  task: ModelTask;
  quality: QualityMode;
  steps: OrchestrationStep[];
}

export interface OrchestrationRequest {
  project_id: string;
  task: ModelTask;
  quality: QualityMode;
  preferred_model: ModelRef | null;
  required_capabilities: ProviderCapabilities;
  minimum_context_window_tokens: number | null;
  system_instructions: string;
  character_ids: string[];
  include_character_states: boolean;
  working_memory: WorkingMemoryBlock[];
  context_budget: ContextBudget;
  user_prompt: string;
  temperature: number | null;
}

export interface OrchestrationStepResult {
  step: OrchestrationStep;
  route: RouteDecision;
  context: CompiledContext;
  response: GenerateResponse;
}

export interface OrchestrationResult {
  plan: OrchestrationPlan;
  steps: OrchestrationStepResult[];
}
