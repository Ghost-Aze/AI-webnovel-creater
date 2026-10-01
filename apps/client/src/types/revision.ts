import type { CanonStatus } from "./character";

export type MemoryEntityType = "character" | "character_state";
export type RevisionOperation =
  "create" | "update" | "archive" | "restore" | "canon_status" | "promote";
export type ActorType = "user" | "ai" | "system";
export type ProposalStatus = "draft" | "accepted" | "rejected";

export interface Revision {
  id: string;
  project_id: string;
  entity_type: MemoryEntityType;
  entity_id: string;
  revision: number;
  operation: RevisionOperation;
  actor_type: ActorType;
  actor_id: string | null;
  base_revision: number;
  previous_value: Record<string, unknown> | null;
  new_value: Record<string, unknown>;
  source_type: string | null;
  source_id: string | null;
  created_at: string;
}

export interface Proposal {
  id: string;
  project_id: string;
  entity_type: MemoryEntityType;
  entity_id: string;
  operation: RevisionOperation;
  payload: Record<string, unknown>;
  base_revision: number;
  status: ProposalStatus;
  actor_type: ActorType;
  actor_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateProposalInput {
  project_id: string;
  entity_type: MemoryEntityType;
  entity_id?: string | null;
  operation: RevisionOperation;
  payload: Record<string, unknown>;
  base_revision: number;
  actor_type: ActorType;
  actor_id?: string | null;
}

export type { CanonStatus };
