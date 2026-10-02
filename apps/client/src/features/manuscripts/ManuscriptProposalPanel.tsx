import { useCallback, useEffect, useState } from "react";

import { normalizeCommandError } from "../../lib/command-error";
import {
  listManuscriptProposals,
  promoteManuscriptProposal,
  rejectManuscriptProposal,
} from "../../lib/commands";
import type { Manuscript, ManuscriptProposal } from "../../types/manuscript";

interface ManuscriptProposalPanelProps {
  chapterId: string;
  currentRevision: number;
  disabled?: boolean;
  refreshKey?: number;
  onPromoted: (manuscript: Manuscript) => void;
}

export function ManuscriptProposalPanel({
  chapterId,
  currentRevision,
  disabled = false,
  refreshKey = 0,
  onPromoted,
}: ManuscriptProposalPanelProps) {
  const [proposals, setProposals] = useState<ManuscriptProposal[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadProposals = useCallback(async () => {
    setIsLoading(true);
    setLoadError(null);
    setError(null);
    try {
      setProposals(await listManuscriptProposals(chapterId, "draft"));
    } catch (commandError) {
      setLoadError(normalizeCommandError(commandError).message);
    } finally {
      setIsLoading(false);
    }
  }, [chapterId]);

  useEffect(() => {
    void loadProposals();
  }, [loadProposals, refreshKey]);

  async function handlePromote(proposal: ManuscriptProposal) {
    setBusyId(proposal.id);
    setError(null);
    try {
      const updated = await promoteManuscriptProposal(
        proposal.id,
        currentRevision,
      );
      setProposals((current) =>
        current.filter((item) => item.id !== proposal.id),
      );
      onPromoted(updated);
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setBusyId(null);
    }
  }

  async function handleReject(proposal: ManuscriptProposal) {
    setBusyId(proposal.id);
    setError(null);
    try {
      await rejectManuscriptProposal(proposal.id);
      setProposals((current) =>
        current.filter((item) => item.id !== proposal.id),
      );
    } catch (commandError) {
      setError(normalizeCommandError(commandError).message);
    } finally {
      setBusyId(null);
    }
  }

  return (
    <section
      className="workspace-primary manuscript-proposal-panel"
      aria-labelledby="proposal-heading"
    >
      <div className="section-heading">
        <div>
          <p className="eyebrow">Review gate</p>
          <h2 id="proposal-heading">Manuscript proposals</h2>
        </div>
      </div>
      {isLoading && <p className="loading-state">Loading proposals…</p>}
      {!isLoading && loadError && (
        <div className="error-state" role="alert">
          <strong>Proposals could not be loaded.</strong>
          <span>{loadError}</span>
          <button
            className="button button-ghost"
            type="button"
            onClick={() => void loadProposals()}
          >
            Try again
          </button>
        </div>
      )}
      {!isLoading && !loadError && proposals.length === 0 && (
        <p className="empty-state">No pending proposals.</p>
      )}
      {!isLoading && !loadError && (
        <div className="manuscript-proposal-list">
          {proposals.map((proposal) => (
            <article className="manuscript-proposal" key={proposal.id}>
              <p>{proposal.rationale || "Chapter Chat proposal"}</p>
              <pre>{proposal.proposed_content}</pre>
              <small>Based on revision {proposal.base_revision}</small>
              <div className="editor-conflict-actions">
                <button
                  className="button button-ghost button-small"
                  type="button"
                  onClick={() => void handleReject(proposal)}
                  disabled={disabled || busyId === proposal.id}
                >
                  Reject
                </button>
                <button
                  className="button button-primary button-small"
                  type="button"
                  onClick={() => void handlePromote(proposal)}
                  disabled={disabled || busyId === proposal.id}
                >
                  {busyId === proposal.id ? "Applying…" : "Apply revision"}
                </button>
              </div>
            </article>
          ))}
        </div>
      )}
      {error && !loadError && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
