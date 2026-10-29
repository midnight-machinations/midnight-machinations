import React, { useEffect, useState, useSyncExternalStore } from "react";
import translate from "../game/lang";
import { VOICE } from "./voiceManager";
import Icon from "../components/Icon";
import SpeakingIndicator from "./SpeakingIndicator";
import { VoiceSettings } from "./types";
import Select, { SelectOptionsSearch } from "../components/Select";
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
            void navigator.mediaDevices?.enumerateDevices().then(values => setDevices(values.filter(value => value.kind === "audioinput"))).catch(() => {});
        };
        refresh();
        navigator.mediaDevices?.addEventListener("devicechange", refresh);
        return () => navigator.mediaDevices?.removeEventListener("devicechange", refresh);
    }, [voice.microphoneEnabled]);
    const modes: VoiceSettings["mode"][] = ["pushToTalk", "openMic", "toggleMute"];
    const modeOptions: SelectOptionsSearch<VoiceSettings["mode"]> = new Map(modes.map(mode => {
        const label = translate(`voice.${mode}`);
        return [mode, [label, label]];
    }));
    const defaultDevice = translate("voice.defaultDevice");
    const deviceOptions: SelectOptionsSearch<string> = new Map([["", [defaultDevice, defaultDevice]]]);
    devices.filter(device => device.deviceId !== "").forEach((device, index) => {
        const label = device.label || translate("voice.deviceNumber", index + 1);
        deviceOptions.set(device.deviceId, [label, label]);
    });
    if (voice.settings.deviceId && !deviceOptions.has(voice.settings.deviceId)) {
        const label = translate("voice.savedMicrophone");
        deviceOptions.set(voice.settings.deviceId, [label, label]);
    }
    return <fieldset className="voice-settings">
        <legend>{translate("voice.settings")}</legend>
        <label>{translate("voice.inputMode")}
            <Select className="voice-mode-select" value={voice.settings.mode} optionsSearch={modeOptions}
                onChange={mode => voice.setSettings({ mode })}/>
        </label>
        {voice.settings.mode !== "openMic" && <label>{translate(voice.settings.mode === "toggleMute" ? "voice.toggleMuteKey" : "voice.key")}
            <button type="button" onClick={() => setCaptureKey(true)} onBlur={() => setCaptureKey(false)}
                onKeyDown={event => {
                    if (!captureKey) return;
                    event.preventDefault(); event.stopPropagation();
                    if (event.code !== "Escape") voice.setSettings({ key: event.code });
                    setCaptureKey(false);
                }}>
                {captureKey ? translate("voice.pressKey") : voice.settings.key}
            </button>
        </label>}
        <label>{translate("voice.microphone")}
            <Select className="voice-device-select" value={voice.settings.deviceId} optionsSearch={deviceOptions}
                onChange={deviceId => voice.setSettings({ deviceId })}/>
        </label>
        <label>{translate("voice.masterVolume")}
            <input type="range" min="0" max="1" step="0.01" value={voice.settings.volume}
                onChange={event => voice.setSettings({ volume: Number(event.target.value) })}/>
        </label>
    </fieldset>;
}

export default function VoiceControls() {
    const voice = useVoice();
    const session = voice.session;
    const groups = [...new Set([...(session?.sendGroups ?? []), ...(session?.receiveGroups ?? [])])];
    return <section className="voice-menu chat-menu-colors">
        <header>
            <h2>{translate("voice.title")}</h2>
            <span>{translate(`voice.status.${voice.status}`)}</span>
        </header>
        <div className="voice-menu-content">
            {!voice.roomEnabled && <p>{translate("voice.disabledByHost")}</p>}
            {voice.roomEnabled && !session && <p>{translate("voice.unavailable")}</p>}
            {session && <>
                <div className="voice-buttons">
                    {voice.status === "disconnected" || voice.status === "error"
                        ? <button type="button" onClick={() => void voice.join()}>{translate("voice.join")}</button>
                        : <button type="button" onClick={() => voice.leave()}>{translate("voice.leave")}</button>}
                    <SpeakingIndicator roomClientId={session.selfId}/>
                </div>
                {voice.error && <p role="alert">{translate(voice.error)}</p>}
                {voice.error === "voice.error.playback" && <button type="button" onClick={() => void voice.resumePlayback()}>
                    {translate("voice.resumePlayback")}
                </button>}
                {session.forcedSilenced && <p>{translate("voice.forcedSilenced")}</p>}
                {voice.settings.mode === "pushToTalk" && voice.microphoneEnabled && <button type="button"
                    className="voice-talk" disabled={voice.status !== "connected" || voice.selfMuted || session.forcedSilenced || !session.sendGroups.some(group => !voice.group(group).muted)}
                    onPointerDown={event => { event.currentTarget.setPointerCapture(event.pointerId); voice.holdToTalk(true); }}
                    onPointerUp={() => voice.holdToTalk(false)} onPointerCancel={() => voice.holdToTalk(false)}
                    onLostPointerCapture={() => voice.holdToTalk(false)} onBlur={() => voice.holdToTalk(false)}>
                    {translate("voice.holdToTalk")}
                </button>}
                <fieldset>
                    <legend>{translate("voice.groups")}</legend>
                    {groups.length === 0 && <p>{translate("voice.noGroups")}</p>}
                    {groups.map(group => {
                        const name = group === "lobby" ? translate("voice.lobby") : translate(`chatGroup.${group}.name`);
                        const preference = voice.group(group);
                        const muteLabel = `${translate(preference.muted ? "voice.unmute" : "voice.mute")} ${name}`;
                        const deafenLabel = `${translate(preference.deafened ? "voice.undeafen" : "voice.deafen")} ${name}`;
                        return <div className="voice-group" key={group}>
                            <span title={session.sendGroups.includes(group) ? name : `${name}: ${translate("voice.listenOnly")}`}>{name}</span>
                            <button type="button" className="voice-icon-button" aria-pressed={preference.muted}
                                aria-label={muteLabel} title={muteLabel}
                                onClick={() => voice.setGroup(group, { muted: !preference.muted })}>
                                <Icon size="small">{preference.muted ? "mic_off" : "mic_none"}</Icon>
                            </button>
                            <button type="button" className="voice-icon-button" aria-pressed={preference.deafened}
                                aria-label={deafenLabel} title={deafenLabel}
                                onClick={() => voice.setGroup(group, { deafened: !preference.deafened })}>
                                <Icon size="small">{preference.deafened ? "headset_off" : "headset"}</Icon>
                            </button>
                        </div>;
                    })}
                </fieldset>
                <details>
                    <summary>{translate("voice.playerVolumes")}</summary>
                    {session.peers.filter(peer => peer.id !== session.selfId).map(peer => <div className="voice-person" key={peer.id}>
                        <span title={peer.name}>{peer.name}</span>
                        <SpeakingIndicator roomClientId={peer.id}/>
                        <input type="range" min="0" max="1" step="0.05" value={voice.person(peer.id).volume}
                            aria-label={translate("voice.playerVolume", peer.name)}
                            onChange={event => voice.setPerson(peer.id, { volume: Number(event.target.value) })}/>
                    </div>)}
                </details>
            </>}
            <VoiceSettingsControls/>
        </div>
    </section>;
}
