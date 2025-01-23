import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { FluentBundle, FluentResource } from "@fluent/bundle";
import { LocalizationProvider, ReactLocalization } from "@fluent/react";
import enUS from "../locales/en-US.ftl?raw";

const enUSResource = new FluentResource(enUS);
const bundle = new FluentBundle("en-US", { useIsolating: false });
bundle.addResource(enUSResource);
const l10n = new ReactLocalization([bundle]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <LocalizationProvider l10n={l10n}>
      <App />
    </LocalizationProvider>
  </React.StrictMode>
);
