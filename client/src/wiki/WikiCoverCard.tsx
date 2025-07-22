import React, { ReactElement, useContext } from 'react';
import translate from '../game/lang';
import Wiki from './Wiki';
import "./wiki.css";
import { WikiArticleLink } from './WikiArticleLink';
import { getAllRoles } from '../stateContext/stateType/roleListState';
import { MODIFIERS, ModifierType } from '../stateContext/stateType/modifiersState';
import { StateContext } from '../stateContext/StateContext';

export default function WikiCoverCard(props: Readonly<{
    initialWikiPage?: WikiArticleLink
}>): ReactElement {
    return <div className='wiki-cover-card'>
        <h1>{translate("menu.wiki.title")}</h1>
        <Wiki
            enabledRoles={useContext(StateContext)?.enabledRoles??getAllRoles()}
            enabledModifiers={useContext(StateContext)?.enabledModifiers??MODIFIERS as any as ModifierType[]}
            initialWikiPage={props.initialWikiPage}
        />
    </div>;
}
