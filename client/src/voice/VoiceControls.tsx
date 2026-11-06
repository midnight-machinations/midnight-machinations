import type React from "react";
import { useEffect, useState, useSyncExternalStore } from "react";
import { encodeString } from "../components/ChatMessage";
import CheckBox from "../components/CheckBox";
import Icon from "../components/Icon";
import Select, { type SelectOptionsSearch } from "../components/Select";
import StyledText from "../components/StyledText";
import translate from "../game/lang";
import { formatKeybind, isModifierKey, keybindFromEvent } from "./keybind";
import SpeakingIndicator from "./SpeakingIndicator";
import type { VoiceSettings } from "./types";
import { VOICE } from "./voiceManager";
import "./voice.css";

function useVoice() {
    useSyncExternalStore(VOICE.subscribe, VOICE.getRevision);
    return VOICE;
}

export function VoiceSettingsControls() {
    const voice = useVoice();
    const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
    const [captureKey, setCaptureKey] = useState(false);
    useEffect(() => {
        const refresh = () => {
            void navigator.mediaDevices
                ?.enumerateDevices()
                .then((values) => setDevices(values.filter((value) => value.kind === "audioinput")))
                .catch(() => {});
        };
        refresh();
        navigator.mediaDevices?.addEventListener("devicechange", refresh);
        return () => navigator.mediaDevices?.removeEventListener("devicechange", refresh);
    }, [voice.microphoneEnabled]);
    const modes: VoiceSettings["mode"][] = ["pushToTalk", "openMic", "toggleMute"];
    const modeOptions: SelectOptionsSearch<VoiceSettings["mode"]> = new Map(
        modes.map((mode) => {
            const label = translate(`voice.${mode}`);
            return [mode, [<StyledText noLinks={true}>{label}</StyledText>, label]];
        }),
    );
    const defaultDevice = translate("voice.defaultDevice");
    const deviceOptions: SelectOptionsSearch<string> = new Map([
        ["", [<StyledText noLinks={true}>{defaultDevice}</StyledText>, defaultDevice]],
    ]);
    devices
        .filter((device) => device.deviceId !== "")
        .forEach((device, index) => {
            const label = device.label || translate("voice.deviceNumber", index + 1);
            deviceOptions.set(device.deviceId, [<StyledText noLinks={true}>{encodeString(label)}</StyledText>, label]);
        });
    if (voice.settings.deviceId && !deviceOptions.has(voice.settings.deviceId)) {
        const label = translate("voice.savedMicrophone");
        deviceOptions.set(voice.settings.deviceId, [
            <StyledText noLinks={true}>{encodeString(label)}</StyledText>,
            label,
        ]);
    }
    return (
        <fieldset className="voice-settings">
            <legend>
                <StyledText noLinks={true}>{translate("voice.settings")}</StyledText>
            </legend>
            <label>
                <StyledText noLinks={true}>{translate("voice.inputMode")}</StyledText>
                <Select
                    className="voice-mode-select"
                    value={voice.settings.mode}
                    optionsSearch={modeOptions}
                    onChange={(mode) => voice.setSettings({ mode })}
                />
            </label>
            {voice.settings.mode !== "openMic" && (
                <label>
                    <StyledText noLinks={true}>
                        {translate(voice.settings.mode === "toggleMute" ? "voice.toggleMuteKey" : "voice.key")}
                    </StyledText>
                    <button
                        type="button"
                        onClick={() => setCaptureKey(true)}
                        onBlur={() => setCaptureKey(false)}
                        onKeyDown={(event) => {
                            if (!captureKey) return;
                            event.preventDefault();
                            event.stopPropagation();
                            if (isModifierKey(event.code) || event.repeat || event.nativeEvent.isComposing) return;
                            if (event.code !== "Escape") voice.setSettings({ key: keybindFromEvent(event) });
                            setCaptureKey(false);
                        }}
                        onKeyUp={(event) => {
                            if (!captureKey || !isModifierKey(event.code)) return;
                            event.preventDefault();
                            event.stopPropagation();
                            voice.setSettings({ key: keybindFromEvent(event) });
                            setCaptureKey(false);
                        }}
                    >
                        <StyledText noLinks={true}>
                            {captureKey ? translate("voice.pressKey") : encodeString(formatKeybind(voice.settings.key))}
                        </StyledText>
                    </button>
                </label>
            )}
            <label>
                <StyledText noLinks={true}>{translate("voice.microphone")}</StyledText>
                <Select
                    className="voice-device-select"
                    value={voice.settings.deviceId}
                    optionsSearch={deviceOptions}
                    onChange={(deviceId) => voice.setSettings({ deviceId })}
                />
            </label>
            <div className="voice-setting-row">
                <span>
                    <StyledText noLinks={true}>{translate("voice.noiseSuppression")}</StyledText>
                </span>
                <CheckBox
                    checked={voice.settings.noiseSuppression}
                    ariaLabel={translate("voice.noiseSuppression")}
                    onChange={(noiseSuppression) => voice.setSettings({ noiseSuppression })}
                />
            </div>
            <label htmlFor="voice-input-threshold">
                <StyledText noLinks={true}>{translate("voice.inputThreshold")}</StyledText>
                <output htmlFor="voice-input-threshold">
                    <StyledText noLinks={true}>
                        {voice.settings.thresholdDb <= -80
                            ? translate("voice.thresholdOff")
                            : `${voice.settings.thresholdDb} dB`}
                    </StyledText>
                </output>
            </label>
            <input
                id="voice-input-threshold"
                className="voice-threshold"
                type="range"
                min="-80"
                max="0"
                step="1"
                value={voice.settings.thresholdDb}
                style={
                    {
                        "--voice-level": `${Math.max(0, Math.min(100, ((voice.microphoneLevelDb + 80) / 80) * 100))}%`,
                        "--voice-threshold": `${((voice.settings.thresholdDb + 80) / 80) * 100}%`,
                    } as React.CSSProperties
                }
                aria-valuetext={
                    voice.settings.thresholdDb <= -80
                        ? translate("voice.thresholdOff")
                        : `${voice.settings.thresholdDb} dB`
                }
                onChange={(event) => voice.setSettings({ thresholdDb: Number(event.target.value) })}
            />
            <label htmlFor="voice-master-volume">
                <StyledText noLinks={true}>{translate("voice.masterVolume")}</StyledText>
                <output htmlFor="voice-master-volume">
                    <StyledText noLinks={true}>{`${Math.round(voice.settings.volume * 100)}%`}</StyledText>
                </output>
            </label>
            <input
                id="voice-master-volume"
                type="range"
                min="0"
                max="2"
                step="0.01"
                value={voice.settings.volume}
                aria-valuetext={`${Math.round(voice.settings.volume * 100)}%`}
                onChange={(event) => voice.setSettings({ volume: Number(event.target.value) })}
            />
        </fieldset>
    );
}

export default function VoiceControls() {
    const voice = useVoice();
    const session = voice.session;
    const groups = [...new Set([...(session?.sendGroups ?? []), ...(session?.receiveGroups ?? [])])];
    return (
        <section className="voice-menu chat-menu-colors">
            <header>
                <h2>
                    <StyledText noLinks={true}>{translate("voice.title")}</StyledText>
                </h2>
                <span>
                    <StyledText noLinks={true}>{translate(`voice.status.${voice.status}`)}</StyledText>
                </span>
            </header>
            <div className="voice-menu-content">
                {!voice.roomEnabled && (
                    <p>
                        <StyledText noLinks={true}>{translate("voice.disabledByHost")}</StyledText>
                    </p>
                )}
                {voice.roomEnabled && !session && (
                    <p>
                        <StyledText noLinks={true}>{translate("voice.unavailable")}</StyledText>
                    </p>
                )}
                {session && (
                    <>
                        <div className="voice-buttons">
                            {voice.status === "disconnected" || voice.status === "error" ? (
                                <button type="button" onClick={() => void voice.join()}>
                                    <StyledText noLinks={true}>{translate("voice.join")}</StyledText>
                                </button>
                            ) : (
                                <button type="button" onClick={() => voice.leave()}>
                                    <StyledText noLinks={true}>{translate("voice.leave")}</StyledText>
                                </button>
                            )}
                            <SpeakingIndicator roomClientId={session.selfId} />
                        </div>
                        {voice.error && (
                            <p role="alert">
                                <StyledText noLinks={true}>{translate(voice.error)}</StyledText>
                            </p>
                        )}
                        {voice.error === "voice.error.playback" && (
                            <button type="button" onClick={() => void voice.resumePlayback()}>
                                <StyledText noLinks={true}>{translate("voice.resumePlayback")}</StyledText>
                            </button>
                        )}
                        {session.forcedSilenced && (
                            <p>
                                <StyledText noLinks={true}>{translate("voice.forcedSilenced")}</StyledText>
                            </p>
                        )}
                        {voice.settings.mode === "pushToTalk" && voice.microphoneEnabled && (
                            <button
                                type="button"
                                className="voice-talk"
                                disabled={
                                    voice.status !== "connected" ||
                                    voice.selfMuted ||
                                    session.forcedSilenced ||
                                    session.sendGroups.length === 0
                                }
                                onPointerDown={(event) => {
                                    event.currentTarget.setPointerCapture(event.pointerId);
                                    voice.holdToTalk(true);
                                }}
                                onPointerUp={() => voice.holdToTalk(false)}
                                onPointerCancel={() => voice.holdToTalk(false)}
                                onLostPointerCapture={() => voice.holdToTalk(false)}
                                onBlur={() => voice.holdToTalk(false)}
                            >
                                <StyledText noLinks={true}>{translate("voice.holdToTalk")}</StyledText>
                            </button>
                        )}
                        <fieldset>
                            <legend>
                                <StyledText noLinks={true}>{translate("voice.groups")}</StyledText>
                            </legend>
                            {groups.length === 0 && (
                                <p>
                                    <StyledText noLinks={true}>{translate("voice.noGroups")}</StyledText>
                                </p>
                            )}
                            {groups.map((group) => {
                                const name =
                                    group === "lobby" ? translate("voice.lobby") : translate(`chatGroup.${group}.name`);
                                const preference = voice.group(group);
                                const deafenLabel = `${translate(preference.deafened ? "voice.undeafen" : "voice.deafen")} ${name}`;
                                return (
                                    <div className="voice-group" key={group}>
                                        <span
                                            title={
                                                session.sendGroups.includes(group)
                                                    ? name
                                                    : `${name}: ${translate("voice.listenOnly")}`
                                            }
                                        >
                                            <StyledText noLinks={true}>{name}</StyledText>
                                        </span>
                                        <button
                                            type="button"
                                            className="voice-icon-button"
                                            aria-pressed={preference.deafened}
                                            aria-label={deafenLabel}
                                            title={deafenLabel}
                                            onClick={() => voice.setGroup(group, { deafened: !preference.deafened })}
                                        >
                                            <Icon size="small">{preference.deafened ? "headset_off" : "headset"}</Icon>
                                        </button>
                                    </div>
                                );
                            })}
                        </fieldset>
                        {voice.status === "connected" && (
                            <details>
                                <summary>
                                    <StyledText noLinks={true}>{translate("voice.playerVolumes")}</StyledText>
                                </summary>
                                {session.peers
                                    .filter((peer) => peer.id !== session.selfId)
                                    .map((peer) => (
                                        <div className="voice-person" key={peer.id}>
                                            <span title={peer.name}>
                                                <StyledText noLinks={true}>{encodeString(peer.name)}</StyledText>
                                            </span>
                                            <SpeakingIndicator roomClientId={peer.id} />
                                            <input
                                                type="range"
                                                min="0"
                                                max="1"
                                                step="0.05"
                                                value={voice.person(peer.id).volume}
                                                aria-label={translate("voice.playerVolume", peer.name)}
                                                onChange={(event) =>
                                                    voice.setPerson(peer.id, { volume: Number(event.target.value) })
                                                }
                                            />
                                        </div>
                                    ))}
                            </details>
                        )}
                    </>
                )}
                <VoiceSettingsControls />
            </div>
        </section>
    );
}
