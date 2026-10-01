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

  load() {
      karna.time.setTargetFps(120);
    this.pcb = karna.assets.loadImage("assets/pcb.png");
    console.log(
      "script loaded, window is",
      karna.window.width(),
      "x",
      karna.window.height(),
    );
  }

  update() {
      this.player.update();


    if (karna.input.mousePressed(Mouse.Left)) {
      this.trail.push(karna.window.mouse());
      if (this.trail.length > 32) this.trail.shift();
    }

    if (karna.input.keyPressed(Key.Space)) this.trail = [];
  }

  /**
   * @param {Draw} draw
   */
  draw(draw) {
    if (this.pcb) draw.image(this.pcb, karna.window.width() - 266, 10, 256, 256);

    this.player.draw(draw);

    draw.setColor(1, 0.3, 0.8);

    for (const p of this.trail) draw.circle(p.x, p.y, 6);

    draw.setColor(1, 1, 1);
    draw.print(`fps: ${Math.round(karna.time.fps())}`, 10, 10);
    draw.print(
      "WASD / arrows to move, click to drop dots, space to clear",
      10,
      30,
    );
    draw.print(`dt: ${karna.time.delta()}`, 10, 50);
  }
}
