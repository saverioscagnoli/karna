#![allow(unused)]

use karna::App;
use karna::Context;
use karna::Scene;
use karna::SceneView;
use karna::WindowBuilder;
use karna::assets::Handle;
use karna::assets::Image;
use karna::logging::Config;
use karna::render::Draw;

use image::ExtendedColorType;
use image::ImageEncoder;
use image::codecs::png::PngEncoder;
use logging::info;

const ATLAS_SCENE: usize = 0;
const RECT_COUNT: usize = 500;

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn range(&mut self, lo: u32, hi: u32) -> u32 {
        lo + (self.next_u64() % (hi - lo) as u64) as u32
    }

    fn byte(&mut self) -> u8 {
        (self.next_u64() & 0xff) as u8
    }
}

fn random_rect_png(rng: &mut Rng) -> Vec<u8> {
    let w = rng.range(16, 129);
    let h = rng.range(16, 129);

    let (r, g, b) = (rng.byte(), rng.byte(), rng.byte());

    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let border = x == 0 || y == 0 || x == w - 1 || y == h - 1;
            let (cr, cg, cb) = if border { (255, 255, 255) } else { (r, g, b) };
            px[i] = cr;
            px[i + 1] = cg;
            px[i + 2] = cb;
            px[i + 3] = 255;
        }
    }

    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(&px, w, h, ExtendedColorType::Rgba8)
        .expect("failed to encode rectangle to png");
    out
}

struct AtlasDemo {
    handles: Vec<Handle<Image>>,
    active_page: usize,
    timer: f32,
}

impl AtlasDemo {
    fn new() -> Self {
        Self {
            handles: Vec::with_capacity(RECT_COUNT),
            active_page: 0,
            timer: 0.0,
        }
    }
}

impl Scene for AtlasDemo {
    fn load(&mut self, ctx: &mut Context, _scene: &mut SceneView) {
        let mut rng = Rng::new(0x9E37_79B9_7F4A_7C15);

        let pngs: Vec<Vec<u8>> = (0..RECT_COUNT).map(|_| random_rect_png(&mut rng)).collect();

        ctx.assets.write_scope(|w| {
            for png in &pngs {
                self.handles.push(w.load_image(png));
            }
        });

        info!("loaded {} rectangles into the atlas", self.handles.len());
    }

    fn update(&mut self, ctx: &mut Context, _scene: &mut SceneView) {
        let pages = ctx.assets.atlas_page_handles().count();

        if pages == 0 {
            return;
        }

        self.timer += ctx.time.delta();

        if self.timer >= 2.0 {
            self.timer = 0.0;
            self.active_page = (self.active_page + 1) % pages;

            ctx.window
                .set_title(&format!("texture atlas - page {}", self.active_page));
        }
    }

    fn draw(&mut self, ctx: &mut Context, draw: &mut Draw) {
        for (i, handle) in ctx.assets.atlas_page_handles().enumerate() {
            if i == self.active_page {
                draw.image(handle, 0.0, 0.0);
            }
        }
    }
}

fn main() {
    karna::init_logging(Config::default().with_min_level(karna::logging::LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("texture atlas - Page 0")
                .with_size((1024, 1024))
                .with_scene(ATLAS_SCENE, AtlasDemo::new())
                .with_active_scene(ATLAS_SCENE),
        )
        .build()
        .run();
}
