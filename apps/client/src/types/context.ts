import type { ModelRef } from "./provider";

export type ContextTask =
  | "developer_chat"
  | "arc_planning"
  | "chapter_planning"
  | "scene_planning"
  | "writing"
  | "continuity_check"
  | "memory_extraction"
  | { custom: string };

export type ContextBlockKind =
  "system" | "project" | "character" | "character_state" | "working_memory";

export interface WorkingMemoryBlock {
  id: string;
  label: string;
  content: string;
}

export interface ContextBudget {
  output_reserve_tokens: number | null;
  safety_margin_tokens: number;
}

export interface ContextCompileRequest {
  project_id: string;
  task: ContextTask;
  model: ModelRef;
  system_instructions: string;
  character_ids: string[];
  include_character_states: boolean;
  working_memory: WorkingMemoryBlock[];
  budget: ContextBudget;
}

export type OmissionReason = "budget_exceeded" | "truncated";

export interface ContextBlock {
  kind: ContextBlockKind;
  source_id: string | null;
  content: string;
  estimated_tokens: number;
  priority: number;
  truncated: boolean;
}

export interface ContextOmission {
  source_id: string | null;
  kind: ContextBlockKind;
  reason: OmissionReason;
}

export interface CompiledContext {
  project_id: string;
  task: ContextTask;
  model: ModelRef;
  blocks: ContextBlock[];
  omissions: ContextOmission[];
  input_budget_tokens: number;
  estimated_input_tokens: number;
}
