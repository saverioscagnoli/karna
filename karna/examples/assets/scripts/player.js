/// <reference path="../../../../libs/js/karna.d.ts" />

const { Key } = karna;

const SPEED = 300;

export class Player {
  x = 100;
  y = 100;
  size = 40;

  /** @param {UpdateContext} ctx */
  update(ctx) {
    const { input } = ctx;
    const step = SPEED * ctx.time.delta();

    if (input.keyDown(Key.W) || input.keyDown(Key.Up)) this.y -= step;
    if (input.keyDown(Key.S) || input.keyDown(Key.Down)) this.y += step;
    if (input.keyDown(Key.A) || input.keyDown(Key.Left)) this.x -= step;
    if (input.keyDown(Key.D) || input.keyDown(Key.Right)) this.x += step;
  }

  /** @param {Graphics} g */
  draw(draw) {
    draw.setColor(0.2, 0.8, 1);
    draw.rect(this.x, this.y, this.size, this.size);
  }
}
