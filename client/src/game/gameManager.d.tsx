import { WikiArticleLink } from "../components/WikiArticleLink";
import { DoomsayerGuess } from "../menu/game/gameScreenContent/AbilityMenu/RoleSpecificMenus/LargeDoomsayerMenu";
import { ControllerInput } from "./controllerInput";
import { PhaseType, PhaseTimes, PlayerIndex, State, Verdict, ModifierType, FastForwardSetting } from "./gameState.d";
import { ToClientPacket, ToServerPacket } from "./packet";
import { RoleList, RoleOutline } from "./roleListState.d";
import { Role } from "./roleState.d";

export type Server = {
    ws: WebSocket | null,

    open(): Promise<boolean>;
    sendPacket(packets: ToServerPacket): void;
    close(): void;
}

export type StateEventType = ToClientPacket["type"] | "tick" | "filterUpdate" | "openGameMenu" | "closeGameMenu" | "whisperChatOpenOrClose" | "connectionClosed";
export type StateListener = (type?: StateEventType) => void;

export type GameManager = {

    setDisconnectedState(): Promise<void>;
    setLobbyState(): void;
    setGameState(): void;
    setSpectatorGameState(): void;
    setOutsideLobbyState(): Promise<boolean>;
    

    state: State,
    updateChatFilter(filter: PlayerIndex | null): void,

    server: Server,
    listeners: StateListener[],

    addStateListener(listener: StateListener): void;
    removeStateListener(listener: StateListener): void;
    invokeStateListeners(type?: StateEventType): void;

    setPrependWhisperFunction: (f: ((index: PlayerIndex) => void)) => void;
    prependWhisper: (index: PlayerIndex) => void;

    wikiArticleCallbacks: ((article: WikiArticleLink | null) => void)[];
    addSetWikiArticleCallback: (callback: ((article: WikiArticleLink | null) => void)) => void;
    removeSetWikiArticleCallback: (callback: ((article: WikiArticleLink | null) => void)) => void;
    setWikiArticle: (article: WikiArticleLink | null) => void;

    leaveGame(): void;

    sendLobbyListRequest(): void;
    /**
     * @returns A promise that will be fulfilled as true if the join was 
     *          successful and false if the join was unsuccessful
     */
    sendHostPacket(): Promise<boolean>;
    /**
     * @returns A promise that will be fulfilled as true if the join was 
     *          successful and false if the join was unsuccessful
     */
    sendRejoinPacket(roomCode: number, playerId: number): Promise<boolean>;
    /**
     * @returns A promise that will be fulfilled as true if the join was 
     *          successful and false if the join was unsuccessful
     */
    sendJoinPacket(roomCode: number): Promise<boolean>;
    sendKickPlayerPacket(playerId: number): void;
    sendSetPlayerHostPacket(playerId: number): void;
    sendRelinquishHostPacket(): void;
    sendSetSpectatorPacket(spectator: boolean): void;
    sendSetNamePacket(name: string): void;
    sendReadyUpPacket(ready: boolean): void;
    sendSendLobbyMessagePacket(text: string): void;
    sendSetLobbyNamePacket(name: string): void;
    sendStartGamePacket(): Promise<boolean>;
    sendBackToLobbyPacket(): void;
    sendSetPhaseTimePacket(phase: PhaseType, time: number): void;
    sendSetPhaseTimesPacket(phaseTimeSettings: PhaseTimes): void;
    sendSetRoleListPacket(roleListEntries: RoleList): void;
    sendSetRoleOutlinePacket(index: number, roleOutline: RoleOutline): void;
    sendSimplifyRoleListPacket(): void;
    
    sendJudgementPacket(judgement: Verdict): void;
    sendSaveWillPacket(will: string): void;
    sendSaveNotesPacket(notes: string[]): void;
    sendSaveCrossedOutOutlinesPacket(crossedOutOutlines: number[]): void;
    sendSaveDeathNotePacket(notes: string): void;
    sendSendChatMessagePacket(text: string, block: boolean, controllingPlayer?: PlayerIndex): void;
    sendSendWhisperPacket(playerIndex: number, text: string, controllingPlayer?: PlayerIndex): void;
    sendEnabledRolesPacket(roles: Role[]): void;
    sendEnabledModifiersPacket(modifiers: ModifierType[]): void;

    sendControllerInput(input: ControllerInput): void;
    sendSetDoomsayerGuess(guesses: [
        [number, DoomsayerGuess],
        [number, DoomsayerGuess],
        [number, DoomsayerGuess]
    ]): void;
    sendSetConsortOptions(
        roleblock: boolean, 
        youWereRoleblockedMessage: boolean, 
        youSurvivedAttackMessage: boolean, 
        youWereGuardedMessage: boolean, 
        youWereTransportedMessage: boolean, 
        youWerePossessedMessage: boolean, 
        yourTargetWasJailedMessage: boolean
    ): void

    sendVoteFastForwardPhase(fastForward: FastForwardSetting): void;
    sendHostDataRequest(): void;
    sendHostEndGamePacket(): void;
    sendHostSkipPhase(): void;
    sendHostSetPlayerNamePacket(player_id: number, name: string): void;

    messageListener(serverMessage: ToClientPacket): void;

    tick(timePassedMs: number): void;

}

export declare function createGameManager(): GameManager;