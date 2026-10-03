import type { CanonStatus } from "./character";

export type ProjectMemoryStatus = "active" | "archived";

export interface ProjectMemoryListFilter {
  include_archived: boolean;
}

export interface StoryFact {
  id: string;
  project_id: string;
  title: string;
  content: string;
  status: ProjectMemoryStatus;
  canon_status: CanonStatus;
  revision: number;
  created_at: string;
  updated_at: string;
}

export interface CanonRule {
  id: string;
  project_id: string;
  title: string;
  rule: string;
  scope: string;
  status: ProjectMemoryStatus;
  canon_status: CanonStatus;
  revision: number;
  created_at: string;
  updated_at: string;
}

export interface CreateStoryFactInput {
  title: string;
  content: string;
}

export type UpdateStoryFactInput = CreateStoryFactInput;

export interface CreateCanonRuleInput {
  title: string;
  rule: string;
  scope: string;
}

export type UpdateCanonRuleInput = CreateCanonRuleInput;
