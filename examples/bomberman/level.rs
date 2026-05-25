use std::time::Duration;

use karna::ContextRefMut;
use karna::Image;
use math::Vector2;
use renderer::Draw;
use renderer::sprite::AnimatedSprite;
use renderer::sprite::animation::Animation;
use renderer::sprite::animation::Animations;
use renderer::sprite::animation::Frame;
use utils::Handle;

use crate::bomberman::Bomberman;
use crate::consts::GRID_HEIGHT_TILES;
use crate::consts::GRID_WIDTH_TILES;
use crate::consts::TILE_SIZE;

#[derive(Debug)]
struct TileAssets {
    floor: Handle<Image>,
    floor_edge: Handle<Image>,
    floor_shadow: Handle<Image>,
    obstacle_idle: AnimatedSprite,
    obstacle_idle_edge: AnimatedSprite,
    wall_top: Handle<Image>,
    wall_top_left: Handle<Image>,
    wall_top_right: Handle<Image>,
    wall_left: Handle<Image>,
    wall_right: Handle<Image>,
    wall_bottom: Handle<Image>,
    wall_bottom_left: Handle<Image>,
    wall_bottom_right: Handle<Image>,
    wall_center: Handle<Image>,
}

#[derive(Debug, Clone, Copy)]
pub enum TileKind {
    Wall,
    Obstacle,
    Floor,
}

impl TileKind {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'W' => Some(TileKind::Wall),
            'O' => Some(TileKind::Obstacle),
            'F' => Some(TileKind::Floor),
            _ => None,
        }
    }

    pub fn from_token(token: &str) -> Option<Self> {
        let t = token.trim();
        if t.len() != 1 {
            return None;
        }
        Self::from_char(t.chars().next().unwrap())
    }
}

#[derive(Debug, Clone)]
pub enum LevelParseError {
    WrongHeight {
        expected: usize,
        found: usize,
    },
    WrongWidth {
        row: usize,
        expected: usize,
        found: usize,
    },
    UnknownTile {
        row: usize,
        col: usize,
        token: String,
    },
}

#[derive(Debug)]
pub struct Level {
    tiles: [[TileKind; GRID_WIDTH_TILES]; GRID_HEIGHT_TILES],
    assets: TileAssets,
    bomberman: Bomberman,
}

impl Level {
    /// Parse the `.lvl` string and load the tile textures.
    pub fn new(level_string: &str, ctx: ContextRefMut) -> Result<Self, LevelParseError> {
        let mut tiles = [[TileKind::Floor; GRID_WIDTH_TILES]; GRID_HEIGHT_TILES];

        let lines: Vec<&str> = level_string
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();

        if lines.len() != GRID_HEIGHT_TILES {
            return Err(LevelParseError::WrongHeight {
                expected: GRID_HEIGHT_TILES,
                found: lines.len(),
            });
        }

        for (y, line) in lines.iter().enumerate() {
            // ignores the trailing comma in your format
            let tokens: Vec<&str> = line.split_terminator(',').collect();

            if tokens.len() != GRID_WIDTH_TILES {
                return Err(LevelParseError::WrongWidth {
                    row: y,
                    expected: GRID_WIDTH_TILES,
                    found: tokens.len(),
                });
            }

            for (x, token) in tokens.iter().enumerate() {
                let Some(kind) = TileKind::from_token(token) else {
                    return Err(LevelParseError::UnknownTile {
                        row: y,
                        col: x,
                        token: token.to_string(),
                    });
                };

                tiles[y][x] = kind;
            }
        }

        let obstacle_image = ctx.assets.load_png(include_bytes!("images/obstacle.png"));
        let obstacle_anims = Animations::new()
            .add_animation(
                "idle",
                Animation::default()
                    .add_frame(Frame::new(0, 0, 16, 16, Duration::from_millis(200)))
                    .add_frame(Frame::new(16, 0, 16, 16, Duration::from_millis(200)))
                    .add_frame(Frame::new(32, 0, 16, 16, Duration::from_millis(200)))
                    .add_frame(Frame::new(48, 0, 16, 16, Duration::from_millis(200))),
            )
            .add_animation(
                "idle-edge",
                Animation::default()
                    .add_frame(Frame::new(0, 16, 16, 16, Duration::from_millis(200)))
                    .add_frame(Frame::new(16, 16, 16, 16, Duration::from_millis(200)))
                    .add_frame(Frame::new(32, 16, 16, 16, Duration::from_millis(200)))
                    .add_frame(Frame::new(48, 16, 16, 16, Duration::from_millis(200))),
            );

        let mut obstacle_idle = AnimatedSprite::new(obstacle_image, obstacle_anims.clone());
        obstacle_idle.animator.play("idle", false);

        let mut obstacle_idle_edge = AnimatedSprite::new(obstacle_image, obstacle_anims);
        obstacle_idle_edge.animator.play("idle-edge", false);

        let assets = TileAssets {
            floor: ctx.assets.load_png(include_bytes!("images/floor.png")),
            floor_edge: ctx.assets.load_png(include_bytes!("images/floor-edge.png")),
            floor_shadow: ctx
                .assets
                .load_png(include_bytes!("images/floor-shadow.png")),
            obstacle_idle,
            obstacle_idle_edge,
            wall_top: ctx.assets.load_png(include_bytes!("images/w-t.png")),
            wall_top_left: ctx.assets.load_png(include_bytes!("images/w-tl.png")),
            wall_top_right: ctx.assets.load_png(include_bytes!("images/w-tr.png")),
            wall_left: ctx.assets.load_png(include_bytes!("images/w-l.png")),
            wall_right: ctx.assets.load_png(include_bytes!("images/w-r.png")),
            wall_bottom: ctx.assets.load_png(include_bytes!("images/w-b.png")),
            wall_bottom_left: ctx.assets.load_png(include_bytes!("images/w-bl.png")),
            wall_bottom_right: ctx.assets.load_png(include_bytes!("images/w-br.png")),
            wall_center: ctx.assets.load_png(include_bytes!("images/w-center.png")),
        };

        let bomberman = ctx.assets.load_png(include_bytes!("images/bomberman.png"));
        let sprite = AnimatedSprite::new(
            bomberman,
            Animations::default()
                .add_animation(
                    "walk-left",
                    Animation::default()
                        .add_frame(Frame::new(0, 0, 16, 25, Duration::from_millis(200)))
                        .add_frame(Frame::new(16, 0, 16, 25, Duration::from_millis(200)))
                        .add_frame(Frame::new(32, 0, 16, 25, Duration::from_millis(200))),
                )
                .add_animation(
                    "walk-down",
                    Animation::default()
                        .add_frame(Frame::new(1, 30, 14, 24, Duration::from_millis(200)))
                        .add_frame(Frame::new(16, 30, 15, 24, Duration::from_millis(200)))
                        .add_frame(Frame::new(32, 30, 15, 24, Duration::from_millis(200))),
                )
                .add_animation(
                    "walk-right",
                    Animation::default()
                        .add_frame(Frame::new(0, 58, 16, 24, Duration::from_millis(200)))
                        .add_frame(Frame::new(16, 58, 16, 24, Duration::from_millis(200)))
                        .add_frame(Frame::new(32, 58, 16, 24, Duration::from_millis(200))),
                )
                .add_animation(
                    "walk-up",
                    Animation::default()
                        .add_frame(Frame::new(0, 86, 15, 23, Duration::from_millis(200)))
                        .add_frame(Frame::new(16, 87, 15, 22, Duration::from_millis(200)))
                        .add_frame(Frame::new(32, 86, 15, 23, Duration::from_millis(200))),
                ),
        );

        let mut bomberman = Bomberman::new(Vector2::new(100.0, 100.0), sprite);
        bomberman.load();

        Ok(Self {
            tiles,
            assets,
            bomberman,
        })
    }

    pub fn update(&mut self, ctx: ContextRefMut) {
        let dt = ctx.time.delta();

        self.assets.obstacle_idle.update(dt);
        self.assets.obstacle_idle_edge.update(dt);
        self.bomberman.update(ctx);
    }

    pub fn render(&self, draw: &mut Draw) {
        draw.push_state();
        draw.translate(0.0, 0.0);
        draw.scale(3.0, 3.0);

        for y in 0..self.tiles.len() {
            for x in 0..self.tiles[y].len() {
                let px = (x * TILE_SIZE) as f32;
                let py = (y * TILE_SIZE) as f32;
                let tile = self.tiles[y][x];

                let image = match tile {
                    TileKind::Floor => {
                        if y == 0 {
                            continue;
                        }

                        let prev_tile = self.tiles[y - 1][x];
                        match prev_tile {
                            TileKind::Wall => self.assets.floor_edge,
                            TileKind::Obstacle => self.assets.floor_shadow,
                            _ => self.assets.floor,
                        }
                    }
                    TileKind::Wall => {
                        if y == 0 {
                            if x == 0 {
                                self.assets.wall_top_left
                            } else if x == GRID_WIDTH_TILES - 1 {
                                self.assets.wall_top_right
                            } else {
                                self.assets.wall_top
                            }
                        } else if y == GRID_HEIGHT_TILES - 1 {
                            if x == 0 {
                                self.assets.wall_bottom_left
                            } else if x == GRID_WIDTH_TILES - 1 {
                                self.assets.wall_bottom_right
                            } else {
                                self.assets.wall_bottom
                            }
                        } else {
                            if x == 0 {
                                self.assets.wall_left
                            } else if x == GRID_WIDTH_TILES - 1 {
                                self.assets.wall_right
                            } else {
                                self.assets.wall_center
                            }
                        }
                    }
                    TileKind::Obstacle => {
                        let above = if y > 0 {
                            self.tiles[y - 1][x]
                        } else {
                            TileKind::Floor
                        };

                        let sprite = match above {
                            TileKind::Wall => &self.assets.obstacle_idle_edge,
                            _ => &self.assets.obstacle_idle,
                        };

                        sprite.draw(draw, px, py);
                        continue;
                    }
                };

                draw.image(image, px, py);
            }
        }

        draw.pop_state();
        self.bomberman.draw(draw);
    }
}
