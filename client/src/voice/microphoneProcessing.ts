import workletUrl from "./microphone.worklet.js?worker&url";
import { VoiceSettings } from "./types";

const modules = new WeakMap<AudioContext, Promise<void>>();

/** Owns the processing graph. Only the destination track may be transmitted. */
export class MicrophoneProcessing {
    private closed = false;
    private constructor(
        readonly source: MediaStreamAudioSourceNode,
        readonly node: AudioWorkletNode,
        readonly destination: MediaStreamAudioDestinationNode,
        readonly analyser: AnalyserNode,
    ) {}
    get track() { return this.destination.stream.getAudioTracks()[0]; }
    configure(settings: VoiceSettings) {
        this.node.port.postMessage({ noiseSuppression: settings.noiseSuppression, thresholdDb: settings.thresholdDb });
    }
    close() {
        if (this.closed) return;
        this.closed = true;
        this.node.port.postMessage({ close: true });
        this.node.port.onmessage = null;
        this.node.onprocessorerror = null;
        this.source.disconnect();
        this.node.disconnect();
        this.analyser.disconnect();
        this.track.stop();
    }
    static async create(audio: AudioContext, stream: MediaStream, settings: VoiceSettings,
        onLevel: (db: number, gateOpen: boolean) => void, onError: () => void) {
        let module = modules.get(audio);
        if (!module) {
            module = audio.audioWorklet.addModule(workletUrl);
            modules.set(audio, module);
            module.catch(() => modules.delete(audio));
        }
        await module;
        const source = audio.createMediaStreamSource(stream);
        const node = new AudioWorkletNode(audio, "voice-microphone", {
            numberOfInputs: 1, numberOfOutputs: 1, outputChannelCount: [1],
            channelCount: 1, channelCountMode: "explicit",
            processorOptions: { noiseSuppression: settings.noiseSuppression, thresholdDb: settings.thresholdDb },
        });
        const destination = audio.createMediaStreamDestination();
        const analyser = audio.createAnalyser();
        analyser.fftSize = 512;
        const processing = new MicrophoneProcessing(source, node, destination, analyser);
        // Never publish an enabled track, even briefly, while creating/replacing it.
        processing.track.enabled = false;
        try {
            await new Promise<void>((resolve, reject) => {
                const timeout = window.setTimeout(() => reject(new Error("Microphone processor did not initialize")), 10000);
                node.onprocessorerror = () => { clearTimeout(timeout); reject(new Error("Microphone processor failed")); };
                node.port.onmessage = ({ data }) => {
                    if (data.ready) { clearTimeout(timeout); resolve(); }
                    else if (typeof data.levelDb === "number") onLevel(data.levelDb, data.gateOpen);
                };
            });
            node.onprocessorerror = onError;
            source.connect(node);
            node.connect(analyser);
            analyser.connect(destination);
            return processing;
        } catch (error) { processing.close(); throw error; }
    }
}
