import React, { ReactElement } from "react";
import Wiki from "../../components/Wiki";
import translate from "../../game/lang";
import "./standaloneWiki.css";
import { WikiArticleLink } from "../../components/WikiArticleLink";
import { MODIFIERS, ModifierType } from "../../game/gameState.d";
import { getAllRoles } from "../../game/roleListState.d";
import StyledText from "../../components/StyledText";

export default function StandaloneWiki(props: Readonly<{
    initialWikiPage?: WikiArticleLink
}>): ReactElement {
    return <div className="standalone-wiki wiki-menu-colors">
        <header>
            <h2><StyledText noLinks={true}>{translate("menu.wiki.title")}</StyledText></h2>
        </header>
        <div>
            <Wiki 
                enabledRoles={getAllRoles()}
                enabledModifiers={MODIFIERS as any as ModifierType[]}
                initialWikiPage={props.initialWikiPage}
                onPageChange={page => {
                    if (page !== null) {
                        window.history.replaceState({}, '', `/wiki/${page}`)
                    } else {
                        window.history.replaceState({}, '', `/wiki`)
                    }
                }}
                static={true}
            />
        </div>
    </div>
}