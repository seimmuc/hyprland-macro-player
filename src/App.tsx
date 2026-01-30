import {ReactNode, useState, MouseEvent} from "react";
import {invoke} from "@tauri-apps/api/core";
import "./styles/styles.scss";
import {SysInfo} from "./lib/data_types.ts";
import {MacroSection} from "./components/MacroSection.tsx";


function App() {
  return (
    <main className="container">
      <DebugComp />
      <MacroSection />
    </main>
  );
}

const DebugComp: (props: any) => ReactNode | Promise<ReactNode> = () => {
  const [count, setCount] = useState<number>(-1);
  const [systemInfo, setSystemInfo] = useState<SysInfo | undefined>(undefined);
  async function getCount(e: MouseEvent<HTMLButtonElement>) {
    e.preventDefault();
    setCount(await invoke("check_count"));
  }
  async function increment(e: MouseEvent<HTMLButtonElement>) {
    e.preventDefault();
    setCount(await invoke("increment_count"));
  }

  if (systemInfo === undefined) {
    console.info('scheduling system_info call');
    invoke<SysInfo | undefined>("system_info").then(setSystemInfo);
  }
  if (count === -1) {
    invoke<number>("check_count").then(setCount);
  }

  return (
    <div>
      <div>
        <span>Count: {count}</span>
        <button onClick={getCount}>Get count</button>
        <button onClick={increment}>Increment</button>
      </div>
      <div>
        {systemInfo === undefined ? (
            <p>Fetching system info</p>
        ) : (
            <p>Your system info: {JSON.stringify(systemInfo)}</p>
        )}
      </div>
    </div>
  );
}

export default App;
