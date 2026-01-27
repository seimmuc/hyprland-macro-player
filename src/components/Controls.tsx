import style from "../styles/Controls.module.scss";
import "material-symbols/rounded.css";
import {ActionBoxState, ActionFunctions, MacroState} from "../lib/componentData.ts";
import {AnimatePresence, motion } from "motion/react";
import {useState} from "react";

export function Controls({ aState, mState, funcs }: {aState: ActionBoxState, mState: MacroState, funcs: ActionFunctions }) {
  const [firstState, setFirstState] = useState<boolean>(true);  // skip animation on initial load
  function actionPlayPause() {
    if (mState.awaitingResponse) {
      return;
    }
    if (mState.currentState === 'PLAYING') {
      funcs.controls.pause();
    } else {
      funcs.controls.play();
    }
    setFirstState(false);
  }

  function actionStop() {
    if (mState.awaitingResponse || mState.currentState === 'EDITING') {
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
  const playDisabled = mState.awaitingResponse ||
      (mState.currentState === 'EDITING' && aState.actions.length === 0);

  return (
    <div className={style.controlsBox}>
      <button className={style.switchingIcon} disabled={playDisabled} onClick={actionPlayPause}>
        <AnimatePresence>
          {mState.currentState !== 'PLAYING' ? (
            <motion.span key="pl" className="material-symbols-rounded" {...playPauseAnim}>play_circle</motion.span>
          ) : (
            <motion.span key="pa" className="material-symbols-rounded" {...playPauseAnim}>pause_circle</motion.span>
          )}
        </AnimatePresence>
      </button>
      <button disabled={mState.awaitingResponse || mState.currentState === 'EDITING'} onClick={actionStop}>
        <span className="material-symbols-rounded">stop_circle</span>
      </button>
      <button disabled={mState.awaitingResponse || mState.currentState !== 'EDITING'} onClick={actionStop}>
        <span className="material-symbols-rounded">change_circle</span>
      </button>
      {/*<AnimatePresence>*/}
      {/*  <input type="number" /> TODO loops */}
      {/*</AnimatePresence>*/}
    </div>
  );
}
