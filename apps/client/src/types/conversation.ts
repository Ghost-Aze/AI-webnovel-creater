import type { ContextBudget } from "./context";
import type {
  ModelRef,
  ModelTask,
  ProviderCapabilities,
  QualityMode,
} from "./provider";
import type { OrchestrationResult } from "./orchestration";

export type ConversationKind = "developer_chat" | "arc_chat" | "chapter_chat";

export type MessageRole = "system" | "user" | "assistant" | "tool";

export interface Conversation {
  id: string;
  project_id: string;
  chapter_id: string | null;
  kind: ConversationKind;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface ConversationMessage {
  id: string;
  conversation_id: string;
  sequence: number;
  role: MessageRole;
  content: string;
  model: ModelRef | null;
  created_at: string;
}

export interface CreateConversationInput {
  project_id: string;
  chapter_id?: string | null;
  kind: ConversationKind;
  title: string;
}

export interface ConversationListFilter {
  kind: ConversationKind | null;
}

export interface AppendMessageInput {
  role: MessageRole;
  content: string;
  model: ModelRef | null;
}

export interface DeveloperChatSendRequest {
  conversation_id: string;
  task: ModelTask;
  quality: QualityMode;
  preferred_model: ModelRef | null;
  required_capabilities: ProviderCapabilities;
  minimum_context_window_tokens: number | null;
  system_instructions: string;
  character_ids: string[];
  include_character_states: boolean;
  context_budget: ContextBudget;
  message: string;
  temperature: number | null;
  retry_attempt: boolean;
}

export interface DeveloperChatSendResult {
  conversation: Conversation;
  user_message: ConversationMessage;
  assistant_message: ConversationMessage;
  orchestration: OrchestrationResult;
}

export interface MemoryToolUpdateCharacterInput {
  name: string;
  summary: string | null;
  role: string | null;
}

export interface MemoryToolRequest {
  project_id: string;
  character_id: string;
  base_revision: number;
  actor_id: string | null;
  action: {
    action: "update_character";
    input: MemoryToolUpdateCharacterInput;
  };
}
