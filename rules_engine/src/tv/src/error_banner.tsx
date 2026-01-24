// error_banner.tsx - Error display overlay component

import "./styles/error_styles.css";

export interface ErrorBannerProps {
  message: string;
  errorType?: "error" | "warning";
  onDismiss?: () => void;
  actions?: Array<{
    label: string;
    onClick: () => void;
  }>;
}

export function ErrorBanner({
  message,
  errorType = "error",
  onDismiss,
  actions,
}: ErrorBannerProps) {
  const bannerClass =
    errorType === "warning" ? "tv-error-banner tv-warning" : "tv-error-banner";

  return (
    <div className={bannerClass} role="alert">
      <div className="tv-error-icon">{errorType === "error" ? "\u26A0" : "\u26A1"}</div>
      <div className="tv-error-content">
        <span className="tv-error-message">{message}</span>
        {actions && actions.length > 0 && (
          <div className="tv-error-actions">
            {actions.map((action, index) => (
              <button
                key={index}
                className="tv-error-action-btn"
                onClick={action.onClick}
              >
                {action.label}
              </button>
            ))}
          </div>
        )}
      </div>
      {onDismiss && (
        <button
          className="tv-error-dismiss"
          onClick={onDismiss}
          aria-label="Dismiss"
        >
          {"\u2715"}
        </button>
      )}
    </div>
  );
}

export function getErrorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    return String(error.message);
  }
  return "An unknown error occurred";
}
