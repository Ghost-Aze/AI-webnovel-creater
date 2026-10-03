import { ChatComposer, type ChatComposerProps } from "./ChatComposer";

interface ChatEmptyStateProps {
  title: string;
  description: string;
  composerProps: ChatComposerProps;
}

export function ChatEmptyState({
  title,
  description,
  composerProps,
}: ChatEmptyStateProps) {
  return (
    <div className="chat-empty-state" data-testid="chat-empty-state">
      <div className="chat-empty-copy">
        <p className="eyebrow">AI workspace</p>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      <ChatComposer {...composerProps} />
    </div>
  );
}
