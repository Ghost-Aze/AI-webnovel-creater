import type { Revision } from "../../types/revision";

interface RevisionDiffProps {
  revision: Revision;
}

const ignoredFields = new Set([
  "id",
  "project_id",
  "revision",
  "updated_at",
  "created_at",
]);

function labelForField(field: string): string {
  return field
    .replaceAll("_", " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function displayValue(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "string") return value;
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

export function RevisionDiff({ revision }: RevisionDiffProps) {
  const previous = revision.previous_value ?? {};
  const fields = Array.from(
    new Set([...Object.keys(previous), ...Object.keys(revision.new_value)]),
  ).filter((field) => {
    if (ignoredFields.has(field)) return false;
    return (
      JSON.stringify(previous[field]) !==
      JSON.stringify(revision.new_value[field])
    );
  });

  if (fields.length === 0) {
    return <p className="revision-diff-empty">No field changes recorded.</p>;
  }

  return (
    <div className="revision-diff" data-testid="revision-diff">
      {fields.map((field) => (
        <div className="revision-diff-row" key={field}>
          <strong>{labelForField(field)}</strong>
          <del>{displayValue(previous[field])}</del>
          <ins>{displayValue(revision.new_value[field])}</ins>
        </div>
      ))}
    </div>
  );
}
