import type { Project } from "../../types/project";

export function projectStatusLabel(project: Project): string {
  return project.status === "archived" ? "Archived" : "Active";
}

export function projectUpdatedLabel(project: Project): string {
  const date = new Date(project.updated_at);
  if (Number.isNaN(date.getTime())) return "Updated recently";
  return `Updated ${new Intl.DateTimeFormat(undefined, { dateStyle: "medium" }).format(date)}`;
}
