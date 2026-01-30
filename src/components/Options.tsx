import style from '../styles/Options.module.scss';
import {ImmerHook} from 'use-immer';
import {MacroOptions, OptionsHyprland} from '../lib/data_types.ts';
import {ChangeEvent, JSX} from 'react';

export function OptionsSection({ optionsImmer }: { optionsImmer: ImmerHook<MacroOptions> }) {
  let child: JSX.Element;
  if (optionsImmer[0].type === 'hyprland') {
    child = <HyprlandOptions optionsImmer={optionsImmer} />
  } else {
    throw new Error(`Unknown options type "${optionsImmer[0].type}"`);
  }
  return (
    // TODO animate height
    <div className={[style.optsNotice, style.optionsSection].join(' ')}>{child}</div>
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
    })
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
    <span>Unfortunately your system is not unsupported</span>
  );
}

