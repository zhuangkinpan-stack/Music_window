import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import App from "./App";
import { QuickBarApp } from "./features/quick-bar/QuickBarApp";
import "./styles.css";

const quickWindow =
  new URLSearchParams(window.location.search).get("window") === "quick";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    {quickWindow ? <QuickBarApp /> : <App />}
  </StrictMode>,
);

