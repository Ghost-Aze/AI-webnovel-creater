import { ChatWorkspace } from "./ChatWorkspace";

interface DeveloperChatPanelProps {
  projectId: string;
  disabled?: boolean;
}

const assistantInstructions: Record<string, string> = {
  "general-assistant":
    "You are the Developer Chat assistant. Respect the project's canon and keep all canonical changes explicit and reviewable.",
  "world-builder":
    "You are the world-building assistant. Respect the project's canon, identify assumptions clearly, and keep all canonical changes explicit and reviewable.",
  "continuity-reviewer":
    "You are the continuity reviewer. Check the project's canon carefully, call out contradictions, and keep all canonical changes explicit and reviewable.",
  "writing-coach":
    "You are the writing coach. Give concrete prose and scene guidance while respecting the project's canon and keeping all canonical changes explicit and reviewable.",
};

export function DeveloperChatPanel({
  projectId,
  disabled = false,
}: DeveloperChatPanelProps) {
  return (
    <ChatWorkspace
      projectId={projectId}
      kind="developer_chat"
      title="Developer Chat"
      scopeLabel="Project scoped"
      task="developer_chat"
      disabled={disabled}
      assistantInstructions={(assistantId) =>
        assistantInstructions[assistantId] ?? assistantInstructions["general-assistant"]
      }
      placeholder="Ask about this project…"
      messageLabel="Developer Chat message"
    />
  );
}
