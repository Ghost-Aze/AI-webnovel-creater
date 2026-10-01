export type CommandErrorCode =
  | "validation"
  | "not_found"
  | "archived_project"
  | "archived_character"
  | "duplicate_name"
  | "invalid_status"
  | "conflict"
  | "locked_canon"
  | "invalid_proposal"
  | "provider_not_found"
  | "model_not_found"
  | "no_suitable_model"
  | "unsupported_capability"
  | "invalid_provider_request"
  | "provider_failure"
  | "secure_store_unavailable"
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
  archived_character: "Archived characters cannot be edited.",
  duplicate_name: "A character with this name already exists in the project.",
  invalid_status: "The project has an invalid status.",
  conflict: "This record changed. Reload and try again.",
  locked_canon: "Locked canon cannot be changed.",
  invalid_proposal: "This proposal is no longer available.",
  provider_not_found: "The requested provider is not available.",
  model_not_found: "The requested model is not available.",
  no_suitable_model: "No suitable model is available for this task.",
  unsupported_capability: "The provider does not support this capability.",
  invalid_provider_request: "The provider request is invalid.",
  provider_failure: "The provider operation failed.",
  secure_store_unavailable:
    "Secure credential storage is not available on this build.",
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
