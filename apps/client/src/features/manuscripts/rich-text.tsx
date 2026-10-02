import { useEffect, useRef } from "react";

import { sanitizeHtml } from "./rich-text-utils";

interface RichTextEditorProps {
  value: string;
  disabled?: boolean;
  onChange: (value: string) => void;
}

const toolbar = [
  { label: "B", name: "Bold", command: "bold", value: undefined },
  { label: "I", name: "Italic", command: "italic", value: undefined },
  { label: "H2", name: "Heading 2", command: "formatBlock", value: "<h2>" },
  {
    label: "•",
    name: "Bulleted list",
    command: "insertUnorderedList",
    value: undefined,
  },
  {
    label: "1.",
    name: "Numbered list",
    command: "insertOrderedList",
    value: undefined,
  },
  { label: "❝", name: "Quote", command: "formatBlock", value: "<blockquote>" },
  { label: "↶", name: "Undo", command: "undo", value: undefined },
  { label: "↷", name: "Redo", command: "redo", value: undefined },
] as const;

export function RichTextEditor({
  value,
  disabled,
  onChange,
}: RichTextEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const renderedValue = useRef("");
  const initialValue = useRef(value);

  useEffect(() => {
    const editor = editorRef.current;
    if (!editor || renderedValue.current === value) return;
    if (document.activeElement !== editor) editor.innerHTML = value;
    renderedValue.current = value;
  }, [value]);

  function emitChange() {
    const editor = editorRef.current;
    if (!editor) return;
    const sanitized = sanitizeHtml(editor.innerHTML);
    renderedValue.current = sanitized;
    onChange(sanitized);
  }

  function runCommand(command: string, argument?: string) {
    editorRef.current?.focus();
    document.execCommand(command, false, argument);
    emitChange();
  }

  return (
    <div className="rich-editor" data-testid="rich-editor">
      <div
        className="rich-editor-toolbar"
        role="toolbar"
        aria-label="Formatting"
      >
        {toolbar.map((item) => (
          <button
            className="rich-editor-tool"
            key={item.name}
            type="button"
            aria-label={item.name}
            title={item.name}
            disabled={disabled}
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => runCommand(item.command, item.value)}
          >
            {item.label}
          </button>
        ))}
      </div>
      <div
        ref={editorRef}
        className="manuscript-editor rich-editor-surface"
        contentEditable={!disabled}
        role="textbox"
        aria-label="Manuscript"
        aria-multiline="true"
        data-placeholder="Begin the chapter…"
        onInput={emitChange}
        dangerouslySetInnerHTML={{ __html: initialValue.current }}
        suppressContentEditableWarning
      />
    </div>
  );
}
