import React, { ReactElement, useContext } from "react";
import translate from "../../game/lang";
import Icon from "../../components/Icon";
import { Button } from "../../components/Button";
import { WebsocketContext } from "../WebsocketContext";
import { StateContext } from "../../stateContext/StateContext";



export default function LobbyNamePane(): ReactElement {
    const lobbyState = useContext(StateContext)!;
    const client = lobbyState.clients.get(lobbyState.myId!);
    const isSpectator = client?.clientType.type === "spectator";
    const ready = client?.ready??false;
    const websocketContext = useContext(WebsocketContext)!;

    let myName = "";
    if(client?.clientType.type === "player"){
        myName = client.clientType.name
    }

    return <section className="player-list-menu-colors selector-section lobby-name-pane">
        {!isSpectator && <NameSelector name={myName}/>}
        <div className="name-pane-buttons">
            <Button onClick={() => websocketContext.sendSetSpectatorPacket(!isSpectator)}>
                {isSpectator
                    ? <><Icon>sports_esports</Icon> {translate("switchToPlayer")}</>
                    : <><Icon>visibility</Icon> {translate("switchToSpectator")}</>}
            </Button>
            {ready === "host" && <button
                onClick={() => websocketContext.sendRelinquishHostPacket()}
            ><Icon>remove_moderator</Icon> {translate("menu.lobby.button.relinquishHost")}</button>}
            {ready !== "host" && <Button
                onClick={() => {websocketContext.sendReadyUpPacket(ready === "notReady")}}
            >
                {ready === "ready"
                    ? <><Icon>clear</Icon> {translate("menu.lobby.button.unready")}</>
                    : <><Icon>check</Icon> {translate("menu.lobby.button.readyUp")}</>}
            </Button>}
        </div>
    </section>
}

function NameSelector(props: {name: String}): ReactElement {
    
    const [enteredName, setEnteredName] = React.useState("");
    const websocketContext = useContext(WebsocketContext)!;

    return <div className="name-pane-selector">
        <div className="lobby-name">
            <section><h2>{props.name ?? ""}</h2></section>
        </div>
        <div className="name-box">
            <input type="text" value={enteredName}
                onChange={(e)=>{setEnteredName(e.target.value)}}
                placeholder={translate("menu.lobby.field.namePlaceholder")}
                onKeyUp={(e)=>{
                    if(e.key === 'Enter')
                        websocketContext.sendSetNamePacket(enteredName);
                }}
            />
            <button onClick={()=>{
                websocketContext.sendSetNamePacket(enteredName)
            }}>{translate("menu.lobby.button.setName")}</button>
        </div>
    </div>
}