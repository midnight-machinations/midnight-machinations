import React, { ReactElement, useEffect, useMemo } from "react";
import translate from "../../../game/lang";
import GAME_MANAGER from "../../../index";
import { usePlayerState } from "../../../components/useHooks";
import { getSingleRoleJsonData } from "../../../game/roleState.d";
import { TextDropdownArea } from "../../../components/TextAreaDropdown";
import ListMap from "../../../ListMap";
import { controllerIdToLinkWithPlayer } from "../../../game/controllerInput";
import { PlayerIndex, UnsafeString } from "../../../game/gameState.d";
import { ContentTab } from "../GameScreen";

export function defaultAlibi(): string {
    return DEFAULT_ALIBI;
}
const DEFAULT_ALIBI = "ROLE\nNight 1: \nNight 2:";

export default function WillMenu(): ReactElement {
    const playerIndex = usePlayerState(
        playerState => playerState.myIndex
    )!;

    const cantChat = usePlayerState(
        playerState => playerState.sendChatGroups.length === 0,
        ["yourSendChatGroups"]
    )!;

    const role = usePlayerState(
        playerState => playerState.roleState.type,
        ["yourRoleState"]
    )!;

    const savedAbilities = usePlayerState(
        playerState => playerState.savedControllers,
        ["yourAllowedControllers", "yourAllowedController"]
    )!;
    const alibiSelection = new ListMap(savedAbilities, (k1, k2)=>controllerIdToLinkWithPlayer(k1)===controllerIdToLinkWithPlayer(k2)).get({type: "alibi", player: playerIndex});
    const alibi = (alibiSelection?.selection.type === "string")?alibiSelection.selection.selection:"";
    useEffect(()=>{
        if(alibi===""){
            GAME_MANAGER.sendSaveWillPacket("");
        }
    }, [alibi])

    const notes = usePlayerState(
        playerState => playerState.notes,
        ["yourNotes"]
    )!;
    const deathNote = usePlayerState(
        playerState => playerState.deathNote,
        ["yourDeathNote"]
    )!;

    const cantPost = useMemo(() => {
        return cantChat
    }, [cantChat]);


    const canPostAsPlayers: PlayerIndex[] = savedAbilities
        .map(([id,_])=>id.type==="chat"?id.player:undefined)
        .filter((p)=>p!==undefined?true:false) as PlayerIndex[];
    
    return <div className="will-menu will-menu-colors">
        <ContentTab
            close={"WillMenu"}
        >
                {translate("menu.will.title")}
        </ContentTab>
        <section>
            <TextDropdownArea
                titleString={translate("menu.will.will")}
                defaultOpen={true}
                savedText={alibi}
                cantPost={cantPost}
                onSave={(text) => {
                    GAME_MANAGER.sendSaveWillPacket(text);
                }}
            />
            {(notes.length === 0 ? [""] : notes).map((note, i) => {
                const title: UnsafeString = (note as string).split('\n')[0] || translate("menu.will.notes");
                return <TextDropdownArea
                    canPostAs={canPostAsPlayers}

                    key={title as string + i}
                    titleString={title}
                    savedText={note}
                    cantPost={cantPost}
                    onAdd={() => {
                        if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                            const notes = [...GAME_MANAGER.state.clientState.notes];
                            notes.splice(i+1, 0, "");
                            GAME_MANAGER.sendSaveNotesPacket(notes as string[]);
                        }
                    }}
                    onSubtract={() => {
                        if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                            const notes = [...GAME_MANAGER.state.clientState.notes];
                            notes.splice(i, 1);
                            GAME_MANAGER.sendSaveNotesPacket(notes as string[]);
                        }
                    }}
                    onSave={(text) => {
                        if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                            const notes = [...GAME_MANAGER.state.clientState.notes];
                            notes[i] = text;
                            GAME_MANAGER.sendSaveNotesPacket(notes as string[]);
                        }
                    }}
                />
            })}
            {getSingleRoleJsonData(role).canWriteDeathNote===true ? <TextDropdownArea
                titleString={translate("menu.will.deathNote")}
                savedText={deathNote}
                cantPost={cantPost}
                onSave={(text) => {
                    GAME_MANAGER.sendSaveDeathNotePacket(text);
                }}
            />:null}
        </section>
    </div>
}

