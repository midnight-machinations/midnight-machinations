import { useEffect, useRef, useState } from "react";
import Icon from "./Icon";
import "./flushInput.css"

export default function FlushInput(props: Readonly<{
    value: string;
    setValue: (value: string) => void;
    onConfirm: (value: string) => void;
    className?: string;
    onlyEnterToConfirm?: boolean;
    dontHidePencil?: boolean;
}>) {
    const inputRef = useRef<HTMLInputElement | null>(null);

    const calculateInputFieldWidth = () => {
        if (inputRef.current === null) return 50;

        const style = globalThis.getComputedStyle(inputRef.current);

        // Measure text size using temporary span element
        const temp = document.createElement("span");
        temp.style.fontSize = style.fontSize;
        temp.style.fontFamily = style.fontFamily;
        temp.style.fontWeight = style.fontWeight;
        temp.style.whiteSpace = "pre";  // Don't trim whitespace
        temp.textContent = props.value;
        document.body.appendChild(temp);
        const inputWidth = temp.getBoundingClientRect().width;
        temp.remove();
        return inputWidth;
    };

    const [inputFieldWidth, setInputFieldWidth] = useState(calculateInputFieldWidth());

    useEffect(() => {
        setInputFieldWidth(calculateInputFieldWidth());
    }, [props.value]);
    
    const [inputFocused, setInputFocused] = useState(false);

    return <div className="flush-input-container">
        <input className={`flush-input ${props.className ?? ''}`} type="text" value={props.value}
            onChange={(e)=>{props.setValue(e.target.value)}}
            onKeyUp={(e)=>{
                if(e.key === 'Enter')
                    props.onConfirm(props.value);
            }}
            onFocus={e => setInputFocused(true)}
            onBlur={e => {
                const newName = e.target.value;
                props.setValue(newName);
                if (!props.onlyEnterToConfirm) {
                    props.onConfirm(newName);
                }
                setInputFocused(false);
            }}
            ref={(el) => {
                inputRef.current = el;
                setInputFieldWidth(calculateInputFieldWidth());
            }}
            style={{ width: `${inputFieldWidth}px` }}
        />
        {(props.dontHidePencil || !inputFocused) && <button
            className="flush-input-button"
            onClick={() => {
                inputRef.current?.focus();
                inputRef.current?.select();
            }}
        >
            <Icon>edit</Icon>
        </button>}
    </div>
}