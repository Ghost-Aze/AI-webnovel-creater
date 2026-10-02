import { describe, expect, it } from "vitest";

import {
  manuscriptToEditorHtml,
  plainTextToHtml,
  sanitizeHtml,
} from "../features/manuscripts/rich-text-utils";

describe("rich text utilities", () => {
  it("converts legacy plain text into safe paragraphs", () => {
    expect(plainTextToHtml("First line\nsecond line\n\nNext paragraph")).toBe(
      "<p>First line<br>second line</p><p>Next paragraph</p>",
    );
    expect(manuscriptToEditorHtml("<p>Legacy</p>", "plain_text")).toContain(
      "&lt;p&gt;Legacy&lt;/p&gt;",
    );
  });

  it("keeps formatting tags and removes unsafe tags and attributes", () => {
    expect(
      sanitizeHtml(
        '<p onclick="alert(1)"><strong>Safe</strong><script>bad()</script></p>',
      ),
    ).toBe("<p><strong>Safe</strong></p>");
  });
});
