import style from "../styles/ActionBox.module.scss";
import "material-symbols/rounded.css";
import {Action, ActionType, SleepAction, KeyAction, CraftAction, KeyCombo} from "../lib/data_types.ts";
import {JSX, useState, KeyboardEvent, ChangeEvent} from "react";
import {AnimatePresence, motion, Reorder } from "motion/react";
import {useDragControls} from "framer-motion";

import {ActionFunctions, ActionBoxState, keyComboReact, keyComboToString} from "../lib/componentData.ts";


export function ActionBox({ state, paused, funcs }: { state: ActionBoxState; paused: boolean; funcs: ActionFunctions }) {
  return (
    <div className={style.actionsBox}>
      <ActionList state={state} paused={paused} funcs={funcs} />
      <ActionsToolbar createAction={funcs.createNewAction} locked={state.locked} />
    </div>
  );
}

export function ActionsToolbar({ createAction, locked }: { createAction: (type: ActionType) => Action; locked: boolean }) {
  return (
    <div className={style.toolbar}>
      <div className={style.newActionSection}>
        <span>Add</span>
        <button disabled={locked} onClick={locked ? undefined : () => createAction("sleep")}>
          <span className="icon material-symbols-rounded">timer</span>
          <span className="label">Sleep</span>
        </button>
        <button disabled={locked} onClick={locked ? undefined : () => createAction("key")}>
          <span className="icon material-symbols-rounded">keyboard_alt</span>
          <span className="label">Key</span>
        </button>
        <button disabled={locked} onClick={locked ? undefined : () => createAction("craft")}>
          {/* alternate icons: hardware, gavel, home_repair_service, assignment_add */}
          <span className="icon material-symbols-rounded">order_play</span>
          <span className="label">Craft</span>
        </button>
      </div>
      <div className={style.actionsStatus}>
        <span>Cycle time: idk</span>
      </div>
    </div>
  );
}

export function ActionList({ state, paused, funcs }: { state: ActionBoxState; paused: boolean; funcs: ActionFunctions; }) {
  function ap(index: number): number | undefined {
    if (state.progress !== undefined) {
      const cur_action_index = state.progress.action_index;
      if (index < cur_action_index) {
        return -1;
      }
      if (index > cur_action_index) {
        return 2;
      }
      if (index === cur_action_index) {
        return state.progress.action_progress;
      }
    }
  }
  return (
    <Reorder.Group as="ol" axis="y" className={style.actionsList} values={state.actions} onReorder={funcs.setActions}>
      <AnimatePresence>
        {state.actions.map((action, i) => (
          <ActionRootComp
              key={action.actionId}
              action={action}
              locked={state.locked}
              paused={paused}
              funcs={funcs}
              progress={ap(i)}
          />
        ))}
      </AnimatePresence>
    </Reorder.Group>
  );
}

export function ActionRootComp({ action, locked, paused, funcs, progress }: {
  action: Action;
  locked: boolean;
  paused: boolean;
  funcs: ActionFunctions;
  progress: number | undefined;
}) {
  let child: JSX.Element;
  if (action.action === "sleep") {
    child = <ActionSleep action={action} locked={locked} editAction={funcs.editAction} />;
  } else if (action.action === "key") {
    child = <ActionKey action={action} locked={locked} editAction={funcs.editAction} />;
  } else if (action.action === "craft") {
    child = <ActionCraft action={action} locked={locked} editAction={funcs.editAction} />;
  } else {
    // @ts-expect-error TS2339
    throw new Error(`Unknown action type "${action.action}"`);
  }
  const controls = locked ? undefined : useDragControls();
  const cls: string[] = [style.action];
  let progressPercent: string | undefined = undefined;
  if (progress !== undefined) {
    if (progress < 0) {
      cls.push(style.idle);
    } else if (progress > 1) {
      cls.push(style.complete);
    } else {
      cls.push(style.inProgress);
      progressPercent = `${Math.round(progress * 100)}%`;
    }
    if (paused) {
      cls.push(style.paused);
    }
  }

  return (
    <Reorder.Item value={action} dragListener={false} dragControls={controls} style={{
      // @ts-expect-error TS2353: Object literal may only specify known properties
      '--progress': progressPercent
    }}>
      <motion.div
          className={cls.join(' ')}
          initial={{ opacity: 0, scale: 0, height: 0 }}
          animate={{ opacity: 1, scale: 1, height: style.actionHeight }}
          exit={{ opacity: 0, scale: 0, height: 0 }}
          transition={{ duration: parseFloat(style.animationDuration) }}
      >
        <div className={style.dragHandle} onPointerDown={controls === undefined ? undefined : (e) => controls.start(e)}>
          <span className="material-symbols-rounded">drag_handle</span>
        </div>
        <div className={style.actionContent}>{child}</div>
        <button onClick={() => funcs.removeAction(action.actionId)} disabled={locked}>
          <span className="material-symbols-rounded">close</span>
        </button>
      </motion.div>
    </Reorder.Item>
  );
}

function ActionSleep({action, locked, editAction }: { action: SleepAction; locked: boolean; editAction: ActionFunctions["editAction"] }) {
  const waitTime = action.duration_ms / 1000;
  function onChange(e: ChangeEvent<HTMLInputElement>) {
    if (locked) {
      e.preventDefault();
      return;
    }
    const time = parseFloat(e.target.value);
    editAction(action.actionId, act => {(act as SleepAction).duration_ms = Math.floor(time * 1000)});
  }
  return (
    <>
      <span>Sleep</span>
      <input type="range" min="0" max="15" step="0.1" value={waitTime} disabled={locked} onChange={onChange} />
      <input type="number" min="0" max="180" step="0.1" value={waitTime} disabled={locked} onChange={onChange} />
    </>
  );
}

function ActionKey({ action, locked, editAction }: { action: KeyAction; locked: boolean; editAction: ActionFunctions["editAction"] }) {
  const [capturing, setCapturing] = useState<boolean>(false);
  // const [keysDown, setKeysDown] = useState<string[]>([]);
  function onKeyDown(e: KeyboardEvent) {
    if (!capturing || locked) {
      return;
    }
    e.preventDefault();
    const key = e.key.toLowerCase();
    if (["shift", "alt", "control", "super"].includes(key)) {
      return;
    }
    const kc: KeyCombo | undefined = key === "escape" ? undefined : keyComboReact(e);
    console.log('onKeyDown', JSON.stringify(kc));
    setCapturing(false);
    editAction(action.actionId, act => {(act as KeyAction).key = kc});
  }
  return (
    <>
      <span>Key</span>
      <button onClick={locked ? undefined : () => setCapturing(!capturing)} onKeyDown={onKeyDown} disabled={locked}>
        <span className="material-symbols-rounded">{capturing ? "screen_record" : "radio_button_unchecked"}</span>
      </button>
      {(action.key !== undefined && action.key !== null) && (
        <span className={style.keyLabel}>{keyComboToString(action.key)}</span>
      )}
    </>
  );
}

function ActionCraft({action, locked, editAction}: { action: CraftAction; locked: boolean; editAction: ActionFunctions["editAction"] }) {
  return (
    <>
      <span>Craft</span>
    </>
  );
}
