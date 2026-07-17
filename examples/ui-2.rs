use karna::prelude::*;

fn v(r: f32, g: f32, b: f32) -> math::Vector4<f32> {
    math::Vector4::new(r, g, b, 1.0)
}

fn rounded(mut w: WidgetStyle, base: f32, hovered: f32, held: f32) -> WidgetStyle {
    w.base.border_radius = base;
    w.hovered.border_radius = hovered;
    w.held.border_radius = held;
    w
}

fn gold() -> WidgetStyle {
    rounded(
        WidgetStyle::from_accent(v(0.55, 0.42, 0.10), v(1.0, 0.96, 0.85)),
        4.0,
        12.0,
        6.0,
    )
}

fn red() -> WidgetStyle {
    rounded(
        WidgetStyle::from_accent(v(0.45, 0.13, 0.13), v(1.0, 0.90, 0.90)),
        4.0,
        12.0,
        6.0,
    )
}

struct BarColors {
    back: Color,
    fill: Color,
    text: Color,
}

fn bar(ui: &mut Ui, rect: Rect, current: f32, max: f32, c: &BarColors, font: Handle<Font>) {
    let t = if max > 0.0 {
        (current / max).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let d = ui.draw();

    d.set_color(c.back);
    d.rect_v(rect.position, rect.size);
    d.set_color(c.fill);
    d.rect_v(
        rect.position,
        Size::new(rect.size.width * t, rect.size.height),
    );

    let label = format!("{:.0} / {:.0}", current, max);
    let ts = d.measure_text(font, &label);
    d.set_color(c.text);
    d.text(
        font,
        &label,
        rect.position.x + (rect.size.width - ts.width) * 0.5,
        rect.position.y + (rect.size.height - ts.height) * 0.5,
    );
}

fn panel(ui: &mut Ui, rect: Rect, title: &str, font: Handle<Font>, f: impl FnOnce(&mut Ui)) {
    let title_h = 32.0;
    let pad = 8.0;

    let d = ui.draw();
    d.set_color(Color::rgba(0.08, 0.09, 0.12, 0.92));
    d.rect_v(rect.position, rect.size);
    d.set_color(Color::rgb(0.16, 0.13, 0.08));
    d.rect_v(rect.position, Size::new(rect.size.width, title_h));

    let ts = d.measure_text(font, title);
    d.set_color(Color::rgb(0.90, 0.78, 0.45));
    d.text(
        font,
        title,
        rect.position.x + (rect.size.width - ts.width) * 0.5,
        rect.position.y + (title_h - ts.height) * 0.5,
    );

    ui.at(
        rect.position.x + pad,
        rect.position.y + title_h + pad,
        rect.size.width - 2.0 * pad,
    );
    f(ui);
}

pub struct HudDemo {
    ui: UiState,
    theme: Theme,
    font: Handle<Font>,
    ability_style: WidgetStyle,

    hp: f32,
    hp_max: f32,
    mp: f32,
    mp_max: f32,
    xp: f32,
    xp_next: f32,

    music_volume: f32,
    show_quest: bool,
}

impl HudDemo {
    pub fn new(font: Handle<Font>) -> Self {
        let theme = Theme::dark(font);

        Self {
            ui: UiState::default(),
            theme,
            font,
            ability_style: rounded(theme.button, 6.0, 14.0, 8.0),
            hp: 340.0,
            hp_max: 425.0,
            mp: 120.0,
            mp_max: 200.0,
            xp: 8630.0,
            xp_next: 12400.0,
            music_volume: 0.8,
            show_quest: true,
        }
    }

    fn cast_ability(&mut self, i: usize) {
        let cost = 15.0 + i as f32 * 10.0;
        if self.mp >= cost {
            self.mp -= cost;
            self.xp = (self.xp + 150.0).min(self.xp_next);
            println!("Cast ability {}!", i + 1);
        } else {
            println!("Not enough mana...");
        }
    }
}

impl Scene for HudDemo {
    fn load(&mut self, _ctx: ContextMut, scene: &mut SceneHandle) {
        scene.set_clear_color(Color::rgb(0.05, 0.10, 0.06));
    }

    fn update(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        let dt = ctx.time.delta();
        self.mp = (self.mp + 8.0 * dt).min(self.mp_max);
        self.hp = (self.hp + 3.0 * dt).min(self.hp_max);
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        let dt = ctx.time.delta();
        let mut ui_state = std::mem::take(&mut self.ui);
        let mut ui = Ui::begin(draw, ctx.input, &mut ui_state, self.theme, dt);

        let view = ui.viewport();
        let font = self.font;

        let hp_colors = BarColors {
            back: Color::rgb(0.15, 0.05, 0.05),
            fill: Color::rgb(0.75, 0.15, 0.15),
            text: Color::rgb(1.0, 0.95, 0.95),
        };
        let mp_colors = BarColors {
            back: Color::rgb(0.05, 0.07, 0.15),
            fill: Color::rgb(0.20, 0.35, 0.85),
            text: Color::rgb(0.95, 0.97, 1.0),
        };
        let xp_colors = BarColors {
            back: Color::rgb(0.10, 0.07, 0.13),
            fill: Color::rgb(0.55, 0.30, 0.75),
            text: Color::rgb(0.97, 0.93, 1.0),
        };

        bar(
            &mut ui,
            Rect {
                position: Vector2::new(20.0, 20.0),
                size: Size::new(220.0, 22.0),
            },
            self.hp,
            self.hp_max,
            &hp_colors,
            font,
        );
        bar(
            &mut ui,
            Rect {
                position: Vector2::new(20.0, 46.0),
                size: Size::new(220.0, 16.0),
            },
            self.mp,
            self.mp_max,
            &mp_colors,
            font,
        );

        let slot = 48.0;
        let gap = 6.0;
        let bar_w = 5.0 * slot + 4.0 * gap;
        let origin = Vector2::new((view.width - bar_w) * 0.5, view.height - slot - 20.0);

        bar(
            &mut ui,
            Rect {
                position: Vector2::new(origin.x, origin.y - 12.0),
                size: Size::new(bar_w, 8.0),
            },
            self.xp,
            self.xp_next,
            &xp_colors,
            font,
        );

        let mut cast = None;
        for i in 0..5 {
            ui.at(origin.x + i as f32 * (slot + gap), origin.y, slot);
            let label = ["1", "2", "3", "4", "5"][i];
            if ui.button_styled(label, self.ability_style) {
                cast = Some(i);
            }
        }

        if self.show_quest {
            let mut abandon = false;
            let music = &mut self.music_volume;
            let slider_style = self.theme.slider;

            panel(
                &mut ui,
                Rect {
                    position: Vector2::new(view.width - 320.0, 80.0),
                    size: Size::new(300.0, 260.0),
                },
                "Super fantastic quest",
                font,
                |ui| {
                    ui.label("Idk slay something");
                    ui.label("Bottom text");
                    ui.spacing(8.0);

                    if ui.button_styled("Accept", gold()) {
                        println!("Quest accepted!");
                    }
                    if ui.button("Track") {
                        println!("Tracking quest.");
                    }
                    if ui.button_styled("Abandon", red()) {
                        println!("Quest abandoned :(");
                        abandon = true;
                    }

                    ui.spacing(12.0);
                    ui.slider_styled("Music", music, 0.0..=1.0, slider_style, v(0.85, 0.85, 0.88));
                },
            );

            if abandon {
                self.show_quest = false;
            }
        }

        ui.end();
        self.ui = ui_state;

        if let Some(i) = cast {
            self.cast_ability(i);
        }
    }

    fn on_key_press(&mut self, _ctx: ContextMut, _scene: &mut SceneHandle, keys: &[Keycode]) {
        const HOTKEYS: [Keycode; 5] = [
            Keycode::Digit1,
            Keycode::Digit2,
            Keycode::Digit3,
            Keycode::Digit4,
            Keycode::Digit5,
        ];
        for (i, k) in HOTKEYS.iter().enumerate() {
            if keys.contains(k) {
                self.cast_ability(i);
            }
        }
        if keys.contains(&Keycode::Space) {
            self.hp = (self.hp - 60.0).max(0.0);
        }
        if keys.contains(&Keycode::KeyQ) {
            self.show_quest = !self.show_quest;
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
                .build_scene("hud", |ctx, _| {
                    let mut font = Handle::default();
                    ctx.assets.write_scope(|a| {
                        font = a.load_font(include_bytes!("assets/inter.ttf"), 16);
                    });
                    HudDemo::new(font)
                })
                .with_active_scene("hud"),
        )
        .build()
        .run();
}
