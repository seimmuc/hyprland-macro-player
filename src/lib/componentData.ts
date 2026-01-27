import {Action, ActionType, KeyCombo, ModifierKey, ProgressInfo, RustMacro} from "./data_types.ts";
import {KeyboardEvent} from "react";
import {wordToTitleCase} from "./utils.ts";

export interface ActionBoxState {
  locked: boolean;
  progress: ProgressInfo | undefined;
  actions: Action[];
  loops: number;
}

export interface MacroState {
  currentState: 'EDITING' | 'PLAYING' | 'PAUSED';
  activeMacro: RustMacro | undefined;
  awaitingResponse: boolean;
  error: string | undefined;
}

export interface ActionFunctions {
  setActions: (newActions: Action[]) => void;
  editAction: (actionId: number, func: (action: Action) => void) => void;
  createNewAction: (type: ActionType) => Action;
  removeAction: (actionId: Action["actionId"]) => boolean;
  // setLoopCount TODO loops;
  controls: {
    play: () => Promise<boolean>;
    pause: () => Promise<boolean>;
    stop: () => Promise<boolean>;
  }
}

export function keyComboReact(e: KeyboardEvent): KeyCombo {
  const modifiers: ModifierKey[] = [];
  if (e.shiftKey) {
    modifiers.push('shift');
  }
  if (e.ctrlKey) {
    modifiers.push('ctrl');
  }
  if (e.altKey) {
    modifiers.push('alt');
  }
  if (e.metaKey) {
    modifiers.push('super');
  }
  return {key: e.key.toLowerCase(), modifiers};
}

export function keyComboToString(kc: KeyCombo): string {
  return [...(kc.modifiers ?? []), kc.key].map(wordToTitleCase).join(' ');
}

export function actionsToRust(actions: Action[]): RustMacro["actions"] {
  const result: RustMacro["actions"] = [];
  for (const action of actions) {
    const { actionId, ...rustAction } = action;
    result.push(rustAction);
  }
  return result;
}
