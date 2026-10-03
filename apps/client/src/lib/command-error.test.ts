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
    [
      "secure_store_unavailable",
      "Secure credential storage is not available on this build.",
    ],
  ])("maps %s without backend details", (code, message) => {
    const error = normalizeCommandError({
      code: code.toUpperCase(),
      details: { raw: "secret prompt" },
    });
    expect(error).toMatchObject({ code, message });
    expect(error.message).not.toContain("secret prompt");
  });

  it.each([
    ["provider_unauthorized", "provider_settings", false],
    ["provider_endpoint_not_found", "provider_settings", false],
    ["secure_store_unavailable", "provider_settings", false],
    ["provider_rate_limited", "retry", true],
    ["provider_unavailable", "retry", true],
    ["provider_timeout", "retry", true],
    ["provider_network_failure", "retry", true],
  ] as const)(
    "exposes safe recovery metadata for %s",
    (code, action, retryable) => {
      const error = normalizeCommandError({
        code,
        details: { raw: "Bearer secret prompt" },
      });
      expect(error).toMatchObject({ code, action, retryable });
      expect(error.message).not.toContain("Bearer");
      expect(error.message).not.toContain("secret prompt");
    },
  );

  it("keeps unknown provider details non-retryable and generic", () => {
    const error = normalizeCommandError({
      code: "provider_unknown",
      details: { body: "secret provider response" },
    });

    expect(error).toMatchObject({
      code: "internal",
      action: null,
      retryable: false,
    });
    expect(error.message).not.toContain("secret provider response");
  });
});
