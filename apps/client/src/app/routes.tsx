import { Navigate, Route, Routes } from "react-router-dom";

import { AppShell } from "../components/layout/AppShell";
import { ProjectWorkspacePage } from "../features/projects/ProjectWorkspacePage";
import { ProjectsPage } from "../features/projects/ProjectsPage";
import { ProviderSettingsPage } from "../features/providers/ProviderSettingsPage";
import { ChapterEditorPage } from "../features/manuscripts/ChapterEditorPage";
import { ManuscriptsPage } from "../features/manuscripts/ManuscriptsPage";
import { DeveloperChatPage } from "../features/chat/DeveloperChatPage";

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route path="/projects" element={<ProjectsPage />} />
        <Route path="/chat" element={<DeveloperChatPage />} />
        <Route path="/manuscripts" element={<ManuscriptsPage />} />
        <Route
          path="/projects/:projectId/chat"
          element={<DeveloperChatPage />}
        />
        <Route
          path="/projects/:projectId/manuscripts"
          element={<ManuscriptsPage />}
        />
        <Route path="/projects/:projectId" element={<ProjectWorkspacePage />} />
        <Route
          path="/projects/:projectId/chapters/:chapterId"
          element={<ChapterEditorPage />}
        />
        <Route path="/settings/providers" element={<ProviderSettingsPage />} />
        <Route path="*" element={<Navigate to="/projects" replace />} />
      </Route>
    </Routes>
  );
}
