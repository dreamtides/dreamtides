import type { FlexNode, GameAction } from "../types/battle";
import { extractOverlayContent } from "../util/flex-node-parser";

interface OverlayPromptProps {
  overlay: FlexNode;
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

export function OverlayPrompt({
  overlay,
  onAction,
  disabled,
}: OverlayPromptProps) {
  const content = extractOverlayContent(overlay);
  if (!content) return null;

  return (
    <div
      className="fixed inset-0 flex items-center justify-center z-50"
      style={{ background: "rgba(0, 0, 0, 0.7)" }}
    >
      <div
        className="rounded-lg p-6 max-w-lg w-full mx-4 flex flex-col gap-4"
        style={{
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
        }}
      >
        {content.texts.map((text, i) => (
          <p key={i} className="text-center">
            {text}
          </p>
        ))}
        {content.buttons.length > 0 && (
          <div className="flex gap-2 justify-center flex-wrap">
            {content.buttons.map((btn, i) => (
              <button
                key={i}
                onClick={() => {
                  if (!disabled) onAction(btn.action);
                }}
                disabled={disabled}
                className="px-4 py-2 rounded text-sm font-bold"
                style={{
                  background: "var(--color-primary)",
                  color: "var(--color-text)",
                  cursor: disabled ? "not-allowed" : "pointer",
                  opacity: disabled ? 0.5 : 1,
                  border: "1px solid var(--color-border)",
                }}
              >
                {btn.label}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
