import style from '../styles/Options.module.scss';
import {ImmerHook} from 'use-immer';
import {MacroOptions, OptionsHyprland} from '../lib/data_types.ts';
import {ChangeEvent, JSX, useState} from 'react';
import {AnimatePresence, motion} from 'motion/react';
import {useMeasure} from '@uidotdev/usehooks';

export function OptionsSection({ optionsImmer }: { optionsImmer: ImmerHook<MacroOptions> }) {
  const [expanded, setExpanded] = useState<boolean>(true);
  const [optSecRef, optSecRect] = useMeasure();
  const [firstState, setFirstState] = useState<boolean>(true);

  let child: JSX.Element;
  if (optionsImmer[0].type === 'hyprland') {
    child = <HyprlandOptions optionsImmer={optionsImmer} />
  } else {
    throw new Error(`Unknown options type "${optionsImmer[0].type}"`);
  }

  function toggleExpanded() {
    setExpanded(!expanded);
    setFirstState(false);
  }

  const expandAnim = {
    initial: firstState ? undefined : { opacity: 0, height: 0 },
    animate: firstState ? undefined : { opacity: 1, height: optSecRect.height ?? 0 },
    exit: { opacity: 0, height: 0 },
    transition: {duration: parseFloat(style.animationDuration)}
  }

  console.log(firstState);

  return (
    <div className={[style.optsNotice].join(' ')}>
      <ExpandCollapseButton name="Macro Options" expanded={expanded} toggleExpand={toggleExpanded} firstState={firstState} />
      <AnimatePresence>
        {expanded && (
          <motion.div {...expandAnim}>
            <div ref={optSecRef} className={style.optionsSection}>{child}</div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function ExpandCollapseButton({ name, expanded, toggleExpand, firstState }: {
  name: string;
  expanded: boolean;
  toggleExpand: () => void;
  firstState?: boolean;
}) {
  const iconAnim = {
    initial: firstState ? undefined : { rotateX: -180 },
    animate: { rotateX: 0 },
    exit: { rotateX: 180 },
    transition: {duration: parseFloat(style.animationDuration)}
  }
  return (
    <button className={style.expandCollapseHeader} onClick={toggleExpand}>
      <div className={style.expColIcon}>
        <AnimatePresence>
          {expanded ? (
            <motion.span key="up" className="material-symbols-rounded" {...iconAnim}>expand_circle_up</motion.span>
          ) : (
            <motion.span key="dn" className="material-symbols-rounded" {...iconAnim}>expand_circle_down</motion.span>
          )}
        </AnimatePresence>
      </div>
      <span className={style.headerName}>{name}</span>
    </button>
  );
}

export function HyprlandOptions({ optionsImmer }: { optionsImmer: ImmerHook<OptionsHyprland> }) {
  const [options, setOptions] = optionsImmer;

  function setWinId(e: ChangeEvent<HTMLInputElement>) {
    e.preventDefault();
    if (e.target.value.includes(' ')) {
      // Disallow spaces
      return;
    }
    setOptions(o => {
      o.window_identifier = e.target.value;
    });
  }
  return (
    <>
      <label>
        <span>Window Identifier</span>
        <input
            type='text'
            placeholder='activewindow'
            title='class: or title: id, or leave empty for "activewindow"'
            value={options.window_identifier}
            onChange={setWinId}
        />
      </label>
    </>
  );
}

export function UnsupportedNotice({  }: {  }) {
  return (
    <div className={style.optsNotice}>
      <span>Unfortunately your system is not unsupported</span>
    </div>
  );
}
