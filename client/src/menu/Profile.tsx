import { ReactElement, useContext } from "react";
import translate from "../game/lang";
import { useAuth0 } from "@auth0/auth0-react";
import React from "react";
import { AnchorControllerContext } from "./Anchor";
import Icon from "../components/Icon";
import "./profile.css";


export default function Profile(props: {}): ReactElement {
    const anchorController = useContext(AnchorControllerContext)!;
    const { user, logout } = useAuth0();

    if (user === undefined) {
        setTimeout(() => {
            anchorController.clearCoverCard();
        })
        return <></>;
    }

    return <div className="profile-menu">
        <header>
            <h1>{translate("menu.profile.title")}</h1>
            <button 
                onClick={() => {if (window.confirm(translate("logout.confirm"))) logout()}}
            ><Icon>logout</Icon>{translate("logout")}</button>
        </header>
        <div className="profile">
            {user.picture && <img src={user.picture} alt={translate("menu.profile.picture")}/>}
            <div className="profile-details">
                <h2>{user.name}</h2>
                <code>id: {user.sub}</code>
            </div>
        </div>
    </div>
}