// The AudioWorkletGlobalScope, which TypeScript's DOM library does not describe.
declare const sampleRate: number;

declare abstract class AudioWorkletProcessor {
    readonly port: MessagePort;
    constructor(options?: AudioWorkletNodeOptions);
    abstract process(inputs: Float32Array[][], outputs: Float32Array[][], parameters: Record<string, Float32Array>): boolean;
}

declare function registerProcessor(name: string, processor: new (options: AudioWorkletNodeOptions) => AudioWorkletProcessor): void;

// RNNoise compiled to WebAssembly by @jitsi/rnnoise-wasm, which ships no types.
declare module "@jitsi/rnnoise-wasm/dist/rnnoise-sync.js" {
    interface RnnoiseModule {
        HEAPF32: Float32Array;
        _malloc(bytes: number): number;
        _free(pointer: number): void;
        _rnnoise_create(model: number): number;
        _rnnoise_destroy(state: number): void;
        _rnnoise_process_frame(state: number, output: number, input: number): number;
    }
    export default function createRNNWasmModuleSync(): RnnoiseModule;
}
