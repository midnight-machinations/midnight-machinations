import React, { ReactElement } from 'react';
import translate from '../game/lang';
import Wiki from '../components/Wiki';
import "./wiki.css";
import { WikiArticleLink } from './WikiArticleLink';
import { useLobbyOrGameState } from './useHooks';
import { MODIFIERS, ModifierType } from '../game/gameState.d';
import { getAllRoles } from '../game/roleListState.d';
import StyledText from './StyledText';

export default function WikiCoverCard(props: Readonly<{
    initialWikiPage?: WikiArticleLink
}>): ReactElement {
    const enabledRoles = useLobbyOrGameState(
        state => state.enabledRoles,
        ["enabledRoles"],
        getAllRoles()
    )!;
    const enabledModifiers = useLobbyOrGameState(
        state => state.enabledModifiers,
        ["enabledModifiers"],
        MODIFIERS as any as ModifierType[]
    )!;

    return <div className='wiki-cover-card'>
        <h1><StyledText noLinks={true}>{translate("menu.wiki.title")}</StyledText></h1>
        <Wiki enabledRoles={enabledRoles} enabledModifiers={enabledModifiers} initialWikiPage={props.initialWikiPage}/>
    </div>;
}
