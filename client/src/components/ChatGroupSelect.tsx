import GAME_MANAGER from "..";
import type { ControllerID } from "../game/controllerInput";
import type { ChatGroup } from "../game/gameState.d";
import translate from "../game/lang";
import Select, { type SelectOptionsSearch } from "./Select";
import StyledText from "./StyledText";

export default function ChatGroupSelect(
    props: Readonly<{
        id: ControllerID;
        groups: ChatGroup[];
        selected: ChatGroup | null;
        disabled?: boolean;
    }>,
) {
    const options: SelectOptionsSearch<ChatGroup | "none"> = new Map();
    for (const group of props.groups) {
        const name = translate(`chatGroup.${group}.name`);
        options.set(group, [
            <StyledText key={group} noLinks={true}>{`${translate(`chatGroup.${group}.icon`)} ${name}`}</StyledText>,
            name,
        ]);
    }
    if (props.groups.length === 0)
        options.set("none", [
            <StyledText key="none" noLinks={true}>
                {translate("none")}
            </StyledText>,
            translate("none"),
        ]);
    const selected =
        props.selected !== null && props.groups.includes(props.selected)
            ? props.selected
            : props.groups.includes("all")
              ? "all"
              : (props.groups[0] ?? "none");
    return (
        <div className="chat-group-select" title={translate("menu.chat.sendGroup")}>
            <Select
                value={selected}
                optionsSearch={options}
                disabled={props.disabled || props.groups.length === 0}
                onChange={(group) => {
                    if (group !== "none")
                        GAME_MANAGER.sendControllerInput({
                            id: props.id,
                            selection: { type: "chatGroup", selection: group },
                        });
                }}
            />
        </div>
    );
}
