import React, { ReactElement, useEffect, useMemo, useRef, useState } from "react";
import StyledText from "./StyledText";
import { encodeString } from "./ChatMessage";
import GAME_MANAGER, { replaceMentions } from "..";
import { Button } from "./Button";
import Icon from "./Icon";
import translate from "../game/lang";
import "./textAreaDropdown.css";
import DetailsSummary from "./DetailsSummary";
import { useLobbyOrGameState, usePlayerNames, usePlayerState } from "./useHooks";
import { PlayerIndex, UnsafeString } from "../game/gameState.d";
import PlayerOptionDropdown from "./PlayerOptionDropdown";
import ListMap from "../ListMap";

export function TextDropdownArea(props: Readonly<{
    titleString: UnsafeString,
    savedText: UnsafeString,
    defaultOpen?: boolean,
    open?: boolean,
    dropdownArrow?: boolean,
    onAdd?: () => void,
    onSubtract?: () => void,
    onSave: (text: string) => void,
    cantPost: boolean,

    canPostAs?: PlayerIndex[]
}>): ReactElement {
    const [field, setField] = useState<UnsafeString>(props.savedText);

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

    function send(field: UnsafeString){
        save(field);
        GAME_MANAGER.sendSendChatMessagePacket(field as string, true, postingAsPlayer??undefined);
    }

    function save(field: UnsafeString) {
        props.onSave(field as string);
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
        titleString: UnsafeString,
        savedText: UnsafeString,
        field: UnsafeString,
        open?: boolean,
        onAdd?: () => void,
        onSubtract?: () => void,
        onSave: (text: UnsafeString) => void,
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

    const roleList = useLobbyOrGameState(
        gameState => gameState.roleList,
        ["roleList"]
    ) ?? [];

    const modifierSettings = useLobbyOrGameState(
        gameState => gameState.modifierSettings,
        ["modifierSettings"]
    ) ?? new ListMap();

    function save(field: UnsafeString) {
        props.onSave(field);
    }

    function send(field: UnsafeString){
        save(field);
        props.onSend();
    }

    return <div>
        <StyledText>{encodeString(replaceMentions(props.titleString, playerNames, roleList, modifierSettings))}</StyledText>
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
    field: UnsafeString,
    setField: (field: UnsafeString) => void,
    save: (field: UnsafeString) => void,
    send: (field: UnsafeString) => void,
}>): ReactElement {
    const [writing, setWriting] = useState<boolean>(false);
    const [hover, setHover] = useState<boolean>(false);
    const playerNames = usePlayerNames();

    const roleList = useLobbyOrGameState(
        gameState => gameState.roleList,
        ["roleList"]
    ) ?? [];

    const modifierSettings = useLobbyOrGameState(
        gameState => gameState.modifierSettings,
        ["modifierSettings"]
    ) ?? new ListMap();

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
                    {encodeString(replaceMentions(props.field, playerNames, roleList, modifierSettings))}
                </StyledText>
            </div>
            :
            <textarea
                className="textarea"
                ref={textareaRef}
                value={props.field as string}
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