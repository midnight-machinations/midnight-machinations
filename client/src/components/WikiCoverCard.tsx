import type { ReactElement } from "react";
import Wiki from "../components/Wiki";
import translate from "../game/lang";
import "./wiki.css";
import { ALL_MODIFIERS } from "../game/modifiers";
import { getAllRoles } from "../game/roleListState.d";
import { useLobbyOrGameState } from "./useHooks";
import type { WikiArticleLink } from "./WikiArticleLink";

export default function WikiCoverCard(
    props: Readonly<{
        initialWikiPage?: WikiArticleLink;
    }>,
): ReactElement {
    const enabledRoles = useLobbyOrGameState((state) => state.enabledRoles, ["enabledRoles"], getAllRoles())!;
    const enabledModifiers = useLobbyOrGameState(
        (state) => state.modifierSettings.keys(),
        ["modifierSettings"],
        ALL_MODIFIERS,
    )!;

    return (
        <div className="wiki-cover-card">
            <h1>{translate("menu.wiki.title")}</h1>
            <Wiki
                enabledRoles={enabledRoles}
                enabledModifiers={enabledModifiers}
                initialWikiPage={props.initialWikiPage}
            />
        </div>
    );
}
