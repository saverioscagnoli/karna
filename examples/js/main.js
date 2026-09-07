import { Color, Cursor, Key, Vec2, vec2 } from "karna";

import { clamp } from "./util.js";

const ACCEL = 900;
const DRAG = 0.86;
const SIZE = 50;

export default {
  /**
   *
   * @param {import("karna").Context} ctx
   */
  load(ctx) {
    ctx.window.setTitle("karna - javascript");

    this.pos = vec2(80, 80);
    this.vel = vec2(0, 0);

    this.pcb = ctx.assets.loadImage("assets/pcb.png");
    this.font = ctx.assets.loadFont("assets/jbmono.ttf");

    console.log(
      "scene loaded, viewport",
      ctx.window.width(),
      "x",
      ctx.window.height(),
    );
  },

  /**
   *
   * @param {import("karna").Context} ctx
   */
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

    // The box under the mouse gets a pointer cursor.
    const mouse = ctx.window.mousePosition();
    const over =
      mouse.x >= 400 && mouse.x < 560 && mouse.y >= 300 && mouse.y < 460;

    ctx.window.setCursor(over ? Cursor.POINTER : Cursor.DEFAULT);
  },

  /**
   *
   * @param {import("karna").Context} ctx
   * @param {import("karna").Draw} draw
   */
  draw(ctx, draw) {
    draw.setColor(Color.CYAN);
    draw.rect(400, 300, 160, 160);

    draw.setColor(Color.WHITE);
    draw.image(this.pcb, 620, 80);

    draw.setColor("#f38ba8");
    draw.rect(this.pos.x, this.pos.y, SIZE, SIZE);

    const style = { font: this.font, size: 18 };

    draw.setColor(Color.WHITE);
    draw.print(`${ctx.time.fps().toFixed(0)} fps`, style, 8, 8);

    draw.print(
      [
        { text: "wasd", color: "#a6e3a1", bold: true },
        { text: " to move", color: "#a6adc8" },
      ],
      style,
      8,
      30,
    );
  },
};
