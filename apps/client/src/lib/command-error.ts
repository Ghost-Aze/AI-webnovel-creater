export type CommandErrorCode =
  | "validation"
  | "not_found"
  | "archived_project"
  | "archived_memory"
  | "archived_character"
  | "archived_chapter"
  | "duplicate_name"
  | "duplicate_chapter_number"
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
  | "provider_unauthorized"
  | "provider_endpoint_not_found"
  | "provider_rate_limited"
  | "provider_unavailable"
  | "provider_timeout"
  | "provider_network_failure"
  | "secure_store_unavailable"
  | "storage"
  | "internal";

export class CommandError extends Error {
  readonly code: CommandErrorCode;
  readonly retryable: boolean;
  readonly action: CommandErrorAction;

  constructor(
    code: CommandErrorCode,
    message: string,
    options: { retryable?: boolean; action?: CommandErrorAction } = {},
  ) {
    super(message);
    this.name = "CommandError";
    this.code = code;
    this.retryable = options.retryable ?? false;
    this.action = options.action ?? null;
  }
}

export type CommandErrorAction = "retry" | "provider_settings" | null;

const messages: Record<Exclude<CommandErrorCode, "validation">, string> = {
  not_found: "The project could not be found.",
  archived_project: "Archived projects cannot be edited.",
  archived_memory: "Archived project memory cannot be edited.",
  archived_character: "Archived characters cannot be edited.",
  archived_chapter: "Archived chapters cannot be edited.",
  duplicate_name: "A character with this name already exists in the project.",
  duplicate_chapter_number:
    "A chapter with this number already exists in the project.",
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
  provider_unauthorized: "The provider credentials were rejected.",
  provider_endpoint_not_found:
    "The provider endpoint or selected model was not found.",
  provider_rate_limited: "The provider rate limit was reached. Try again soon.",
  provider_unavailable: "The provider is temporarily unavailable.",
  provider_timeout: "The provider request timed out.",
  provider_network_failure: "The provider could not be reached.",
  secure_store_unavailable:
    "Secure credential storage is not available on this build.",
  storage: "The project could not be saved. Try again.",
  internal: "Something went wrong. Try again.",
};

const recovery: Record<CommandErrorCode, {
  retryable: boolean;
  action: CommandErrorAction;
}> = {
  validation: { retryable: false, action: null },
  not_found: { retryable: false, action: null },
  archived_project: { retryable: false, action: null },
  archived_memory: { retryable: false, action: null },
  archived_character: { retryable: false, action: null },
  archived_chapter: { retryable: false, action: null },
  duplicate_name: { retryable: false, action: null },
  duplicate_chapter_number: { retryable: false, action: null },
  invalid_status: { retryable: false, action: null },
  conflict: { retryable: false, action: null },
  locked_canon: { retryable: false, action: null },
  invalid_proposal: { retryable: false, action: null },
  provider_not_found: { retryable: false, action: "provider_settings" },
  model_not_found: { retryable: false, action: "provider_settings" },
  no_suitable_model: { retryable: false, action: "provider_settings" },
  unsupported_capability: { retryable: false, action: null },
  invalid_provider_request: { retryable: false, action: null },
  provider_failure: { retryable: false, action: null },
  provider_unauthorized: { retryable: false, action: "provider_settings" },
  provider_endpoint_not_found: {
    retryable: false,
    action: "provider_settings",
  },
  provider_rate_limited: { retryable: true, action: "retry" },
  provider_unavailable: { retryable: true, action: "retry" },
  provider_timeout: { retryable: true, action: "retry" },
  provider_network_failure: { retryable: true, action: "retry" },
  secure_store_unavailable: { retryable: false, action: "provider_settings" },
  storage: { retryable: true, action: "retry" },
  internal: { retryable: false, action: null },
};

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return typeof value === "object" && value !== null
    ? (value as Record<string, unknown>)
    : undefined;
}

export function normalizeCommandError(value: unknown): CommandError {
  if (value instanceof CommandError) {
    if (value.code === "validation") {
      return new CommandError(value.code, value.message, recovery[value.code]);
    }
    return new CommandError(value.code, messages[value.code], recovery[value.code]);
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
    return new CommandError(code, message, recovery[code]);
  }
  return new CommandError(code, messages[code], recovery[code]);
}
