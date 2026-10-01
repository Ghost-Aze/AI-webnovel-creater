export type ProjectStatus = "active" | "archived";

export interface Project {
  id: string;
  name: string;
  description: string;
  status: ProjectStatus;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectInput {
  name: string;
  description?: string | null;
}

export interface UpdateProjectInput {
  name: string;
  description?: string | null;
}

export interface ProjectListFilter {
  include_archived: boolean;
}
