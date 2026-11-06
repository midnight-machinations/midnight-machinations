import type { AudioFilePath } from "./Anchor";

const audio = new Audio();
let queue: string[] = [];
let autoplay = true;
let currentlyPlaying = false;

function playQueue() {
    if (queue.length === 0) return;

    currentlyPlaying = true;
    console.log(`Audio: Playing ${queue[0]}`);
    const next = () => {
        queue = queue.slice(1);
        currentlyPlaying = false;
        if (autoplay) playQueue();
    };
    playFile(queue[0], next, next);
}

function playFile(src: string, onEnd?: () => void, onError?: () => void) {
    audio.pause();
    audio.src = src;
    audio.load();

    const onEnded = () => {
        if (onEnd !== undefined) onEnd();
        audio.removeEventListener("ended", onEnded);
    };

    playAudio(() => {
        if (onError !== undefined) onError();
    });
    audio.addEventListener("ended", onEnded);
}

function playAudio(onError?: () => void) {
    audio
        .play()
        .then(() => {
            audio.currentTime = 0;
            audio.playbackRate = 1;
        })
        .catch((error) => {
            console.log(`Audio failed to play: ${error}`);
            if (onError !== undefined) onError();
        });
}

/** Plays queued sound effects one after another through a single shared audio element */
const AudioController = {
    setVolume(volume: number) {
        audio.volume = volume;
    },
    unpauseQueue() {
        if (!currentlyPlaying) {
            autoplay = true;
            playQueue();
        }
    },
    pauseQueue() {
        autoplay = false;
        audio.pause();
    },
    clearQueue() {
        queue = [];
    },
    queueFile(src: AudioFilePath) {
        queue.push(src);
        if (autoplay) {
            AudioController.unpauseQueue();
        }
    },
};

export default AudioController;

AudioController.clearQueue();
AudioController.pauseQueue();
