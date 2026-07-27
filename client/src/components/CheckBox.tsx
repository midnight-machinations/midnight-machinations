import Icon from "./Icon"
import "./checkBox.css"

export default function CheckBox(props: {
    checked: boolean,
    onChange: (checked: boolean) => void
}) {
    return <button
        className="checkbox"
        onClick={()=>{
            props.onChange(!props.checked)
        }}
    >
        <Icon>{props.checked ? "check" : "close"}</Icon>
    </button>
}