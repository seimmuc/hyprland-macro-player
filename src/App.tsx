import {ReactNode, useState, MouseEvent} from "react";
import {invoke} from "@tauri-apps/api/core";
import "./styles/styles.scss";
import {MacroOptions, OptionsHyprland, SysInfo} from "./lib/data_types.ts";
import {MacroSection} from "./components/MacroSection.tsx";
import {ImmerHook, useImmer} from "use-immer";
import {createInitOptions} from "./lib/componentData.ts";
import {OptionsSection, UnsupportedNotice} from "./components/Options.tsx";


function App({sysInfo}: {sysInfo: SysInfo}) {
  const optImm = useImmer<MacroOptions | undefined>(() => createInitOptions(sysInfo));

  return (
    <main className="container">
      {optImm[0] === undefined ?
          <UnsupportedNotice /> :
          <OptionsSection optionsImmer={optImm as ImmerHook<OptionsHyprland>} />
      }
      <DebugComp />
      <MacroSection options={optImm[0]} />
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
