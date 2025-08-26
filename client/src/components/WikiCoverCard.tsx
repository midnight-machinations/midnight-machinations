import React, { ReactElement } from 'react';
import translate from '../game/lang';
import Wiki from '../components/Wiki';
import "./wiki.css";
import { WikiArticleLink } from './WikiArticleLink';
import { useLobbyOrGameState } from './useHooks';
import { MODIFIERS, ModifierID } from '../game/gameState.d';
import { getAllRoles } from '../game/roleListState.d';

export default function WikiCoverCard(props: Readonly<{
    initialWikiPage?: WikiArticleLink
}>): ReactElement {
    const enabledRoles = useLobbyOrGameState(
        state => state.enabledRoles,
        ["enabledRoles"],
        getAllRoles()
    )!;
    const enabledModifiers = useLobbyOrGameState(
        state => state.modifiers,
        ["enabledModifiers"],
        MODIFIERS as any as ModifierID[]
    )!;

    return <div className='wiki-cover-card'>
        <h1>{translate("menu.wiki.title")}</h1>
        <Wiki enabledRoles={enabledRoles} enabledModifiers={enabledModifiers} initialWikiPage={props.initialWikiPage}/>
    </div>;
}
