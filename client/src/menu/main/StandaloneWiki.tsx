import React, { ReactElement } from "react";
import Wiki from "../../components/Wiki";
import translate from "../../game/lang";
import "./standaloneWiki.css";
import { WikiArticleLink } from "../../components/WikiArticleLink";
import { MODIFIERS, ModifierID } from "../../game/gameState.d";
import { getAllRoles } from "../../game/roleListState.d";

export default function StandaloneWiki(props: Readonly<{
    initialWikiPage?: WikiArticleLink
}>): ReactElement {
    return <div className="standalone-wiki wiki-menu-colors">
        <header>
            <h2>{translate("menu.wiki.title")}</h2>
        </header>
        <div>
            <Wiki 
                enabledRoles={getAllRoles()}
                enabledModifiers={MODIFIERS as any as ModifierID[]}
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