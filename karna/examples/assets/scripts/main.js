/// <reference path="../../../../assets/karna.d.ts" />
import { Player } from "./player.js";

const { Key, Mouse } = karna;

/** @implements {Scene} */
export default class Demo {
    player = new Player();
    /** @type {Vec2[]} */
    trail = [];
    /** @type {KarnaImage | undefined} */
    pcb;
    /** @type {KarnaAudio | undefined} */
    osu;

    load() {
        karna.time.setTargetFps(60);
        karna.window.setPresentMode(karna.PresentMode.Immediate);

        this.pcb = karna.assets.loadImage("assets/pcb.png");
        this.osu = karna.assets.loadAudio("assets/osu-hit-sound.wav");

        console.log(
            "script loaded, window is",
            karna.window.size().width,
            "x",
            karna.window.size().height,
        );
    }

    update() {
        this.player.update();

        if (karna.input.mousePressed(Mouse.Left)) {
            this.trail.push(karna.window.mousePosition());
            if (this.trail.length > 32) this.trail.shift();
        }

        if (karna.input.keyPressed(Key.Space)) this.trail = [];

        if (karna.input.keyPressed(Key.Y)) {
            karna.audio.play(this.osu);
        }
    }

    /**
     * @param {Draw} draw
     */
    draw(draw) {
        if (this.pcb)
            draw.image(this.pcb, karna.window.size().width - 266, 10, 256, 256);

        this.player.draw(draw);

        draw.setColor([1, 0.3, 0.8]);

        for (const p of this.trail) draw.circle(p.x, p.y, 6);

        draw.setColor([1, 1, 1]);
        draw.print(`fps: ${Math.round(karna.time.fps())}`, 10, 10);
        draw.print(
            "WASD / arrows to move, click to drop dots, space to clear",
            10,
            30,
        );
        draw.print(`dt: ${karna.time.delta()}`, 10, 50);
    }
}
