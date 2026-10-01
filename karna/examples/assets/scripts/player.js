/// <reference path="../../../../assets/karna.d.ts" />

const { Key } = karna;

const SPEED = 300;

export class Player {
  x = 100;
  y = 100;
  size = 40;

  update() {
    const step = SPEED * karna.time.delta();

    if (karna.input.keyDown(Key.W) || karna.input.keyDown(Key.Up)) this.y -= step;
    if (karna.input.keyDown(Key.S) || karna.input.keyDown(Key.Down)) this.y += step;
    if (karna.input.keyDown(Key.A) || karna.input.keyDown(Key.Left)) this.x -= step;
    if (karna.input.keyDown(Key.D) || karna.input.keyDown(Key.Right)) this.x += step;
  }

  draw(draw) {
    draw.setColor([0.2, 0.8, 1]);
    draw.rect(this.x, this.y, this.size, this.size);
  }
}
