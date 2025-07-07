import React, { ReactElement, useContext, useMemo } from "react";
import translate from "../../../game/lang";
import { TextDropdownArea } from "../../../components/TextAreaDropdown";
import { GameScreenMenuType } from "../GameScreenMenuContext";
import GameScreenMenuTab from "../GameScreenMenuTab";
import { WebsocketContext } from "../../WebsocketContext";
import { usePlayerState } from "../../../stateContext/useHooks";
import { getSingleRoleJsonData } from "../../../stateContext/stateType/roleState";
import { StateContext } from "../../../stateContext/StateContext";

export function defaultAlibi(): string {
    return DEFAULT_ALIBI;
}
const DEFAULT_ALIBI = "ROLE\nNight 1: \nNight 2:";

export default function WillMenu(): ReactElement {
    const cantChat = usePlayerState()!.sendChatGroups.length === 0;

    const role = usePlayerState()!.roleState.type;

    const alibi = usePlayerState()!.will;
    const notes = usePlayerState()!.notes;
    const deathNote = usePlayerState()!.deathNote;

    const gameState = useContext(StateContext)!;
    const websocketContext = useContext(WebsocketContext)!;

    const cantPost = useMemo(() => {
        return cantChat
    }, [cantChat])
    
    return <div className="will-menu will-menu-colors">
        <GameScreenMenuTab
            close={GameScreenMenuType.WillMenu}
            helpMenu={"standard/alibi"}
        >
                {translate("menu.will.title")}
        </GameScreenMenuTab>
        <section>
            <TextDropdownArea
                titleString={translate("menu.will.will")}
                defaultOpen={true}
                savedText={alibi}
                cantPost={cantPost}
                onSave={(text) => {
                    websocketContext.sendSaveWillPacket(text);
                }}
            />
            {(notes.length === 0 ? [""] : notes).map((note: string, i: number) => {
                const title = note.split('\n')[0] || translate("menu.will.notes");
                return <TextDropdownArea
                    key={title + i}
                    titleString={title}
                    savedText={note}
                    cantPost={cantPost}
                    onAdd={() => {
                        if(gameState.type === "game" && gameState.clientState.type === "player"){
                            const notes = [...gameState.clientState.notes];
                            notes.splice(i+1, 0, "");
                            websocketContext.sendSaveNotesPacket(notes);
                        }
                    }}
                    onSubtract={() => {
                        if(gameState.type === "game" && gameState.clientState.type === "player"){
                            const notes = [...gameState.clientState.notes];
                            notes.splice(i, 1);
                            websocketContext.sendSaveNotesPacket(notes);
                        }
                    }}
                    onSave={(text) => {
                        if(gameState.type === "game" && gameState.clientState.type === "player"){
                            const notes = [...gameState.clientState.notes];
                            notes[i] = text;
                            websocketContext.sendSaveNotesPacket(notes);
                        }
                    }}
                />
            })}
            {getSingleRoleJsonData(role).canWriteDeathNote===true ? <TextDropdownArea
                titleString={translate("menu.will.deathNote")}
                savedText={deathNote}
                cantPost={cantPost}
                onSave={(text) => {
                    websocketContext.sendSaveDeathNotePacket(text);
                }}
            />:null}
        </section>
    </div>
}

