export type CommandErrorCode =
  | "validation"
  | "not_found"
  | "archived_project"
  | "invalid_status"
  | "storage"
  | "internal";

export class CommandError extends Error {
  readonly code: CommandErrorCode;

  constructor(code: CommandErrorCode, message: string) {
    super(message);
    this.name = "CommandError";
    this.code = code;
  }
}

const messages: Record<Exclude<CommandErrorCode, "validation">, string> = {
  not_found: "The project could not be found.",
  archived_project: "Archived projects cannot be edited.",
  invalid_status: "The project has an invalid status.",
  storage: "The project could not be saved. Try again.",
  internal: "Something went wrong. Try again.",
};

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return typeof value === "object" && value !== null
    ? (value as Record<string, unknown>)
    : undefined;
}

export function normalizeCommandError(value: unknown): CommandError {
  if (value instanceof CommandError) {
    if (value.code === "validation") {
      return new CommandError(value.code, value.message);
    }
    return new CommandError(value.code, messages[value.code]);
  }

  const record = asRecord(value);
  let payload: Record<string, unknown> | undefined = record;

  if (typeof value === "string") {
    try {
      payload = asRecord(JSON.parse(value));
    } catch {
      payload = undefined;
    }
  }

  const rawCode =
    typeof payload?.code === "string" ? payload.code.toLowerCase() : "internal";
  const code = (
    rawCode in messages || rawCode === "validation" ? rawCode : "internal"
  ) as CommandErrorCode;
  if (code === "validation") {
    const details = asRecord(payload?.details);
    const message =
      typeof details?.message === "string"
        ? details.message
        : "Please check the project details.";
    return new CommandError(code, message);
  }
  return new CommandError(code, messages[code]);
}
