import createRNNWasmModuleSync from "@jitsi/rnnoise-wasm/dist/rnnoise-sync.js";

// RNNoise consumes 10 ms frames of mono 48 kHz PCM in the int16 amplitude range.
const rnnoise = createRNNWasmModuleSync();
const FRAME = 480;

class VoiceMicrophoneProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();
        if (sampleRate !== 48000) throw new Error("Voice processing requires 48 kHz audio");
        this.state = rnnoise._rnnoise_create(0);
        this.buffer = rnnoise._malloc(FRAME * 4);
        this.frame = new Float32Array(FRAME);
        this.framePosition = 0;
        this.output = new Float32Array(2048);
        this.read = 0;
        this.write = FRAME;
        this.suppression = options.processorOptions.noiseSuppression;
        this.thresholdDb = options.processorOptions.thresholdDb;
        this.hold = 0;
        this.gain = 0;
        this.frames = 0;
        this.closed = false;
        this.port.onmessage = ({ data }) => {
            if (data.close) {
                rnnoise._rnnoise_destroy(this.state);
                rnnoise._free(this.buffer);
                this.closed = true;
                this.port.close();
            } else {
                this.suppression = data.noiseSuppression;
                this.thresholdDb = data.thresholdDb;
            }
        };
        this.port.postMessage({ ready: true });
    }
    process(inputs, outputs) {
        if (this.closed) return false;
        const input = inputs[0]?.[0];
        const output = outputs[0]?.[0];
        if (!output) return true;
        for (let i = 0; i < output.length; i++) {
            this.frame[this.framePosition++] = input?.[i] ?? 0;
            if (this.framePosition === FRAME) {
                this.framePosition = 0;
                if (this.suppression) {
                    const pcm = rnnoise.HEAPF32.subarray(this.buffer / 4, this.buffer / 4 + FRAME);
                    for (let j = 0; j < FRAME; j++) pcm[j] = this.frame[j] * 32768;
                    rnnoise._rnnoise_process_frame(this.state, this.buffer, this.buffer);
                    for (let j = 0; j < FRAME; j++) this.frame[j] = pcm[j] / 32768;
                }
                let energy = 0;
                for (let j = 0; j < FRAME; j++) energy += this.frame[j] * this.frame[j];
                const db = 20 * Math.log10(Math.max(0.000001, Math.sqrt(energy / FRAME)));
                // A short hold and hysteresis avoid chopping word endings at the threshold.
                const threshold = this.thresholdDb - (this.hold > 0 ? 3 : 0);
                if (this.thresholdDb <= -80 || db >= threshold) this.hold = 20;
                else this.hold = Math.max(0, this.hold - 1);
                const target = this.hold > 0 ? 1 : 0;
                for (let j = 0; j < FRAME; j++) {
                    // Ramp over 5 ms rather than introducing clicks at gate transitions.
                    this.gain += Math.max(-1 / 240, Math.min(1 / 240, target - this.gain));
                    this.output[this.write++ % this.output.length] = this.frame[j] * this.gain;
                }
                if (++this.frames % 10 === 0) this.port.postMessage({ levelDb: Math.max(-80, db), gateOpen: target === 1 });
            }
            output[i] = this.output[this.read++ % this.output.length];
        }
        return true;
    }
}
registerProcessor("voice-microphone", VoiceMicrophoneProcessor);
