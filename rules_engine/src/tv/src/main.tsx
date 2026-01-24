import React from "react";
import ReactDOM from "react-dom/client";
import { AppRoot } from "./app_root";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppRoot />
  </React.StrictMode>,
);
