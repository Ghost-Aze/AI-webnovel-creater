export type ChapterStatus = "draft" | "final" | "archived";
export type ManuscriptActorType = "user" | "ai" | "system";
export type ManuscriptContentFormat = "plain_text" | "html";

export interface Chapter {
  id: string;
  project_id: string;
  number: number;
  title: string;
  synopsis: string;
  status: ChapterStatus;
  revision: number;
  created_at: string;
  updated_at: string;
}

export interface CreateChapterInput {
  number: number;
  title: string;
  synopsis?: string | null;
}

export interface UpdateChapterInput {
  number: number;
  title: string;
  synopsis?: string | null;
  status?: ChapterStatus | null;
}

export interface ChapterListFilter {
  include_archived: boolean;
}

export interface Manuscript {
  id: string;
  chapter_id: string;
  content: string;
  content_format: ManuscriptContentFormat;
  revision: number;
  created_at: string;
  updated_at: string;
}

export interface ManuscriptRevision {
  id: string;
  manuscript_id: string;
  revision: number;
  content: string;
  content_format: ManuscriptContentFormat;
  label: string;
  actor_type: ManuscriptActorType;
  actor_id: string | null;
  created_at: string;
}

export interface SaveManuscriptInput {
  content: string;
  content_format?: ManuscriptContentFormat | null;
  label?: string | null;
  actor_type?: ManuscriptActorType | null;
  actor_id?: string | null;
  expected_revision: number;
}
