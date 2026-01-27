import {FormEvent, ReactNode, useState, MouseEvent} from "react";
import {invoke} from "@tauri-apps/api/core";
import "./App.css";
import "./styles/styles.scss";
import {SysInfo} from "./lib/data_types.ts";
import {MacroSection} from "./components/MacroSection.tsx";


function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="container">
      <form
        className="row"
        onSubmit={greet}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>
      <DebugComp />
      <MacroSection />
    </main>
  );
}

const DebugComp: (props: any) => ReactNode | Promise<ReactNode> = () => {
  const [count, setCount] = useState<number>(-1);
  const [cmdLine, setCmdLine] = useState<string>("");
  const [cmdResult, setCmdResult] = useState<{ exit_code: number, stdout: string }>({exit_code: 0, stdout: ""});
  const [longTaskResult, setLongTaskResult] = useState<string>("");
  const [systemInfo, setSystemInfo] = useState<SysInfo | undefined>(undefined);
  async function getCount(e: MouseEvent<HTMLButtonElement>) {
    e.preventDefault();
    setCount(await invoke("check_count"));
  }
  async function increment(e: MouseEvent<HTMLButtonElement>) {
    e.preventDefault();
    setCount(await invoke("increment_count"));
  }
  async function runCmd(e: MouseEvent<HTMLButtonElement>) {
    e.preventDefault();
    console.log('runCmd', cmdLine);
    setCmdResult(await invoke("run_cmd", { command: cmdLine }));
    console.log('run_cmd returned something');
    console.log(cmdResult);
  }
  async function startLongTask(e: MouseEvent<HTMLButtonElement>) {
    e.preventDefault();
    setLongTaskResult(await invoke("long_task", { ping: 'ping' }));
    console.log('long_task returned');
    console.log(longTaskResult);
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
        <input placeholder="Enter command" onChange={(e) => setCmdLine(e.currentTarget.value)}/>
        <button onClick={runCmd}>Run</button>
        <span style={{border: "1px solid yellow"}}>{JSON.stringify(cmdResult)}</span>
      </div>
      <div>
        <button onClick={startLongTask}>Long Task</button>
        <span>{longTaskResult}</span>
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
