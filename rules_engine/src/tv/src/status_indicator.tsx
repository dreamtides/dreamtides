// status_indicator.tsx - Sync status indicator component

import { useEffect, useState, useRef } from "react";
import * as ipc from "./ipc_bridge";
import type { SyncState } from "./ipc_bridge";
import "./styles/status_indicator_styles.css";

export interface StatusIndicatorProps {
  autoDismissMs?: number;
}

const SAVED_AUTO_DISMISS_MS = 3000;

const STATUS_CONFIG: Record<
  SyncState,
  { icon: string; label: string; className: string }
> = {
  idle: { icon: "", label: "", className: "" },
  saving: { icon: "\u231B", label: "Saving...", className: "tv-status-saving" },
  loading: {
    icon: "\u21BB",
    label: "Loading...",
    className: "tv-status-loading",
  },
  saved: { icon: "\u2713", label: "Saved", className: "tv-status-saved" },
  error: { icon: "\u26A0", label: "Error", className: "tv-status-error" },
};

export function StatusIndicator({
  autoDismissMs = SAVED_AUTO_DISMISS_MS,
}: StatusIndicatorProps) {
  const [syncState, setSyncState] = useState<SyncState>("idle");
  const [visible, setVisible] = useState(false);
  const dismissTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    const subscription = ipc.onSyncStateChanged((payload) => {
      setSyncState(payload.state);
      // Don't show indicator for idle or saving states - saving indicator
      // was explicitly decided against to avoid visual noise during edits.
      // Similarly, ability parsing runs silently in the background without
      // any visual status indication to avoid distracting the user.
      setVisible(payload.state !== "idle" && payload.state !== "saving");

      if (dismissTimeoutRef.current) {
        clearTimeout(dismissTimeoutRef.current);
        dismissTimeoutRef.current = null;
      }

      if (payload.state === "saved") {
        dismissTimeoutRef.current = window.setTimeout(() => {
          setVisible(false);
          setSyncState("idle");
        }, autoDismissMs);
      }
    });

    return () => {
      subscription.dispose();
      if (dismissTimeoutRef.current) {
        clearTimeout(dismissTimeoutRef.current);
      }
    };
  }, [autoDismissMs]);

  if (!visible) {
    return null;
  }

  const config = STATUS_CONFIG[syncState];
  if (!config.icon) {
    return null;
  }

  return (
    <div
      className={`tv-status-indicator ${config.className}`}
      role="status"
      aria-live="polite"
    >
      <span className="tv-status-icon">{config.icon}</span>
      <span className="tv-status-label">{config.label}</span>
    </div>
  );
}
