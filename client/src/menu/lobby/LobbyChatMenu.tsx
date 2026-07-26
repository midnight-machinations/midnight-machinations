import React, { ReactElement } from "react";
import "./lobbyChatMenu.css";
import "../game/headerMenu.css"
import translate from "../../game/lang";
import { ChatMessageSection, ChatTextInput } from "../game/gameScreenContent/ChatMenu";
import { Button } from "../../components/Button";
import Icon from "../../components/Icon";
import { useLobbyState } from "../../components/useHooks";


export default function LobbyChatMenu(props: Readonly<{spectator: boolean}>): ReactElement {
    const [collapsed, setCollapsed] = React.useState(false);

    const [chatNotification, setChatNotification] = React.useState(false);
    
    useLobbyState(
        _ => setChatNotification(collapsed),
        ["addChatMessages"]
    );

    return <section className="lobby-chat-menu chat-menu-colors selector-section">
        <Button
            className="lobby-chat-menu-header"
            onClick={() => {
                setCollapsed(collapsed => !collapsed);
                setChatNotification(false);
            }}
        >
            <h2>
                <Icon size="small">chat</Icon>
                {translate("menu.chat.title")}
                {chatNotification && <div className="chat-notification highlighted">!</div>}
            </h2>
            <Icon>{collapsed ? "keyboard_arrow_up" : "keyboard_arrow_down"}</Icon>
        </Button>
        <div className="lobby-menu-chat" hidden={collapsed}>
            <ChatMessageSection/>
        </div>
        <ChatTextInput disabled={props.spectator} hidden={collapsed}/>
    </section>
}