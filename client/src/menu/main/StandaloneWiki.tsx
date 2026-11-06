import type { ReactElement } from "react";
import Wiki from "../../components/Wiki";
import translate from "../../game/lang";
import "./standaloneWiki.css";
import type { WikiArticleLink } from "../../components/WikiArticleLink";
import { ALL_MODIFIERS } from "../../game/modifiers";
import { getAllRoles } from "../../game/roleListState.d";

export default function StandaloneWiki(
    props: Readonly<{
        initialWikiPage?: WikiArticleLink;
    }>,
): ReactElement {
    return (
        <div className="standalone-wiki wiki-menu-colors">
            <header>
                <h2>{translate("menu.wiki.title")}</h2>
            </header>
            <div>
                <Wiki
                    enabledRoles={getAllRoles()}
                    enabledModifiers={ALL_MODIFIERS}
                    initialWikiPage={props.initialWikiPage}
                    onPageChange={(page) => {
                        if (page !== null) {
                            window.history.replaceState({}, "", `/wiki/${page}`);
                        } else {
                            window.history.replaceState({}, "", `/wiki`);
                        }
                    }}
                    static={true}
                />
            </div>
        </div>
    );
}
