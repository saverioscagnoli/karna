use karna::ContextRefMut;
use karna::KeyCode;
use math::Vector2;
use renderer::Draw;
use renderer::sprite::AnimatedSprite;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Bomberman {
    position: Vector2,
    velocity: Vector2,
    sprite: AnimatedSprite,
    last_direction: Direction,
}

impl Bomberman {
    const ACCEL: f32 = 250.0;

    pub fn new(position: Vector2, sprite: AnimatedSprite) -> Self {
        Self {
            position,
            velocity: Vector2::zeros(),
            sprite,
            last_direction: Direction::Down,
        }
    }

    pub fn load(&mut self) {
        self.sprite.animator.play("walk-down", true);
        self.sprite.animator.pause();
    }

    pub fn update(&mut self, ctx: ContextRefMut) {
        let dt = ctx.time.delta();

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.velocity.y = -Self::ACCEL;
            self.last_direction = Direction::Up;
            self.sprite.animator.play("walk-up", false);
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.velocity.y = Self::ACCEL;
            self.last_direction = Direction::Down;
            self.sprite.animator.play("walk-down", false);
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.velocity.x = -Self::ACCEL;
            self.last_direction = Direction::Left;
            self.sprite.animator.play("walk-left", false);
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.velocity.x = Self::ACCEL;
            self.last_direction = Direction::Right;
            self.sprite.animator.play("walk-right", false);
        }

        self.position += self.velocity * dt;

        if self.velocity.x == 0.0 && self.velocity.y == 0.0 {
            match self.last_direction {
                Direction::Up => self.sprite.animator.set_frame(1),
                Direction::Down => self.sprite.animator.set_frame(1),
                Direction::Left => self.sprite.animator.set_frame(0),
                Direction::Right => self.sprite.animator.set_frame(0),
            };

            self.sprite.animator.pause();
        }

        self.velocity = Vector2::zeros();
        self.sprite.update(dt);
    }

    pub fn draw(&self, draw: &mut Draw) {
        draw.push_state();
        draw.translate_v(self.position);
        draw.scale_v([3.0, 3.0]);
        self.sprite.draw(draw, 0.0, 0.0);
        draw.pop_state();
    }
}
