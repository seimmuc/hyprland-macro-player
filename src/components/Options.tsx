import style from '../styles/Options.module.scss';
import {ImmerHook} from 'use-immer';
import {MacroOptions, OptionsHyprland, OptionsWindows} from '../lib/data_types.ts';
import {ChangeEvent, JSX, useState} from 'react';
import {AnimatePresence, motion} from 'motion/react';
import {useMeasure} from '@uidotdev/usehooks';
import {ButtonGroup} from "./ButtonGroup.tsx";
import {wordToTitleCase} from "../lib/utils.ts";

const LABEL_ANIM = {
  initial: {opacity: 0, scaleY: 0, height: 0, marginTop: `-${style.optionsLblGap}`},
  animate: {opacity: 1, scaleY: 1, height: 'auto', marginTop: 0},
  exit: {opacity: 0, scaleY: 0, height: 0, marginTop: `-${style.optionsLblGap}`},
  transition: {duration: parseFloat(style.animationDuration)}
};

export function OptionsSection({ optionsImmer }: { optionsImmer: ImmerHook<MacroOptions> }) {
  const [expanded, setExpanded] = useState<boolean>(true);
  const [optSecRef, optSecRect] = useMeasure();
  const [firstState, setFirstState] = useState<boolean>(true);

  let child: JSX.Element;
  if (optionsImmer[0].type === 'hyprland') {
    child = <HyprlandOptions optionsImmer={optionsImmer as ImmerHook<OptionsHyprland>} />
  } else if (optionsImmer[0].type === 'windows') {
    child = <WindowsOptions optionsImmer={optionsImmer as ImmerHook<OptionsWindows>} />
  } else {
    // @ts-expect-error TS2339
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

const WINDOWS_WID_MODES: OptionsWindows['window_id_mode'][] = ['none', 'title', 'process'];
const WINDOWS_MATCH_MODES: OptionsWindows['match_mode'][] = ['simple', 'regex'];

export function WindowsOptions({ optionsImmer }: { optionsImmer: ImmerHook<OptionsWindows> }) {
  const [options, setOptions] = optionsImmer;

  function onWinModeChange(newVal: Set<string | number>) {
    if (newVal.size !== 1) {
      return;
    }
    const selVal = newVal.values().next().value as OptionsWindows['window_id_mode'];
    if (!WINDOWS_WID_MODES.includes(selVal)) {
      return;
    }
    setOptions(o => { o.window_id_mode = selVal; });
  }
  function onWinStrChange(e: ChangeEvent<HTMLInputElement>) {
    e.preventDefault();
    setOptions(o => {
      o.window_id_str = e.target.value;
    });
  }
  function onWinStrModeChange(newVal: Set<string | number>) {
    if (newVal.size !== 1) {
      return;
    }
    const selVal = newVal.values().next().value as OptionsWindows['match_mode'];
    if (!WINDOWS_MATCH_MODES.includes(selVal)) {
      return;
    }
    setOptions(o => {
      o.match_mode = selVal;
    })
  }
  function onAutoFocChange(e: ChangeEvent<HTMLInputElement>) {
    setOptions(o => {
      o.auto_focus = e.target.checked;
    });
  }
  return (
    <>
      <label>
        <span>Window input</span>
        <ButtonGroup selectionMode="single" onSelectionChange={onWinModeChange} selectedKeys={new Set([options.window_id_mode])}>
          {WINDOWS_WID_MODES.map(mode => (
            <ButtonGroup.Button key={mode} id={mode}>{wordToTitleCase(mode)}</ButtonGroup.Button>
          ))}
        </ButtonGroup>
      </label>
      <AnimatePresence>
        {options.window_id_mode !== 'none' && (
          <>
            <motion.label {...LABEL_ANIM}>
              <span>Window matcher</span>
              <motion.input
                  type="text"
                  placeholder={options.window_id_mode === 'title' ? 'Window title' : 'Process name'}
                  value={options.window_id_str}
                  onChange={onWinStrChange}
              />
              <AnimatePresence>
                {options.window_id_mode === 'title' && (
                  <motion.div
                      initial={{opacity: 0, width: 0}}
                      animate={{opacity: 1, width: 'auto'}}
                      exit={{opacity: 0, width: 0}}
                      transition={{duration: parseFloat(style.animationDuration)}}
                  >
                    <ButtonGroup selectionMode="single" onSelectionChange={onWinStrModeChange} selectedKeys={new Set([options.match_mode])}>
                      {WINDOWS_MATCH_MODES.map(mode => (
                        <ButtonGroup.Button key={mode} id={mode}>{wordToTitleCase(mode)}</ButtonGroup.Button>
                      ))}
                    </ButtonGroup>
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.label>
            <motion.label {...LABEL_ANIM} >
              <span>Auto-focus</span>
              <input type="checkbox" checked={options.auto_focus} onChange={onAutoFocChange} />
            </motion.label>
          </>
        )}
      </AnimatePresence>
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
