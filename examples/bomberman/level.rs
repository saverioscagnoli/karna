use karna::ContextRefMut;
use karna::Image;
use renderer::Draw;
use utils::Handle;

use crate::consts::GRID_HEIGHT_TILES;
use crate::consts::GRID_WIDTH_TILES;
use crate::consts::TILE_SIZE;

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

    floor: Handle<Image>,
    obstacle: Handle<Image>,

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

        Ok(Self {
            tiles,

            floor: ctx.assets.load_png(include_bytes!("images/floor.png")),
            obstacle: ctx.assets.load_png(include_bytes!("images/obstacle.png")),

            wall_top: ctx.assets.load_png(include_bytes!("images/w-t.png")),
            wall_top_left: ctx.assets.load_png(include_bytes!("images/w-tl.png")),
            wall_top_right: ctx.assets.load_png(include_bytes!("images/w-tr.png")),
            wall_left: ctx.assets.load_png(include_bytes!("images/w-l.png")),
            wall_right: ctx.assets.load_png(include_bytes!("images/w-r.png")),
            wall_bottom: ctx.assets.load_png(include_bytes!("images/w-b.png")),
            wall_bottom_left: ctx.assets.load_png(include_bytes!("images/w-bl.png")),
            wall_bottom_right: ctx.assets.load_png(include_bytes!("images/w-br.png")),
            wall_center: ctx.assets.load_png(include_bytes!("images/w-center.png")),
        })
    }

    #[inline]
    fn is_wall(&self, x: isize, y: isize) -> bool {
        if x < 0 || y < 0 {
            return false;
        }

        let x = x as usize;
        let y = y as usize;

        if x >= GRID_WIDTH_TILES || y >= GRID_HEIGHT_TILES {
            return false;
        }

        matches!(self.tiles[y][x], TileKind::Wall)
    }

    /// Choose the right wall sprite based on neighbor wall tiles.
    ///
    /// This is a simple "auto-tiling" rule using the grid indices.
    #[inline]
    fn wall_image_at(&self, x: usize, y: usize) -> Handle<Image> {
        let x = x as isize;
        let y = y as isize;

        let up = self.is_wall(x, y - 1);
        let down = self.is_wall(x, y + 1);
        let left = self.is_wall(x - 1, y);
        let right = self.is_wall(x + 1, y);

        // Isolated wall tile (no neighbors) -> center looks best.
        if !up && !down && !left && !right {
            return self.wall_center;
        }

        // Corners: require the wall to continue in the two "inside" directions.
        if !up && !left && right && down {
            return self.wall_top_left;
        }
        if !up && !right && left && down {
            return self.wall_top_right;
        }
        if !down && !left && right && up {
            return self.wall_bottom_left;
        }
        if !down && !right && left && up {
            return self.wall_bottom_right;
        }

        // Edges
        if !up && down {
            return self.wall_top;
        }
        if !down && up {
            return self.wall_bottom;
        }
        if !left && right {
            return self.wall_left;
        }
        if !right && left {
            return self.wall_right;
        }

        // Fallback (interior / ambiguous) -> center.
        self.wall_center
    }

    pub fn render(&self, draw: &mut Draw) {
        draw.push_state();
        draw.translate(0.0, 0.0);
        draw.scale(3.0, 3.0);
        for y in 0..GRID_HEIGHT_TILES {
            for x in 0..GRID_WIDTH_TILES {
                let px = (x * 16) as f32;
                let py = (y * 16) as f32;

                // Draw base floor everywhere.
                draw.image(self.floor, px, py);

                match self.tiles[y][x] {
                    TileKind::Floor => {}
                    TileKind::Obstacle => {
                        draw.image(self.obstacle, px, py);
                    }
                    TileKind::Wall => {
                        let wall = self.wall_image_at(x, y);
                        draw.image(wall, px, py);
                    }
                }
            }
        }

        draw.pop_state();
    }
}
