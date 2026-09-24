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

  /** @param {LoadContext} ctx */
  load(ctx) {
    ctx.time.setTargetFps(120);
    this.pcb = ctx.assets.loadImage("assets/pcb.png");
    karna.log(
      "script loaded, window is",
      ctx.window.width(),
      "x",
      ctx.window.height(),
    );
  }

  /** @param {UpdateContext} ctx */
  update(ctx) {
    this.player.update(ctx);

    if (ctx.input.mousePressed(Mouse.Left)) {
      this.trail.push(ctx.window.mouse());
      if (this.trail.length > 32) this.trail.shift();
    }

    if (ctx.input.keyPressed(Key.Space)) this.trail = [];
  }

  /**
   * @param {DrawContext} ctx
   * @param {Draw} draw
   */
  draw(ctx, draw) {
    if (this.pcb) draw.image(this.pcb, ctx.window.width() - 266, 10, 256, 256);

    this.player.draw(draw);

    draw.setColor(1, 0.3, 0.8);

    for (const p of this.trail) draw.circle(p.x, p.y, 6);

    draw.setColor(1, 1, 1);
    draw.print(`fps: ${Math.round(ctx.time.fps())}`, 10, 10);
    draw.print(
      "WASD / arrows to move, click to drop dots, space to clear",
      10,
      30,
    );
    draw.print(`dt: ${ctx.time.delta()}`, 10, 50);
  }
}
