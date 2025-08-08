import React, { ReactElement, useEffect, useMemo, useRef, useState } from "react";
import StyledText from "./StyledText";
import { sanitizePlayerMessage } from "./ChatMessage";
import GAME_MANAGER, { replaceMentions } from "..";
import { Button } from "./Button";
import Icon from "./Icon";
import translate from "../game/lang";
import "./textAreaDropdown.css";
import DetailsSummary from "./DetailsSummary";
import { usePlayerNames } from "./useHooks";

export function TextDropdownArea(props: Readonly<{
    titleString: string,
    savedText: string,
    defaultOpen?: boolean,
    open?: boolean,
    dropdownArrow?: boolean,
    onAdd?: () => void,
    onSubtract?: () => void,
    onSave: (text: string) => void,
    cantPost: boolean
}>): ReactElement {
    const [field, setField] = useState<string>(props.savedText);

    useEffect(() => {
        setField(props.savedText)
    }, [props.savedText])

    const unsaved = useMemo(() => {
        if(field==="bruh"){
            console.log(props.savedText);
            console.log(field);
            console.log(props.savedText !== field);
        }
        return props.savedText !== field
    }, [field, props.savedText]);

    function send(field: string){
        save(field);
        GAME_MANAGER.sendSendChatMessagePacket('\n' + field, true);
    }

    function save(field: string) {
        props.onSave(field);
    }

    return (
        <DetailsSummary
            className="text-area-dropdown"
            dropdownArrow={props.dropdownArrow}
            defaultOpen={props.defaultOpen}
            open={props.open}
            summary={<TextDropdownLabel
                titleString={props.titleString}
                savedText={props.savedText}
                field={field}
                onAdd={props.onAdd}
                onSubtract={props.onSubtract}
                onSave={save}
                cantPost={props.cantPost}
            />}
        >
            {unsaved ? "Unsaved" : ""}
            <PrettyTextArea
                field={field}
                setField={setField}
                save={save}
                send={send}
            />
        </DetailsSummary>
    )
}

function TextDropdownLabel(
    props: Readonly<{
        titleString: string,
        savedText: string,
        field: string,
        open?: boolean,
        onAdd?: () => void,
        onSubtract?: () => void,
        onSave: (text: string) => void,
        cantPost: boolean
    }>
): ReactElement {
    
    const unsaved = useMemo(() => {
        return props.savedText !== props.field
    }, [props.field, props.savedText]);

    const playerNames = usePlayerNames();

    function save(field: string) {
        props.onSave(field);
    }

    function send(field: string){
        save(field);
        GAME_MANAGER.sendSendChatMessagePacket('\n' + field, true);
    }

    return <div>
        <StyledText>{replaceMentions(props.titleString, playerNames)}</StyledText>
        <span>
            {props.onSubtract ? <Button
                onClick={(e) => {
                    if(props.onSubtract)
                        props.onSubtract();
                    stopBubblingUpDomEvent(e);
                }}
                pressedChildren={() => <Icon size="small">done</Icon>}
                aria-label={translate("menu.will.subtract")}
            >
                <Icon size="small">remove</Icon>
            </Button> : null}
            {props.onAdd ? <Button
                onClick={(e) => {
                    if(props.onAdd)
                        props.onAdd();
                    stopBubblingUpDomEvent(e);
                }}
                pressedChildren={() => <Icon size="small">done</Icon>}
                aria-label={translate("menu.will.add")}
            >
                <Icon size="small">add</Icon>
            </Button> : null}
            <Button
                highlighted={unsaved}
                onClick={(e) => {
                    save(props.field);
                    stopBubblingUpDomEvent(e);
                    return true;
                }}
                pressedChildren={() => <Icon size="small">done</Icon>}
                aria-label={translate("menu.will.save")}
            >
                <Icon size="small">save</Icon>
            </Button>
            <Button
                disabled={props.cantPost}
                onClick={(e) => {
                    send(props.field);
                    stopBubblingUpDomEvent(e);
                    return true;
                }}
                pressedChildren={() => <Icon size="small">done</Icon>}
                aria-label={translate("menu.will.post")}
            >
                <Icon size="small">send</Icon>
            </Button>
        </span>
    </div>
}

function PrettyTextArea(props: Readonly<{
    field: string,
    setField: (field: string) => void,
    save: (field: string) => void,
    send: (field: string) => void,
}>): ReactElement {
    const [writing, setWriting] = useState<boolean>(false);
    const [hover, setHover] = useState<boolean>(false);
    const playerNames = usePlayerNames();

    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const prettyTextAreaRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleMouseMove = (e: MouseEvent) => {
            if (prettyTextAreaRef.current) {
                if (prettyTextAreaRef.current.contains(e.target as Node)) {
                    setHover(true);
                } else {
                    setHover(false);
                }
            }
        }
        document.addEventListener("mousemove", handleMouseMove);
        return () => document.removeEventListener("mousemove", handleMouseMove);
    }, []);

    // Function to adjust textarea height
    const adjustHeight = () => {
        if (textareaRef.current) {
            textareaRef.current.style.height = "auto"; // Reset height
            textareaRef.current.style.height = `calc(.25rem + ${textareaRef.current.scrollHeight}px)`; // Adjust to fit content
        }
    };

    // Adjust height when the `props.field` value changes
    useEffect(() => {
        adjustHeight();
    }, [props.field, writing, hover]);

    return <div
        ref={prettyTextAreaRef}
        className="pretty-text-area"
        onTouchEnd={() => setWriting(true)}
        onFocus={() => setWriting(true)}
        onBlur={() => setWriting(false)}
    >
        {(!writing && !hover) ?
            <div
                className="textarea"
            >
                <StyledText noLinks={true}>
                    {sanitizePlayerMessage(replaceMentions(props.field, playerNames))}
                </StyledText>
            </div>
            :
            <textarea
                className="textarea"
                ref={textareaRef}
                value={props.field}
                onChange={e => props.setField(e.target.value)}
                onKeyDown={(e) => {
                    if (e.ctrlKey) {
                        if (e.key === 's') {
                            e.preventDefault();
                            props.save(props.field);
                        } else if (e.key === "Enter") {
                            props.send(props.field);
                        }
                    }
                }}>
            </textarea>
        }
    </div>
}


function stopBubblingUpDomEvent(e: React.MouseEvent) {
    e.stopPropagation();
}