import Icon from "./Icon";
import "./checkBox.css";

export default function CheckBox(props: {
    checked: boolean;
    onChange: (checked: boolean) => void;
    disabled?: boolean;
    ariaLabel?: string;
}) {
    return (
        <button
            type="button"
            className="checkbox"
            aria-pressed={props.checked}
            aria-disabled={props.disabled || undefined}
            aria-label={props.ariaLabel}
            tabIndex={props.disabled ? -1 : 0}
            onClick={() => {
                if (!props.disabled) props.onChange(!props.checked);
            }}
        >
            <Icon>{props.checked ? "check" : "close"}</Icon>
        </button>
    );
}
