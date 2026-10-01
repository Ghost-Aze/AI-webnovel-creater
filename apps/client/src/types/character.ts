export type CharacterStatus = "active" | "archived";

export interface Character {
  id: string;
  project_id: string;
  name: string;
  summary: string;
  role: string;
  status: CharacterStatus;
  created_at: string;
  updated_at: string;
}

export interface CreateCharacterInput {
  name: string;
  summary?: string | null;
  role?: string | null;
}

export interface UpdateCharacterInput {
  name: string;
  summary?: string | null;
  role?: string | null;
}

export interface CharacterListFilter {
  include_archived: boolean;
}

export interface CharacterState {
  character_id: string;
  current_location: string;
  physical_condition: string;
  injuries: string;
  emotional_state: string;
  goals: string;
  beliefs: string;
  knowledge: string;
  secrets_known: string;
  current_conflicts: string;
  possessions: string;
  promises: string;
  last_appearance: string;
  current_arc_role: string;
  updated_at: string;
}

export type UpdateCharacterStateInput = Partial<
  Omit<CharacterState, "character_id" | "updated_at">
>;
