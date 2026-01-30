import {ActionBox} from "./ActionBox.tsx";
import {
  Action,
  ActionType,
  MacroEvent,
  MacroOptions,
  PausedEvent,
  RunningEvent,
  RustMacro,
  StoppedEvent
} from "../lib/data_types.ts";
import {useImmer} from "use-immer";
import {ActionFunctions, ActionBoxState, MacroState, actionsToRust} from "../lib/componentData.ts";
import {Controls} from "./Controls.tsx";
import {invoke} from "@tauri-apps/api/core";
import {useEffect, useRef} from "react";
import {listen, Event} from "@tauri-apps/api/event";

/** Next action ID */
let naId = 0;
/** Next macro ID */
let nmId = 0;

export function MacroSection({ options }: { options: MacroOptions | undefined }) {
  // ActionBoxState
  let [abState, setAbState] = useImmer<ActionBoxState>({
    locked: options === undefined,
    progress: undefined,
    actions: [],
    loops: 1
  });
  // MacroState
  let [macroState, setMacroState] = useImmer<MacroState>({
    currentState: 'EDITING',
    activeMacro: undefined,
    awaitingResponse: options === undefined,  // Hacky, this is just to lock it on unsupported systems
    error: undefined,
  });
  let macroStateRef = useRef(macroState);
  useEffect(() => {
    macroStateRef.current = macroState;
  }, [macroState]);

  const abFuncs: ActionFunctions = {
    setActions: (newActions: Action[]): void => {
      setAbState(abs => {abs.actions = newActions});
    },
    editAction: (actionId: number, func: (action: Action) => void): void => {
      setAbState(abs => {
        for (const action of abs.actions) {
          if (action.actionId === actionId) {
            func(action);
            return;
          }
        }
      });
    },
    createNewAction: (type: ActionType): Action => {
      const actionId = naId++;
      let newAction: Action | null;
      switch (type) {
        case "sleep":
          newAction = {actionId, action: "sleep", duration_ms: 1000};
          break;
        case "key":
          newAction = {actionId, action: "key", key: undefined};
          break;
        case "craft":
          newAction = {actionId, action: "craft"};
          break;
        default:
          throw new Error(`Unknown action type: ${type}`);
      }
      setAbState(abs => {
        abs.actions.push(newAction)
      });
      return newAction;
    },
    removeAction: (actionId: number): boolean => {
      const actId = abState.actions.findIndex(a => a.actionId === actionId);
      if (actId === -1) {
        return false;
      }
      setAbState(abs => {
        abs.actions = abs.actions.filter(a => a.actionId !== actionId);
      });
      return true;
    },
    controls: {
      play: async (): Promise<boolean> => {
        if (macroState.awaitingResponse || macroState.currentState === 'PLAYING' || options === undefined) {
          return false;
        }
        switch (macroState.currentState) {
          case 'EDITING':
            // start new macro
            if (macroState.activeMacro !== undefined) {
              throw new Error('cannot play: activeMacro is not undefined');
            }
            if (abState.actions.length < 1) {
              return false;
            }
            const macr: RustMacro = {
              id: nmId++,
              actions: actionsToRust(abState.actions),
              loops: abState.loops,
              options
            };
            setMacroState(ms => { ms.awaitingResponse = true; });
            setAbState(abs => { abs.locked = true; });

            // send command and wait for response
            const stResponse = await invoke<RunningEvent>('start_macro', { macr });

            if (stResponse.event_type !== 'running') {
              throw new Error('start_macro must only return RunningEvent');
            }
            if (stResponse.id !== macr.id) {
              throw new Error('start_macro returned a different macro id');
            }
            setMacroState({
              awaitingResponse: false,
              currentState: 'PLAYING',
              activeMacro: macr,
              error: undefined,
            });
            setAbState(abs => {
              abs.progress = stResponse.progress;
            });
            return true;
          case 'PAUSED':
            // resume paused macro
            if (macroState.activeMacro === undefined) {
              throw new Error('cannot resume: activeMacro is undefined');
            }
            setMacroState(ms => { ms.awaitingResponse = true; });

            // send command and wait for response
            const rsResponse = await invoke<RunningEvent>('resume_macro', { id: macroState.activeMacro.id });

            if (rsResponse.event_type !== 'running') {
              throw new Error('resume_macro must only return RunningEvent');
            }
            if (rsResponse.id !== macroState.activeMacro.id) {
              throw new Error('start_macro returned a different macro id');
            }
            setMacroState(ms => {
              ms.awaitingResponse = false;
              ms.currentState = 'PLAYING';
            });
            setAbState(abs => {
              abs.progress = rsResponse.progress;
            });
            return true;
        }
      },
      pause: async (): Promise<boolean> => {
        // pause running macro
        if (macroState.awaitingResponse || macroState.currentState !== 'PLAYING') {
          return false;
        }
        if (macroState.activeMacro === undefined) {
          throw new Error('cannot pause: activeMacro is undefined');
        }
        setMacroState(ms => { ms.awaitingResponse = true; });

        // send command and wait for response
        const response = await invoke<PausedEvent>('pause_macro', { id: macroState.activeMacro.id });

        if (response.event_type !== 'paused') {
          throw new Error('pause_macro must only return PausedEvent');
        }
        if (response.id !== macroState.activeMacro.id) {
          throw new Error('pause_macro returned a different macro id');
        }
        setMacroState(ms => {
          ms.currentState = 'PAUSED';
          ms.awaitingResponse = false;
        });
        setAbState(abs => {
          abs.progress = response.progress;
        });
        return true;
      },
      stop: async (): Promise<boolean> => {
        // stop the macro
        if (macroState.awaitingResponse || macroState.currentState === 'EDITING') {
          return false;
        }
        if (macroState.activeMacro === undefined) {
          throw new Error('cannot stop: activeMacro is undefined');
        }
        setMacroState(ms => { ms.awaitingResponse = true; });

        // send command and wait for response
        const response = await invoke<StoppedEvent>('stop_macro', { id: macroState.activeMacro.id });

        if (response.event_type !== 'stopped') {
          throw new Error('stop_macro must only return StoppedEvent');
        }
        if (response.id !== macroState.activeMacro.id) {
          throw new Error('stop_macro returned a different macro id');
        }
        post_stop();
        return true;
      }
    }
  }

  function post_stop() {
    setMacroState(ms => {
      ms.currentState = 'EDITING';
      ms.activeMacro = undefined;
      ms.awaitingResponse = false;
    });
    setAbState(abs => {
      abs.progress = undefined;
      abs.locked = false;
    });
  }

  useEffect(() => {
    const onMacroEvent = (event: Event<MacroEvent>) => {
      const me: MacroEvent = event.payload;
      const ms: MacroState = macroStateRef.current;
      // TODO queue events if ms.awaitingResponse is true and process them later
      if (ms.awaitingResponse || ms.currentState === 'EDITING' || ms.activeMacro === undefined || ms.activeMacro.id !== me.id) {
        return;
      }
      switch (me.event_type) {
        case 'update':
          setAbState(abs => {
            abs.progress = me.progress;
          });
          break;
        case 'stopped':
          post_stop();
          break;
        case 'error':
          setAbState(abs => {
            abs.progress = me.progress;
          });
          setMacroState(ms => {
            ms.error = me.error;
          });
          break;
        default:
          throw new Error(`Invalid event_type: "${me.event_type}", can only handle "update" and "stopped"`);
      }
    }

    const unlistenPromise = listen<MacroEvent>('macro_event', onMacroEvent);
    return () => {
      unlistenPromise.then(unlisten => unlisten());
      if (macroStateRef.current.currentState !== 'EDITING') {
        abFuncs.controls.stop();
      }
    }
  }, []);

  return (
    <>
      <ActionBox state={abState} paused={macroState.currentState === 'PAUSED'} funcs={abFuncs} />
      <Controls aState={abState} mState={macroState} funcs={abFuncs} />
    </>
  );
}
