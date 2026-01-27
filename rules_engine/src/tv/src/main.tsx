import React from "react";
import ReactDOM from "react-dom/client";
import { AppRoot } from "./app_root";
import { createLogger } from "./logger_frontend";
import "./styles/app_styles.css";
import "./styles/spreadsheet_overrides.css";

const logger = createLogger("tv.ui.global");

window.addEventListener("error", (event) => {
  logger.error("Uncaught exception", {
    message: event.message,
    filename: event.filename,
    lineno: event.lineno,
    colno: event.colno,
    stack: event.error?.stack,
  });
});

window.addEventListener("unhandledrejection", (event) => {
  const reason = event.reason;
  logger.error("Unhandled promise rejection", {
    reason: reason instanceof Error ? reason.message : String(reason),
    stack: reason instanceof Error ? reason.stack : undefined,
  });
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppRoot />
  </React.StrictMode>,
);
