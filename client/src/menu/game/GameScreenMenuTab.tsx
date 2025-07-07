import React, { ReactElement, useContext } from "react";
import { Button } from "../../components/Button";
import Icon from "../../components/Icon";
import StyledText from "../../components/StyledText";
import { GameScreenMenuContext, GameScreenMenuType } from "./GameScreenMenuContext";
import { WikiArticleLink } from "../../wiki/WikiArticleLink";
import { MobileContext } from "../MobileContext";
import { AppContext } from "../AppContext";
import { StateContext } from "../../stateContext/StateContext";

export default function GameScreenMenuTab(props: Readonly<{
    helpMenu: WikiArticleLink | null
    close: GameScreenMenuType | false, 
    children: string 
}>): ReactElement {
    const appContext = useContext(AppContext)!;
    const menuController = useContext(GameScreenMenuContext)!;
    const spectator = useContext(StateContext)!.clientState.type === "spectator";
    const mobile = useContext(MobileContext)!;

    return <div className="content-tab">
        <div>
            <StyledText>
                {props.children}
            </StyledText>
        </div>

        {props.close && (!spectator || mobile) && <Button className="close"
            onClick={()=>menuController.closeMenu(props.close as GameScreenMenuType)}
        >
            <Icon size="small">close</Icon>
        </Button>}
        {props.helpMenu && !spectator && <Button className="help"
            onClick={()=>{
                menuController.openMenu(GameScreenMenuType.WikiMenu, ()=>{
                    props.helpMenu && appContext.setWikiArticle(props.helpMenu);
                });
            }}
        >
            <Icon size="small">question_mark</Icon>
        </Button>}
    </div>
}