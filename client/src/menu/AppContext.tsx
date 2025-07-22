import { createContext, ReactElement, useMemo, useState } from "react";
import { ErrorCard, ErrorData } from "./App";
import { Theme } from "..";
import React from "react";
import WikiCoverCard from "../wiki/WikiCoverCard";
import WikiArticle from "../wiki/WikiArticle";
import StandaloneWiki from "./main/StandaloneWiki";
import { WikiArticleLink } from "../wiki/WikiArticleLink";
import StartMenu from "./main/StartMenu";
import Credits from "./main/Credits";
import LoadingScreen, { LoadingScreenType } from "./LoadingScreen";
import PlayMenu from "./main/PlayMenu";
import GameScreen from "./game/GameScreen";
import LobbyMenu from "./lobby/LobbyMenu";

type AppContentType = {
    type: "main"
} | {
    type: "manual",
    article?: WikiArticleLink
} | {
    type:"404",
    path: string
} | {
    type:"credits",
} | {
    type:"gameBrowser"
} | {
    type:"gameScreen",
    spectator: boolean
} | {
    type:"lobbyScreen",
} | {
    type:"loading"
    loadingType?: LoadingScreenType,
}

export type AppContextType = {
    setContent: (content: AppContentType) => void,
    
    contentType: AppContentType,
    content: JSX.Element | null,

    getCoverCard: () => JSX.Element | null,
    setCoverCard: (content: JSX.Element) => void,
    clearCoverCard: () => void,
    coverCard: JSX.Element | null,
    coverCardTheme: Theme | null;

    pushErrorCard: (error: ErrorData) => void,
    errorCard: JSX.Element | null,

    openGlobalMenu: () => void,
    closeGlobalMenu: () => void,
    globalMenuOpen: boolean,

    setFontSize: (fontSize: number) => void,
    setAccessibilityFontEnabled: (accessibilityFontEnabled: boolean) => void,

    setWikiArticle: (article: WikiArticleLink) => void,
}

export const AppContext = createContext<AppContextType | undefined>(undefined);

export default function AppContextProvider(props: Readonly<{ children: React.ReactNode }>): ReactElement {
    const [content, setContent] = useState<JSX.Element>(<StartMenu/>);
    const [contentType, setContentType] = useState<AppContentType>({type: "main"});

    const [coverCard, setCoverCard] = useState<JSX.Element | null>(null);
    const [coverCardTheme, setCoverCardTheme] = useState<Theme | null>(null);

    const [errorCard, setErrorCard] = useState<JSX.Element | null>(null);

    const [globalMenuOpen, setGlobalMenuOpen] = useState<boolean>(false);

    const appContext: AppContextType = useMemo(() => ({
        content,
        contentType,
        coverCard,
        coverCardTheme,
        globalMenuOpen,
        errorCard,

        setContent: (contentType)=>{
            switch(contentType.type){
                case "main":
                    setContent(<StartMenu/>);
                break;
                case "manual":
                    if(contentType.article !== undefined){
                        setContent(<StandaloneWiki initialWikiPage={contentType.article}/>);
                    }else{
                        setContent(<StandaloneWiki/>);
                    }
                break;
                case "404":
                    setContent(<div className="hero" style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: "1rem" }}>
                        <h1>404</h1>
                        <p>The requested path ({contentType.path}) could not be found</p>
                    </div>);
                break;
                case "credits":
                    setContent(<Credits/>);
                break;
                case "gameBrowser":
                    setContent(<PlayMenu/>);
                break;
                case "gameScreen":
                    setContent(<GameScreen isSpectator={contentType.spectator}/>);
                break;
                case "lobbyScreen":
                    setContent(<LobbyMenu/>);
                break;
                case "loading":
                    setContent(<LoadingScreen type={contentType.loadingType??"default"}/>);
                break;
            }
            setContentType(contentType);
        },
        getCoverCard: () => {
            return coverCard;
        },
        setCoverCard: (coverCard: JSX.Element, callback?: () => void) => {
            let coverCardTheme: Theme | null = null;
            if (coverCard.type === WikiCoverCard || coverCard.type === WikiArticle) {
                coverCardTheme = "wiki-menu-colors";
            }

            setCoverCard(coverCard);
            setCoverCardTheme(coverCardTheme);
        },
        pushErrorCard: (error: ErrorData) => {
            setErrorCard(
                <ErrorCard
                    onClose={() => setErrorCard(null)}
                    error={error}
                />
            );
        },
        clearCoverCard: () => {
            setCoverCard(null);
            setCoverCardTheme(null);
        },
        openGlobalMenu: () => setGlobalMenuOpen(true),
        closeGlobalMenu: () => setGlobalMenuOpen(false),
        setFontSize: (fontSize: number) => {
            document.documentElement.style.fontSize = `${fontSize}em`;
        },
        setAccessibilityFontEnabled: (enabled: boolean) => {
            const getFont = (font: string, enabled: boolean) => enabled === true ? 'game-accessible-font' : font;

            const iconFactor = enabled ? '1.2' : '1';

            document.documentElement.style.setProperty('--game-font', getFont('game-base-font', enabled));
            document.documentElement.style.setProperty('--kira-font', getFont('game-kira-font', enabled));
            document.documentElement.style.setProperty('--spiral-font', getFont('game-spiral-font', enabled));
            document.documentElement.style.setProperty('--title-font', getFont('game-title-font', enabled));
            document.documentElement.style.setProperty('--computer-font', getFont('computer-font', enabled));
            document.documentElement.style.setProperty('--legible-computer-font', getFont('legible-computer-font', enabled));
            document.documentElement.style.setProperty('--icon-factor', iconFactor);
        },
        
        setWikiArticle: (article) => {
            // TODO
        }
    }), [content, contentType, coverCard, coverCardTheme, errorCard, globalMenuOpen]);

    return <AppContext.Provider value={appContext}>
        {props.children}
    </AppContext.Provider>
}