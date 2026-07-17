use karna::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveSlot(pub u32);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Panel {
    #[default]
    Root,
    SaveSelect,
    Settings,
}

pub struct MainMenu {
    ui: UiState,
    theme: Theme,
    panel: Panel,
    saves: Vec<SaveSlot>,
    selected_save: Option<SaveSlot>,
    music_volume: f32,
    sfx_volume: f32,
    fullscreen: bool,
    vsync: bool,
    danger: WidgetStyle,
    accent: WidgetStyle,
}

impl MainMenu {
    pub fn new(font: Handle<Font>) -> Self {
        let v = |r, g, b| math::Vector4::new(r, g, b, 1.0);

        Self {
            ui: UiState::default(),
            theme: Theme::dark(font),
            panel: Panel::Root,
            saves: vec![SaveSlot(1), SaveSlot(2)],
            selected_save: None,
            music_volume: 0.8,
            sfx_volume: 1.0,
            fullscreen: false,
            vsync: true,
            danger: WidgetStyle::from_accent(v(0.55, 0.16, 0.16), v(1.0, 0.92, 0.92)),
            accent: WidgetStyle::from_accent(v(0.16, 0.35, 0.60), v(0.95, 0.97, 1.0)),
        }
    }

    fn root(&mut self, ui: &mut Ui, ctx: &mut ContextMut) {
        ui.vstack_centered(300.0, |ui| {
            ui.label("Super Awesome Game");
            ui.spacing(24.0);

            if ui.button_styled("Play", self.accent) {
                self.selected_save = None;
                println!("Starting new game...");
            }

            if self.saves.is_empty() {
                ui.with_style(
                    StylePatch {
                        bg: Some(math::Vector4::new(0.14, 0.15, 0.17, 1.0)),
                        fg: Some(math::Vector4::new(0.5, 0.5, 0.5, 1.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.button("Continue");
                    },
                );
            } else if ui.button("Continue") {
                self.panel = Panel::SaveSelect;
            }

            if ui.button("Settings") {
                self.panel = Panel::Settings;
            }

            ui.spacing(12.0);

            if ui.button_styled("Quit", self.danger) {
                ctx.window.close();
            }
        });
    }

    fn save_select(&mut self, ui: &mut Ui) {
        ui.vstack_centered(300.0, |ui| {
            ui.label("Load Game");
            ui.spacing(16.0);

            for slot in self.saves.clone() {
                let label = format!("Save Slot {}", slot.0);
                let style = if self.selected_save == Some(slot) {
                    self.accent
                } else {
                    self.theme.button
                };

                if ui.button_styled(&label, style) {
                    self.selected_save = Some(slot);
                    println!("Loading save {}...", slot.0);
                }
            }

            ui.spacing(12.0);

            if let Some(slot) = self.selected_save {
                if ui.button_styled("Delete Selected", self.danger) {
                    self.saves.retain(|s| *s != slot);
                    self.selected_save = None;
                }
            }

            if ui.button("Back") {
                self.panel = Panel::Root;
            }
        });
    }

    fn settings(&mut self, ui: &mut Ui, ctx: &mut ContextMut) {
        ui.vstack_centered(400.0, |ui| {
            ui.label("Settings");
            ui.spacing(16.0);

            ui.label("Audio");
            if ui.slider("Music", &mut self.music_volume, 0.0..=1.0) {
                println!("Music volume: {:.2}", self.music_volume);
            }
            if ui.slider("SFX", &mut self.sfx_volume, 0.0..=1.0) {
                println!("SFX volume: {:.2}", self.sfx_volume);
            }

            ui.spacing(12.0);

            ui.label("Display");
            if ui.checkbox("Fullscreen", &mut self.fullscreen) {
                ctx.window.set_borderless(self.fullscreen);
            }
            if ui.checkbox("VSync", &mut self.vsync) {
                println!("VSync: {}", self.vsync);
            }

            ui.spacing(16.0);

            if ui.button("Back") {
                self.panel = Panel::Root;
            }
        });
    }
}

impl Scene for MainMenu {
    fn load(&mut self, _ctx: ContextMut, scene: &mut SceneHandle) {
        scene.set_clear_color(Color::rgb(0.07, 0.08, 0.10));
    }

    fn update(&mut self, _ctx: ContextMut, _scene: &mut SceneHandle) {}

    fn draw(&mut self, mut ctx: ContextMut, draw: &mut Draw) {
        let dt = ctx.time.delta();

        let mut ui_state = std::mem::take(&mut self.ui);
        let mut ui = Ui::begin(draw, ctx.input, &mut ui_state, self.theme, dt);

        match self.panel {
            Panel::Root => self.root(&mut ui, &mut ctx),
            Panel::SaveSelect => self.save_select(&mut ui),
            Panel::Settings => self.settings(&mut ui, &mut ctx),
        }

        ui.end();
        self.ui = ui_state;
    }

    fn on_key_press(&mut self, _ctx: ContextMut, _scene: &mut SceneHandle, keys: &[Keycode]) {
        if keys.contains(&Keycode::Escape) {
            match self.panel {
                Panel::Settings | Panel::SaveSelect => self.panel = Panel::Root,
                Panel::Root => {}
            }
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
