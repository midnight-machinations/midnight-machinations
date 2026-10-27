import { ReactElement, useEffect, useRef } from "react";
import "./vcrNoise.css"

// VCR tracking noise, adapted from the "vcr" effect in https://codepen.io/Mobius1/pen/ZNgwbr

// The original effect was tuned for a 360px tall screen; tracking values are scaled to the canvas height
const REFERENCE_HEIGHT = 360;
const MIN_Y = 220;
const MIN_Y2 = 220;
const NUM = 50;
const RADIUS = 4;
const FRAME_INTERVAL = 1000 / 30;

function getRandomInt(min: number, max: number): number {
    min = Math.ceil(min);
    max = Math.floor(max);
    return Math.floor(Math.random() * (max - min + 1)) + min;
}

function renderTail(ctx: CanvasRenderingContext2D, x: number, y: number, radius: number) {
    const n = getRandomInt(1, 50);
    const dir = Math.random() < 0.5 ? 1 : -1;
    let rd = radius;

    for (let i = 0; i < n; i++) {
        const r = getRandomInt(rd -= 0.01, radius);
        radius -= 0.1;
        ctx.fillRect(x += getRandomInt(1, 4) * dir, y, r, r);
    }
}

function renderTrackingNoise(ctx: CanvasRenderingContext2D) {
    const { width, height } = ctx.canvas;
    const scale = height / REFERENCE_HEIGHT;
    const step = 3 * scale;
    let posy1 = MIN_Y * scale;
    let posy3 = MIN_Y2 * scale;

    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = "#333";

    for (let i = 0; i <= NUM; i++) {
        const x = Math.random() * width;
        const y1 = getRandomInt(posy1 += step, height);
        const y2 = getRandomInt(0, posy3 -= step);
        ctx.fillRect(x, y1, RADIUS, RADIUS);
        ctx.fillRect(x, y2, RADIUS, RADIUS);

        renderTail(ctx, x, y1, RADIUS);
        renderTail(ctx, x, y2, RADIUS);
    }
}

/// Full-screen VCR noise. Place it first inside an element with `isolation: isolate`
/// so it renders above that element's background but beneath its content.
export default function VcrNoise(): ReactElement {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        const ctx = canvasRef.current?.getContext("2d");
        if (!ctx || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

        const resize = () => {
            ctx.canvas.width = window.innerWidth;
            ctx.canvas.height = window.innerHeight;
        };
        resize();
        window.addEventListener("resize", resize);

        let frame: number;
        let lastRender = -Infinity;
        const animate = (time: number) => {
            // Small tolerance so a 60Hz display doesn't skip a frame due to timing jitter
            if (time - lastRender >= FRAME_INTERVAL - 1) {
                lastRender = time;
                renderTrackingNoise(ctx);
            }
            frame = requestAnimationFrame(animate);
        };
        frame = requestAnimationFrame(animate);

        return () => {
            window.removeEventListener("resize", resize);
            cancelAnimationFrame(frame);
        };
    }, []);

    return <canvas ref={canvasRef} className="vcr-noise" aria-hidden="true"/>;
}
