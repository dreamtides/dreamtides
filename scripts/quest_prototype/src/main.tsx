import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "boxicons/css/boxicons.min.css";
import App from "./App.tsx";
import "./index.css";
import { parseRuntimeConfig } from "./runtime/runtime-config";

const runtimeConfig = parseRuntimeConfig(window.location.search);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App runtimeConfig={runtimeConfig} />
  </StrictMode>,
);
