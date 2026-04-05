import type { ButtonView, GameAction } from "../types/battle";

/** Strip Unity rich text tags like <color=#hex>...</color> from labels */
function stripRichText(text: string): string {
  return text.replace(/<\/?color[^>]*>/gi, "");
}

interface ActionBarProps {
  primaryButton?: ButtonView;
  secondaryButton?: ButtonView;
  undoButton?: ButtonView;
  devButton?: ButtonView;
  incrementButton?: ButtonView;
  decrementButton?: ButtonView;
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

function ActionButton({
  button,
  onAction,
  disabled,
  primary,
}: {
  button: ButtonView;
  onAction: (action: GameAction) => void;
  disabled: boolean;
  primary?: boolean;
}) {
  const isDisabled = disabled || button.action == null;
  return (
    <button
      onClick={() => {
        if (!isDisabled && button.action != null) onAction(button.action);
      }}
      disabled={isDisabled}
      className="px-4 py-2 rounded text-sm font-bold"
      style={{
        background: primary
          ? "var(--color-primary)"
          : "var(--color-surface-light)",
        color: isDisabled ? "var(--color-text-dim)" : "var(--color-text)",
        cursor: isDisabled ? "not-allowed" : "pointer",
        opacity: isDisabled ? 0.5 : 1,
        border: "1px solid var(--color-border)",
      }}
    >
      {stripRichText(button.label)}
    </button>
  );
}

export function ActionBar({
  primaryButton,
  secondaryButton,
  undoButton,
  devButton,
  incrementButton,
  decrementButton,
  onAction,
  disabled,
}: ActionBarProps) {
  const hasButtons =
    primaryButton ?? secondaryButton ?? undoButton ?? devButton
    ?? incrementButton ?? decrementButton;
  if (!hasButtons) return null;

  return (
    <div
      className="flex gap-2 justify-center items-center py-2 px-4"
      style={{
        background: "var(--color-surface)",
        borderTop: "1px solid var(--color-border)",
        position: "relative",
        zIndex: 45,
      }}
    >
      {incrementButton && (
        <ActionButton button={incrementButton} onAction={onAction} disabled={disabled} />
      )}
      {decrementButton && (
        <ActionButton button={decrementButton} onAction={onAction} disabled={disabled} />
      )}
      {secondaryButton && (
        <ActionButton button={secondaryButton} onAction={onAction} disabled={disabled} />
      )}
      {primaryButton && (
        <ActionButton
          button={primaryButton}
          onAction={onAction}
          disabled={disabled}
          primary
        />
      )}
      {undoButton && (
        <ActionButton button={undoButton} onAction={onAction} disabled={disabled} />
      )}
      {devButton && (
        <ActionButton button={devButton} onAction={onAction} disabled={disabled} />
      )}
    </div>
  );
}
