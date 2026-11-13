import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { NotchWindow } from "./components/NotchWindow";
import "./App.css";

const urlParams = new URLSearchParams(window.location.search);
const isNotch =
  urlParams.get("window") === "notch" ||
  window.location.hash === "#notch" ||
  window.location.pathname.endsWith("/notch");

if (isNotch) {
  document.documentElement.classList.add("notch-mode");
  document.body.classList.add("notch-mode");
  document.documentElement.style.backgroundColor = "transparent";
  document.body.style.backgroundColor = "transparent";
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {isNotch ? <NotchWindow /> : <App />}
  </React.StrictMode>,
);
