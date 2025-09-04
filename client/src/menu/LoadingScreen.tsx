import React from "react";
import "../index.css"
import "./loadingScreen.css"
import translate from "../game/lang";

export default function LoadingScreen(props: {type: "default" | "host" | "join" | "disconnect" | "login"}) {
    return <div className="loading">
        <h1>{translate("menu.loading." + props.type)}</h1>
        <div className="loading-dots"></div>
    </div>
}