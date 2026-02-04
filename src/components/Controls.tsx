import style from "../styles/Controls.module.scss";
import "material-symbols/rounded.css";
import {ActionBoxState, ActionFunctions, MacroState} from "../lib/componentData.ts";
import {AnimatePresence, motion } from "motion/react";
import {ChangeEvent, useState} from "react";
import { useToggle } from "@uidotdev/usehooks";

export function Controls({ aState, mState, funcs }: {aState: ActionBoxState, mState: MacroState, funcs: ActionFunctions }) {
  const [firstState, setFirstState] = useState<boolean>(true);  // skip animation on initial load
  const [showLoops, toggleLoops] = useToggle(false);

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

  function actionToggleLoops() {
    if (showLoops) {
      funcs.setLoopCount(1);
    }
    toggleLoops();
  }

  function actionSetLoops(e: ChangeEvent<HTMLInputElement>) {
    funcs.setLoopCount(parseInt(e.target.value));
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
      <button
          disabled={mState.awaitingResponse || mState.currentState !== 'EDITING'}
          onClick={aState.locked ? undefined : actionToggleLoops}
          style={{color: showLoops ? "var(--col-complementary-fg)" : undefined}}
      >
        <span className="material-symbols-rounded">change_circle</span>
      </button>
      <AnimatePresence>
        {showLoops && (
          <motion.input
              initial={{opacity: 0, scale: 0, padding: 0, width: 0}}
              animate={{opacity: 1, scale: 1, padding: "0.2em 0.4em", width: "3em"}}
              exit={{opacity: 0, scale: 0, padding: 0, width: 0}}
              transition={{duration: parseFloat(style.animationDuration)}}
              disabled={mState.awaitingResponse || mState.currentState !== 'EDITING'}
              type="number"
              min="0"
              max="100"
              step="1"
              value={aState.loops}
              onChange={aState.locked ? undefined : actionSetLoops}
          />
        )}
      </AnimatePresence>
    </div>
  );
}
