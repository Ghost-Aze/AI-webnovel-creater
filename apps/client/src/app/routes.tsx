import { Navigate, Route, Routes } from "react-router-dom";

import { AppShell } from "../components/layout/AppShell";
import { ProjectWorkspacePage } from "../features/projects/ProjectWorkspacePage";
import { ProjectsPage } from "../features/projects/ProjectsPage";
import { ProviderSettingsPage } from "../features/providers/ProviderSettingsPage";

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route path="/projects" element={<ProjectsPage />} />
        <Route path="/projects/:projectId" element={<ProjectWorkspacePage />} />
        <Route path="/settings/providers" element={<ProviderSettingsPage />} />
        <Route path="*" element={<Navigate to="/projects" replace />} />
      </Route>
    </Routes>
  );
}
