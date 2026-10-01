const placeholders = [
  "Context",
  "Characters",
  "Timeline",
  "Plot Threads",
  "Memory",
  "Continuity",
  "Project Bible",
];

export function ContextPanel() {
  return (
    <aside className="context-panel" aria-label="Project context">
      <div className="panel-heading">
        <p className="eyebrow">Workspace</p>
        <h2>Context</h2>
      </div>
      <div className="context-list">
        {placeholders.map((item, index) => (
          <div className="context-placeholder" key={item}>
            <span className="context-icon" aria-hidden="true">
              {["◌", "♙", "◷", "⌁", "▤", "✓", "✧"][index]}
            </span>
            <span>{item}</span>
            <span className="placeholder-badge">Soon</span>
          </div>
        ))}
      </div>
    </aside>
  );
}
