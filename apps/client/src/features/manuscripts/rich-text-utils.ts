const allowedTags = new Set([
  "P",
  "BR",
  "STRONG",
  "B",
  "EM",
  "I",
  "H1",
  "H2",
  "H3",
  "UL",
  "OL",
  "LI",
  "BLOCKQUOTE",
]);

export function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function plainTextToHtml(value: string): string {
  return value
    .split(/\n{2,}/)
    .map(
      (paragraph) => `<p>${escapeHtml(paragraph).replaceAll("\n", "<br>")}</p>`,
    )
    .join("");
}

export function sanitizeHtml(value: string): string {
  if (typeof DOMParser === "undefined") return value;
  const document = new DOMParser().parseFromString(value, "text/html");
  document
    .querySelectorAll("script, style, iframe, object, embed, form")
    .forEach((node) => node.remove());
  document.body.querySelectorAll("*").forEach((element) => {
    if (!allowedTags.has(element.tagName)) {
      element.replaceWith(...Array.from(element.childNodes));
      return;
    }
    Array.from(element.attributes).forEach((attribute) =>
      element.removeAttribute(attribute.name),
    );
  });
  return document.body.innerHTML;
}

export function manuscriptToEditorHtml(
  content: string,
  format: "plain_text" | "html",
): string {
  return format === "html" ? sanitizeHtml(content) : plainTextToHtml(content);
}
