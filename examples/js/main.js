import { Color, Cursor, Key, vec2 } from "karna";

import { clamp } from "./util.js";

const ACCEL = 900;
const DRAG = 0.86;
const SIZE = 50;

/** @type {import("karna").Scene} */
const demo = {
  load(ctx) {
    this.pos = vec2(80, 80);
    this.vel = vec2(0, 0);

    console.log("demo loaded at", ctx.window.width(), "x", ctx.window.height());
  },

  update(ctx) {
    const dt = ctx.time.delta();
    const input = ctx.input;

    if (input.keyDown(Key.W)) this.vel.y -= ACCEL * dt;
    if (input.keyDown(Key.S)) this.vel.y += ACCEL * dt;
    if (input.keyDown(Key.A)) this.vel.x -= ACCEL * dt;
    if (input.keyDown(Key.D)) this.vel.x += ACCEL * dt;

    this.pos = this.pos.add(this.vel.scale(dt));
    this.vel = this.vel.scale(DRAG);

    if (this.vel.length() < 0.25) this.vel.set(0, 0);

    const { width, height } = ctx.window.size();

    this.pos.set(
      clamp(this.pos.x, 0, width - SIZE),
      clamp(this.pos.y, 0, height - SIZE),
    );

    // The box in the middle gets a pointer cursor when the mouse is over it.
    const mouse = ctx.window.mousePosition();
    const over =
      mouse.x >= 400 && mouse.x < 560 && mouse.y >= 300 && mouse.y < 460;

    ctx.window.setCursor(over ? Cursor.POINTER : Cursor.DEFAULT);

    if (input.keyPressed(Key.F)) {
      ctx.window.activateScene("second");
      ctx.window.deactivateScene("demo");
    }

    if (input.keyPressed(Key.Escape)) ctx.window.quit();
  },

  draw(ctx, draw) {
    draw.setColor(Color.CYAN);
    draw.rect(400, 300, 160, 160);

    draw.setColor("#f38ba8");
    draw.rect(this.pos.x, this.pos.y, SIZE, SIZE);
  },
};

/** @type {import("karna").Scene} */
const second = {
  update(ctx) {
    if (ctx.input.keyPressed(Key.F)) {
      ctx.window.activateScene("demo");
      ctx.window.deactivateScene("second");
    }

    if (ctx.input.keyPressed(Key.Escape)) ctx.window.quit();
  },

  draw(ctx, draw) {
    draw.setColor(Color.MAGENTA);
    draw.rect(300, 150, 150, 450);
  },
};

export default {
  title: "karna - javascript",
  width: 1280,
  height: 720,

  scenes: { demo, second },
  scene: "demo",
};
