use karna::prelude::*;

pub struct SaveSlot(pub u32);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Panel {
    #[default]
    Root,
    Settings,
}

pub struct MainMenu {
    ui: UiState,
    theme: Theme,
    panel: Panel,

    music_volume: f32,
    sfx_volume: f32,
    fullscreen: bool,
}

impl MainMenu {
    pub fn new(font: Handle<Font>) -> Self {
        Self {
            ui: UiState::default(),
            theme: Theme::dark(font),
            panel: Panel::Root,
            music_volume: 0.8,
            sfx_volume: 1.0,
            fullscreen: false,
        }
    }
}

impl Scene for MainMenu {
    fn load(&mut self, _ctx: ContextMut, scene: &mut SceneHandle) {
        scene.set_clear_color(Color::rgb(0.07, 0.08, 0.10));
    }

    fn update(&mut self, _ctx: ContextMut, _scene: &mut SceneHandle) {}

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        let mut ui = Ui::begin(draw, ctx.input, &mut self.ui, self.theme);

        match self.panel {
            Panel::Root => {
                ui.vstack_centered(300.0, |ui| {
                    ui.label("Super Awesome Game");
                    ui.spacing(24.0);

                    if ui.button("Play") {
                        println!("There's no game...")
                    }

                    if ui.button("Continue") {
                        println!("There's no game...")
                    }

                    if ui.button("Settings") {
                        self.panel = Panel::Settings;
                    }

                    ui.spacing(12.0);

                    if ui.button("Quit") {
                        ctx.window.close();
                    }
                });
            }

            Panel::Settings => {
                ui.vstack_centered(400.0, |ui| {
                    ui.label("Settings");
                    ui.spacing(16.0);

                    if ui.slider("Music", &mut self.music_volume, 0.0..=1.0) {}

                    if ui.slider("SFX", &mut self.sfx_volume, 0.0..=1.0) {}

                    if ui.checkbox("Fullscreen", &mut self.fullscreen) {
                        ctx.window.set_borderless(self.fullscreen);
                    }

                    ui.spacing(16.0);

                    if ui.button("Back") {
                        self.panel = Panel::Root;
                    }
                });
            }
        }

        ui.end();
    }

    fn on_key_press(&mut self, _ctx: ContextMut, _scene: &mut SceneHandle, keys: &[Keycode]) {
        if keys.contains(&Keycode::Escape) && self.panel == Panel::Settings {
            self.panel = Panel::Root;
        }
    }
}

fn main() {
    karna::logging::init(
        karna::logging::Config {
            min_level: karna::logging::LevelFilter::Debug,
            ..Default::default()
        }
        .hide_wgpu(true),
    )
    .expect("Failed to init logging");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .build_scene("mmenu", |ctx, _| {
                    let mut font = Handle::default();
                    ctx.assets.write_scope(|a| {
                        font = a.load_font(include_bytes!("assets/inter.ttf"), 18);
                    });

                    MainMenu::new(font)
                })
                .with_active_scene("mmenu"),
        )
        .build()
        .run();
}
