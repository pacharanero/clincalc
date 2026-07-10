// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

import React from "react";
import ReactDOM from "react-dom/client";
import { createTheme, MantineProvider } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import App from "./App";
import { ErrorBoundary } from "./ErrorBoundary";
import "./App.css";

// Theme mirrors the gitehr GUI palette while using a quieter, neutral Lato
// typeface for both body and headings. `defaultRadius: "md"` keeps cards /
// buttons softly-rounded rather than boxy; clinicians have read enough
// hospital-IT chrome for a lifetime.
const theme = createTheme({
  fontFamily: "Lato, system-ui, -apple-system, sans-serif",
  headings: { fontFamily: "Lato, system-ui, -apple-system, sans-serif" },
  primaryColor: "teal",
  defaultRadius: "md",
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MantineProvider theme={theme} defaultColorScheme="auto">
      <Notifications position="top-right" />
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </MantineProvider>
  </React.StrictMode>,
);
