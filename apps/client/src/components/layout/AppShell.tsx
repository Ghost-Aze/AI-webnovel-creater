import { Outlet } from "react-router-dom";

import { ContextPanel } from "./ContextPanel";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";

export function AppShell() {
  return (
    <div className="app-shell">
      <Sidebar />
      <main className="app-main">
        <Outlet />
      </main>
      <ContextPanel />
      <StatusBar />
    </div>
  );
}
