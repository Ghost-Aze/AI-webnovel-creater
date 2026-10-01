import { describe, expect, it } from "vitest";

import { normalizeCommandError } from "./command-error";

describe("provider command errors", () => {
  it.each([
    ["provider_not_found", "The requested provider is not available."],
    ["model_not_found", "The requested model is not available."],
    [
      "unsupported_capability",
      "The provider does not support this capability.",
    ],
    ["invalid_provider_request", "The provider request is invalid."],
    ["provider_failure", "The provider operation failed."],
  ])("maps %s without backend details", (code, message) => {
    const error = normalizeCommandError({
      code: code.toUpperCase(),
      details: { raw: "secret prompt" },
    });
    expect(error).toMatchObject({ code, message });
    expect(error.message).not.toContain("secret prompt");
  });
});
