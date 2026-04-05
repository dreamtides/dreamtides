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
  if (content.texts.length === 0 && content.buttons.length === 0) return null;

  // If the overlay has interactive buttons (not just display-only toggle
  // buttons), render as a blocking modal. Otherwise render as a non-blocking
  // banner at the top of the screen.
  const hasInteractiveButtons = content.buttons.some(
    (btn) => {
      const action = btn.action as Record<string, unknown> | null;
      if (!action) return false;
      // BattleDisplayAction buttons (like ToggleStackVisibility) are
      // non-interactive UI toggles, not game prompts
      return !("BattleDisplayAction" in action);
    }
  );

  if (hasInteractiveButtons) {
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

  // Non-blocking banner at the top
  return (
    <div
      className="fixed top-0 left-0 right-0 flex justify-center z-30 pointer-events-none"
      style={{ paddingTop: 40 }}
    >
      <div
        className="rounded-lg px-4 py-2 flex items-center gap-3 pointer-events-auto"
        style={{
          background: "rgba(0, 0, 0, 0.85)",
          border: "1px solid var(--color-border)",
        }}
      >
        {content.texts.map((text, i) => (
          <span key={i} className="text-sm">
            {text}
          </span>
        ))}
        {content.buttons.map((btn, i) => (
          <button
            key={i}
            onClick={() => {
              if (!disabled) onAction(btn.action);
            }}
            disabled={disabled}
            className="px-2 py-1 rounded text-xs"
            style={{
              background: "var(--color-surface-light)",
              border: "1px solid var(--color-border)",
              cursor: disabled ? "not-allowed" : "pointer",
            }}
          >
            {btn.label}
          </button>
        ))}
      </div>
    </div>
  );
}
