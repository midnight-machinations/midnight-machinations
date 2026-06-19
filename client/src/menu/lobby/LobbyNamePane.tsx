import React, { ReactElement, useEffect, useRef, useState } from "react";
import GAME_MANAGER from "../..";
import translate from "../../game/lang";
import Icon from "../../components/Icon";
import { useLobbyState } from "../../components/useHooks";
import { Button } from "../../components/Button";
import { UnsafeString } from "../../game/gameState.d";



export default function LobbyNamePane(): ReactElement {
    const isSpectator = useLobbyState(
        lobbyState => lobbyState.players.get(lobbyState.myId!)?.clientType.type === "spectator",
        ["lobbyClients", "yourId"]
    )!;

    const ready = useLobbyState(
        lobbyState => lobbyState.players.get(lobbyState.myId!)?.ready,
        ["lobbyClients", "playersHost", "playersReady", "yourId"]
    )!;

    const otherPlayersReady = useLobbyState(
        lobbyState => lobbyState.players.values().map(p => p.ready),
        ["lobbyClients", "playersHost", "playersReady"]
    )!;

    // This is an integer so that multiple flashes can overlap
    const [flashingSince, setFlashingSince] = useState(0);
    const [now, setNow] = useState(Date.now());

    useEffect(() => {
        const interval = setInterval(() => setNow(Date.now()), 100);
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        if (ready === "notReady" && !isSpectator) {
            setFlashingSince(Date.now());
        }
    }, [otherPlayersReady])

    return <section className="player-list-menu-colors selector-section">
        <div className="lobby-name-pane">
            {!isSpectator && <NameSelector ready={ready}/>}
            <div className="name-pane-buttons">
                {ready !== "ready" && <Button onClick={() => GAME_MANAGER.sendSetSpectatorPacket(!isSpectator)}>
                    {isSpectator
                        ? <><Icon>sports_esports</Icon> {translate("switchToPlayer")}</>
                        : <><Icon>visibility</Icon> {translate("switchToSpectator")}</>}
                </Button>}
                {ready === "host" && <button
                    onClick={() => GAME_MANAGER.sendRelinquishHostPacket()}
                ><Icon>remove_moderator</Icon> {translate("menu.lobby.button.relinquishHost")}</button>}
                {ready !== "host" && <Button
                    className={(ready !== "ready" ? "ready-up-button " : "") + (ready === "ready" ? "depressed " : ((now < flashingSince + 2000) ? "flashing" : undefined))}
                    onClick={() => {GAME_MANAGER.sendReadyUpPacket(ready === "notReady")}}
                >
                    {ready === "ready"
                        ? <><Icon>clear</Icon> {translate("menu.lobby.button.unready")}</>
                        : <><Icon>check</Icon> {translate("menu.lobby.button.readyUp")}</>}
                </Button>}
            </div>
        </div>
    </section>
}

function NameSelector(props: Readonly<{ ready: "host" | "ready" | "notReady" }>): ReactElement {
    const myName = useLobbyState(
        state => {
            const client = state.players.get(state.myId!);
            if(client === undefined || client === null) return undefined;
            if(client.clientType.type === "spectator") return undefined;
            return client.clientType.name;
        },
        ["lobbyClients", "yourId"]
    )!;
    const [enteredName, setEnteredName] = React.useState<UnsafeString>(myName);

    useEffect(() => {
        setEnteredName(myName);
    }, [myName]);

    const nameInputRef = useRef<HTMLInputElement | null>(null);

    const calculateInputFieldWidth = () => {
        if (nameInputRef.current === null) return 50;

        const style = globalThis.getComputedStyle(nameInputRef.current);

        // Measure text size using temporary span element
        const temp = document.createElement("span");
        temp.style.fontSize = style.fontSize;
        temp.style.fontFamily = style.fontFamily;
        temp.style.fontWeight = style.fontWeight;
        temp.style.whiteSpace = "pre";  // Don't trim whitespace
        temp.textContent = enteredName as string;
        document.body.appendChild(temp);
        const inputWidth = temp.getBoundingClientRect().width;
        temp.remove();
        return inputWidth;
    };

    const [inputFieldWidth, setInputFieldWidth] = useState(calculateInputFieldWidth());

    useEffect(() => {
        setInputFieldWidth(calculateInputFieldWidth());
    }, [enteredName]);
    
    const [inputFocused, setInputFocused] = useState(false);

    return <div className="name-pane-selector">
        <div className="lobby-name">
            {props.ready !== "ready" && <>
                <input type="text" value={enteredName as string}
                    onChange={(e)=>{setEnteredName(e.target.value)}}
                    placeholder={translate("menu.lobby.field.namePlaceholder")}
                    onKeyUp={(e)=>{
                        if(e.key === 'Enter')
                            GAME_MANAGER.sendSetNamePacket(enteredName as string);
                    }}
                    onFocus={() => setInputFocused(true)}
                    onBlur={e => {
                        const newName = e.target.value;
                        setEnteredName(newName);
                        GAME_MANAGER.sendSetNamePacket(newName);
                        setInputFocused(false);
                    }}
                    ref={(el) => {
                        nameInputRef.current = el;
                        setInputFieldWidth(calculateInputFieldWidth());
                    }}
                    style={{ width: `${inputFieldWidth}px` }}
                />
                {!inputFocused && <button
                    onClick={() => {
                        nameInputRef.current?.focus();
                    }}
                >
                    <Icon size="tiny">edit</Icon>
                </button>}
            </>}
            {props.ready === "ready" && <h2>{enteredName as string}</h2>}
        </div>
    </div>
}