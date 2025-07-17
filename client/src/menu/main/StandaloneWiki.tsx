import React, { ReactElement } from "react";
import Wiki from "../../wiki/Wiki";
import translate from "../../game/lang";
import "./standaloneWiki.css";
import { WikiArticleLink } from "../../wiki/WikiArticleLink";
import { getAllRoles } from "../../stateContext/stateType/roleListState";
import { MODIFIERS, ModifierType } from "../../stateContext/stateType/modifiersState";

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
                enabledModifiers={MODIFIERS as any as ModifierType[]}
                initialWikiPage={props.initialWikiPage}
                onPageChange={page => {
                    if (page !== null) {
                        window.history.replaceState({}, '', `/wiki/${page}`)
                    } else {
                        window.history.replaceState({}, '', `/wiki`)
                    }
                }}
            />
        </div>
    </div>
}