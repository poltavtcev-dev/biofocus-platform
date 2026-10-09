import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

// QA: `?qaWidth=320` previews the real menubar width in a normal browser
// (headless Chrome cannot shrink its viewport below ~500px).
const qaWidth = Number(new URLSearchParams(window.location.search).get("qaWidth"));
if (Number.isFinite(qaWidth) && qaWidth > 0) {
  document.body.style.maxWidth = `${qaWidth}px`;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
