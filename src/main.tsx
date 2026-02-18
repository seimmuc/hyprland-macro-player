import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import {SysInfo} from "./lib/data_types.ts";
import {invoke} from "@tauri-apps/api/core";

const sysInfo = await invoke<SysInfo>("system_info");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App sysInfo={sysInfo} />
  </React.StrictMode>,
);
