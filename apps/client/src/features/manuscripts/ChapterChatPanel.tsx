import { useState } from "react";

import { normalizeCommandError, type CommandError } from "../../lib/command-error";
import { createManuscriptProposal } from "../../lib/commands";
import type { ConversationMessage } from "../../types/conversation";
import { ChatWorkspace } from "../chat/ChatWorkspace";

interface ChapterChatPanelProps {
  projectId: string;
  chapterId: string;
  currentRevision: number;
  disabled?: boolean;
  onProposalCreated?: () => void;
}

const assistantInstructions: Record<string, string> = {
  "general-assistant":
    "You are the Chapter Chat assistant. Discuss and propose manuscript changes without changing canonical data automatically.",
  "world-builder":
    "You are the Chapter Chat world-building assistant. Discuss manuscript changes without changing canonical data automatically.",
  "continuity-reviewer":
    "You are the Chapter Chat continuity reviewer. Check the current chapter against the project's canon, then discuss and propose manuscript changes without changing canonical data automatically.",
  "writing-coach":
    "You are the Chapter Chat writing coach. Give concrete guidance on tension, pacing and scene craft without changing canonical data automatically.",
};

export function ChapterChatPanel({
  projectId,
  chapterId,
  currentRevision,
  disabled = false,
  onProposalCreated,
}: ChapterChatPanelProps) {
  const [proposalId, setProposalId] = useState<string | null>(null);
  const [proposalError, setProposalError] = useState<CommandError | null>(null);

  async function handlePropose(message: ConversationMessage) {
    if (disabled) return;
    setProposalError(null);
    try {
      const proposal = await createManuscriptProposal(projectId, {
        chapter_id: chapterId,
        base_revision: currentRevision,
        proposed_content: message.content,
        content_format: "plain_text",
        rationale: "Proposed from Chapter Chat",
        actor_type: "ai",
        actor_id: "chapter-chat",
      });
      setProposalId(proposal.id);
      onProposalCreated?.();
    } catch (error) {
      setProposalError(normalizeCommandError(error));
    }
  }

  return (
    <>
      <ChatWorkspace
        projectId={projectId}
        chapterId={chapterId}
        kind="chapter_chat"
        title="Chapter Chat"
        scopeLabel="Chapter scoped"
        task="developer_chat"
        disabled={disabled}
        assistantInstructions={(assistantId) =>
          assistantInstructions[assistantId] ?? assistantInstructions["general-assistant"]
        }
        placeholder="Ask about this chapter…"
        messageLabel="Chapter Chat message"
        onAssistantAction={(message) => void handlePropose(message)}
        assistantActionLabel="Propose revision"
      />
      {proposalId && <span className="saved-message">Proposal saved</span>}
      {proposalError && (
        <div className="error-state" role="alert">
          {proposalError.message}
        </div>
      )}
    </>
  );
}
