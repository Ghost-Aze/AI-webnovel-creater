export function StatusBar() {
  return (
    <footer className="status-bar" aria-label="Application status">
      <span>
        <i className="status-dot" aria-hidden="true" />
        Local mode
      </span>
      <span>Balanced · Ready for your story</span>
      <span className="status-bar-right">Phase 0</span>
    </footer>
  );
}
