import React, { ReactElement, useEffect, useMemo, useRef, useState } from "react";
import StyledText from "./StyledText";
import { sanitizePlayerMessage } from "./ChatMessage";
import GAME_MANAGER, { replaceMentions } from "..";
import { Button } from "./Button";
import Icon from "./Icon";
import translate from "../game/lang";
import "./textAreaDropdown.css";
import DetailsSummary from "./DetailsSummary";
import { usePlayerNames, usePlayerState } from "./useHooks";
import { PlayerIndex } from "../game/gameState.d";
import PlayerOptionDropdown from "./PlayerOptionDropdown";

export function TextDropdownArea(props: Readonly<{
    titleString: string,
    savedText: string,
    defaultOpen?: boolean,
    open?: boolean,
    dropdownArrow?: boolean,
    onAdd?: () => void,
    onSubtract?: () => void,
    onSave: (text: string) => void,
    cantPost: boolean,

    canPostAs?: PlayerIndex[]
}>): ReactElement {
    const [field, setField] = useState<string>(props.savedText);
    
    const myIndex = usePlayerState((p, _)=>p.myIndex);

    useEffect(() => {
        setField(props.savedText)
    }, [props.savedText]);
    

    const unsaved = useMemo(() => {
        if(field==="bruh"){
            console.log(props.savedText);
            console.log(field);
            console.log(props.savedText !== field);
        }
        return props.savedText !== field
    }, [field, props.savedText]);

    let canPostAs =(
        props.canPostAs===undefined ||
        (myIndex !== undefined && props.canPostAs.length === 1 && props.canPostAs.includes(myIndex))
    )?(
        undefined
    ):props.canPostAs;

    const [postingAsPlayer, setPostingAsPlayer] = useState<PlayerIndex|null>(
        canPostAs!==undefined?canPostAs[0]:null
    );

    useEffect(()=>{
        if(canPostAs === undefined){
            setPostingAsPlayer(null);
        }else if(postingAsPlayer === null || !canPostAs.includes(postingAsPlayer)){
            setPostingAsPlayer(canPostAs[0]);
        }
    }, [canPostAs, postingAsPlayer]);

    function send(field: string){
        save(field);
        GAME_MANAGER.sendSendChatMessagePacket(field, true, postingAsPlayer??undefined);
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
                onSend={()=>send(field)}
                cantPost={props.cantPost}

                postingAs={postingAsPlayer??undefined}
                canPostAs={canPostAs}
                onSetPostingAs={(p)=>setPostingAsPlayer(p)}
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
        onSend: ()=>void,
        cantPost: boolean,

        onSetPostingAs?: (player: PlayerIndex | null) => void,
        canPostAs?: PlayerIndex[],
        postingAs?: PlayerIndex
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
        props.onSend();
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
            {
                (props.canPostAs!==undefined)&&(props.canPostAs.length > 0)?
                <PlayerOptionDropdown
                    value={props.postingAs??props.canPostAs[0]}
                    onChange={(p)=>{
                        if(props.onSetPostingAs!==undefined){
                            props.onSetPostingAs(p)
                        }
                    }}
                    choosablePlayers={props.canPostAs}
                    canChooseNone={false}
                />:null
            }
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