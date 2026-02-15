import style from "../styles/ButtonGroup.module.scss";
import {
  composeRenderProps,
  SelectionIndicator,
  ToggleButton,
  ToggleButtonGroup,
  ToggleButtonGroupProps,
  ToggleButtonProps
} from "react-aria-components";

export function ButtonGroup(props: ToggleButtonGroupProps) {
  return (
    <ToggleButtonGroup orientation="horizontal" className={style.buttonGroup} {...props} />
  );
}

function ButtonGroupItem(props: ToggleButtonProps) {
  const {children, ...oProps} = props;
  return (
    <ToggleButton className={style.buttonGroupItem} {...oProps}>
      {composeRenderProps(children, c => (
        <><SelectionIndicator className={style.selIndicator} data-selected /><span>{c}</span></>
      ))}
    </ToggleButton>
  );
}

ButtonGroup.Button = ButtonGroupItem;
