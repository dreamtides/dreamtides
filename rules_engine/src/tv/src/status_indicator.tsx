// status_indicator.tsx - Sync status indicator component

import { useState, useEffect, useCallback } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type { SyncState } from "./ipc_bridge";
import "./styles/status_indicator.css";

interface SyncStateChangedEvent {
  state: SyncState;
  timestamp: number;
}

interface SyncErrorEvent {
  message: string;
}

export interface StatusIndicatorProps {
  autoHideDelay?: number;
  syncState?: SyncState;
  errorMessage?: string | null;
}

export function StatusIndicator({
  autoHideDelay = 2000,
  syncState,
  errorMessage: propErrorMessage,
}: StatusIndicatorProps): JSX.Element | null {
  const [eventState, setEventState] = useState<SyncState>("idle");
  const [visible, setVisible] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const currentState = syncState ?? eventState;
  const currentError = propErrorMessage ?? errorMessage;

  useEffect(() => {
    if (syncState !== undefined) {
      setVisible(true);

      if (syncState === "idle" || syncState === "saved") {
        const hideTimeout = setTimeout(() => {
          setVisible(false);
        }, autoHideDelay);
        return () => clearTimeout(hideTimeout);
      }
    }
  }, [syncState, autoHideDelay]);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    let hideTimeout: ReturnType<typeof setTimeout> | null = null;

    const setupListener = async () => {
      unlisten = await listen<SyncStateChangedEvent>(
        "sync-state-changed",
        (event) => {
          const newState = event.payload.state;
          setEventState(newState);
          setVisible(true);

          if (hideTimeout) {
            clearTimeout(hideTimeout);
            hideTimeout = null;
          }

          if (newState === "idle" || newState === "saved") {
            hideTimeout = setTimeout(() => {
              setVisible(false);
            }, autoHideDelay);
          }
        }
      );
    };

    if (syncState === undefined) {
      setupListener();
    }

    return () => {
      if (unlisten) unlisten();
      if (hideTimeout) clearTimeout(hideTimeout);
    };
  }, [autoHideDelay, syncState]);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    const setupErrorListener = async () => {
      unlisten = await listen<SyncErrorEvent>("sync-error", (event) => {
        setErrorMessage(event.payload.message);
        setEventState("error");
        setVisible(true);
      });
    };

    if (syncState === undefined) {
      setupErrorListener();
    }

    return () => {
      if (unlisten) unlisten();
    };
  }, [syncState]);

  const handleDismiss = useCallback(() => {
    setVisible(false);
    setErrorMessage(null);
  }, []);

  if (!visible) return null;

  return (
    <div
      className={`tv-status-indicator tv-status-indicator--${currentState}`}
      role="status"
      aria-live="polite"
    >
      <StatusIcon state={currentState} />
      <span className="tv-status-indicator__text">
        {getStatusText(currentState, currentError)}
      </span>
      {currentState === "error" && (
        <button
          className="tv-status-indicator__dismiss"
          onClick={handleDismiss}
          aria-label="Dismiss"
        >
          {"\u00D7"}
        </button>
      )}
    </div>
  );
}

interface StatusIconProps {
  state: SyncState;
}

function StatusIcon({ state }: StatusIconProps): JSX.Element {
  switch (state) {
    case "saving":
      return (
        <span className="tv-status-icon tv-status-icon--saving">{"\u21BB"}</span>
      );
    case "loading":
      return (
        <span className="tv-status-icon tv-status-icon--loading">{"\u21BB"}</span>
      );
    case "error":
      return (
        <span className="tv-status-icon tv-status-icon--error">{"\u26A0"}</span>
      );
    case "saved":
    case "idle":
    default:
      return (
        <span className="tv-status-icon tv-status-icon--idle">{"\u2713"}</span>
      );
  }
}

function getStatusText(state: SyncState, errorMessage: string | null): string {
  switch (state) {
    case "saving":
      return "Saving...";
    case "loading":
      return "Loading...";
    case "error":
      return errorMessage || "Error occurred";
    case "saved":
    case "idle":
    default:
      return "Saved";
  }
}
