import GAME_MANAGER from "../../..";
import { Button } from "../../../components/Button";
import Icon from "../../../components/Icon";
import PlayerOptionDropdown from "../../../components/PlayerOptionDropdown";
import Select, { type SelectOptionsSearch } from "../../../components/Select";
import StyledText from "../../../components/StyledText";
import { usePlayerNames } from "../../../components/useHooks";
import type { ChatFilter } from "../../../game/chatFilter";
import translate from "../../../game/lang";

type FilterMode = NonNullable<ChatFilter>["type"];

export default function ChatFilters(props: Readonly<{ filter: ChatFilter }>) {
    const names = usePlayerNames();
    const filter = props.filter;
    const modes: FilterMode[] = [
        "text",
        "playerNameInMessage",
        "playerMessages",
        "alibiBlocks",
        "nonPlayer",
        "nightResults",
    ];
    if (filter?.type === "whispersBetweenPlayers") modes.push("whispersBetweenPlayers");
    const options: SelectOptionsSearch<FilterMode> = new Map(
        modes.map((mode) => {
            const label = translate(`menu.chat.filter.${mode}`);
            return [
                mode,
                [
                    <StyledText key={mode} noLinks={true}>
                        {label}
                    </StyledText>,
                    label,
                ],
            ];
        }),
    );
    return (
        <div className="chat-filter-controls">
            <div className="chat-filter-actions">
                {filter !== null && (
                    <Button
                        className="chat-filter-icon-button"
                        onClick={() => GAME_MANAGER.updateChatFilter(null)}
                        aria-label={translate("menu.chat.clearFilter")}
                    >
                        <Icon>filter_alt_off</Icon>
                    </Button>
                )}
                <Select<FilterMode>
                    className={filter === null ? "chat-filter-icon-button" : undefined}
                    value={filter?.type ?? null}
                    optionsSearch={options}
                    placeholder={<Icon>filter_alt</Icon>}
                    ariaLabel={translate("menu.chat.filter.label")}
                    hideArrow={filter === null ? true : undefined}
                    onChange={(mode) => {
                        switch (mode) {
                            case "text":
                                GAME_MANAGER.updateChatFilter({ type: "text", text: "" });
                                break;
                            case "playerNameInMessage":
                                GAME_MANAGER.updateChatFilter({ type: mode, player: 0 });
                                break;
                            case "whispersBetweenPlayers":
                                break;
                            default:
                                GAME_MANAGER.updateChatFilter({ type: mode });
                        }
                    }}
                />
            </div>
            {filter?.type === "text" && (
                <input
                    type="search"
                    // biome-ignore lint/a11y/noAutofocus: the search box opens because the user just chose the text filter
                    autoFocus
                    aria-label={translate("menu.chat.filter.search")}
                    placeholder={translate("menu.chat.filter.search")}
                    value={filter.text}
                    onChange={(event) => GAME_MANAGER.updateChatFilter({ type: "text", text: event.target.value })}
                />
            )}
            {filter?.type === "playerNameInMessage" && names.length > 0 && (
                <PlayerOptionDropdown
                    value={filter.player}
                    onChange={(player) => {
                        if (player !== null) GAME_MANAGER.updateChatFilter({ type: "playerNameInMessage", player });
                    }}
                />
            )}
        </div>
    );
}
