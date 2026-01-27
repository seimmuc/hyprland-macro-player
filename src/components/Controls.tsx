import style from "../styles/Controls.module.scss";
import "material-symbols/rounded.css";
import {ActionFunctions, MacroState} from "../lib/componentData.ts";
import {AnimatePresence, motion } from "motion/react";
import {useState} from "react";

export function Controls({ state, funcs }: { state: MacroState, funcs: ActionFunctions }) {
  const [firstState, setFirstState] = useState<boolean>(true);  // skip animation on initial load
  function actionPlayPause() {
    if (state.awaitingResponse) {
      return;
    }
    if (state.currentState === 'PLAYING') {
      funcs.controls.pause();
    } else {
      funcs.controls.play();
    }
    setFirstState(false);
  }

  function actionStop() {
    if (state.awaitingResponse || state.currentState === 'EDITING') {
      return;
    }
    funcs.controls.stop();
  }

  const playPauseAnim = {
    initial: firstState ? undefined : { rotateY: -180 },
    animate: { rotateY: 0 },
    exit: { rotateY: 180 },
    transition: {duration: parseFloat(style.animationDuration)}
  }

  return (
    <div className={style.controlsBox}>
      <button className={style.switchingIcon} disabled={state.awaitingResponse} onClick={actionPlayPause}>
        <AnimatePresence>
          {state.currentState !== 'PLAYING' ? (
            <motion.span key="pl" className="material-symbols-rounded" {...playPauseAnim}>play_circle</motion.span>
          ) : (
            <motion.span key="pa" className="material-symbols-rounded" {...playPauseAnim}>pause_circle</motion.span>
          )}
        </AnimatePresence>
      </button>
      <button disabled={state.awaitingResponse || state.currentState === 'EDITING'} onClick={actionStop}>
        <span className="material-symbols-rounded">stop_circle</span>
      </button>
      <button disabled={state.awaitingResponse || state.currentState !== 'EDITING'} onClick={actionStop}>
        <span className="material-symbols-rounded">change_circle</span>
      </button>
      {/*<AnimatePresence>*/}
      {/*  <input type="number" /> TODO loops */}
      {/*</AnimatePresence>*/}
    </div>
  );
}
