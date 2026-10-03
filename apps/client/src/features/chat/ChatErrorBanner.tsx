import type { CommandError } from "../../lib/command-error";

interface ChatErrorBannerProps {
  error: CommandError;
  onRetry?: () => void;
  onOpenProviderSettings?: () => void;
}

export function ChatErrorBanner({
  error,
  onRetry,
  onOpenProviderSettings,
}: ChatErrorBannerProps) {
  return (
    <div className="chat-error-banner" role="alert">
      <div>
        <strong>Generation failed</strong>
        <span>{error.message}</span>
      </div>
      {error.action === "retry" && error.retryable && onRetry && (
        <button className="button button-ghost" type="button" onClick={onRetry}>
          Retry
        </button>
      )}
      {error.action === "provider_settings" && (
        <a
          className="button button-ghost"
          href="/settings/providers"
          onClick={onOpenProviderSettings}
        >
          Open Provider settings
        </a>
      )}
    </div>
  );
}
