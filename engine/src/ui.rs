use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::RangeInclusive;

use assets::Font;
use utils::Handle;
use winit::event::MouseButton;

use crate::Draw;
use crate::input::Input;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UiId(u64);

impl UiId {
    fn make(label: &str, seq: u32) -> Self {
        let mut h = DefaultHasher::new();

        label.hash(&mut h);
        seq.hash(&mut h);
        UiId(h.finish())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Rect {
    pub position: math::Vector2<f32>,
    pub size: math::Size<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Style {
    pub bg: math::Vector4<f32>,
    pub fg: math::Vector4<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WidgetStyle {
    pub base: Style,
    pub hovered: Style,
    pub held: Style,
}

impl WidgetStyle {
    fn resolve(&self, hovered: bool, held: bool) -> &Style {
        if held {
            &self.held
        } else if hovered {
            &self.hovered
        } else {
            &self.base
        }
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub button: WidgetStyle,
    pub checkbox: WidgetStyle,
    pub slider: WidgetStyle,
    pub text: math::Vector4<f32>,
    pub padding: f32,
    pub item_gap: f32,
    pub widget_height: f32,
    pub font: Handle<Font>,
}

impl Theme {
    pub fn dark(font: Handle<Font>) -> Self {
        let v = |r, g, b| math::Vector4::new(r, g, b, 1.0);
        let ws = |n, h, a| WidgetStyle {
            base: Style {
                bg: n,
                fg: v(0.9, 0.9, 0.9),
            },
            hovered: Style {
                bg: h,
                fg: v(1.0, 1.0, 1.0),
            },
            held: Style {
                bg: a,
                fg: v(1.0, 1.0, 1.0),
            },
        };

        Self {
            button: ws(
                v(0.20, 0.22, 0.27),
                v(0.28, 0.31, 0.38),
                v(0.14, 0.15, 0.19),
            ),
            checkbox: ws(
                v(0.20, 0.22, 0.27),
                v(0.28, 0.31, 0.38),
                v(0.14, 0.15, 0.19),
            ),
            slider: ws(
                v(0.20, 0.22, 0.27),
                v(0.28, 0.31, 0.38),
                v(0.35, 0.55, 0.95),
            ),
            text: v(0.92, 0.92, 0.92),
            padding: 8.0,
            item_gap: 6.0,
            widget_height: 32.0,
            font,
        }
    }
}

#[derive(Default)]
pub struct UiState {
    hot: Option<UiId>,
    active: Option<UiId>,
    pub wants_mouse: bool,
}

pub struct Ui<'a, 'd> {
    draw: &'a mut Draw<'d>,
    state: &'a mut UiState,
    theme: Theme,

    cursor: math::Vector2<f32>,
    col_width: f32,

    seq: u32,
    hot_this_frame: Option<UiId>,
    mouse: math::Vector2<f32>,
    mouse_pressed: bool,
    mouse_held: bool,
}

impl<'a, 'd> Ui<'a, 'd> {
    pub fn begin(
        draw: &'a mut Draw<'d>,
        input: &'a Input,
        state: &'a mut UiState,
        theme: Theme,
    ) -> Self {
        draw.set_layer(renderer::Layer::Ui);

        let mp = input.mouse_position();
        let mouse = math::Vector2::new(mp[0], mp[1]);
        let mouse_pressed = input.pressed_mouse_buttons.contains(&MouseButton::Left);
        let mouse_held = input.held_mouse_buttons.contains(&MouseButton::Left);

        Self {
            draw,
            state,
            theme,
            cursor: math::Vector2::new(0.0, 0.0),
            col_width: 200.0,
            seq: 0,
            hot_this_frame: None,
            mouse,
            mouse_pressed,
            mouse_held,
        }
    }

    pub fn end(self) {
        self.state.hot = self.hot_this_frame;

        if !self.mouse_held {
            self.state.active = None;
        }

        self.state.wants_mouse = self.hot_this_frame.is_some() || self.state.active.is_some();
    }

    pub fn vstack_centered(&mut self, width: f32, f: impl FnOnce(&mut Self)) {
        let view = self.draw.viewport().as_f32();
        self.col_width = width;
        self.cursor = math::Vector2::new((view.width - width) * 0.5, view.height / 3.0);
        f(self);
    }

    pub fn spacing(&mut self, px: f32) {
        self.cursor.y += px;
    }

    fn next_rect(&mut self, h: f32) -> Rect {
        let r = Rect {
            position: self.cursor,
            size: math::Size::new(self.col_width, h),
        };
        self.cursor.y += h + self.theme.item_gap;
        r
    }

    fn interact(&mut self, id: UiId, rect: Rect) -> (bool, bool, bool) {
        let hovered = rect.size.contains_point(rect.position, self.mouse);
        let mut clicked = false;

        if hovered {
            self.hot_this_frame = Some(id);
            if self.mouse_pressed {
                self.state.active = Some(id);
            }
        }

        let held = self.state.active == Some(id);

        if held && !self.mouse_held {
            clicked = hovered;
            self.state.active = None;
        }

        (hovered, held && self.mouse_held, clicked)
    }

    pub fn label(&mut self, text: &str) {
        let size = self.draw.measure_text(self.theme.font, text);
        let rect = self.next_rect(size.height.max(self.theme.widget_height));

        self.draw.set_color(self.theme.text);
        self.draw.text(
            self.theme.font,
            text,
            rect.position.x,
            rect.position.y + self.theme.padding,
        );
    }

    pub fn button(&mut self, label: &str) -> bool {
        self.seq += 1;
        let id = UiId::make(label, self.seq);
        let rect = self.next_rect(self.theme.widget_height);
        let (hovered, held, clicked) = self.interact(id, rect);

        let style = self.theme.button.resolve(hovered, held);

        self.draw.set_color(style.bg);
        self.draw.rect_v(rect.position, rect.size);

        let text_size = self.draw.measure_text(self.theme.font, label);
        self.draw.set_color(style.fg);
        self.draw.text(
            self.theme.font,
            label,
            rect.position.x + (rect.size.width - text_size.width) * 0.5,
            rect.position.y + (rect.size.height - text_size.height) * 0.5,
        );

        clicked
    }

    pub fn checkbox(&mut self, label: &str, value: &mut bool) -> bool {
        self.seq += 1;
        let id = UiId::make(label, self.seq);
        let rect = self.next_rect(self.theme.widget_height);
        let (hovered, held, clicked) = self.interact(id, rect);

        if clicked {
            *value = !*value;
        }

        let style = self.theme.checkbox.resolve(hovered, held);
        let box_size = rect.size.height - 2.0 * self.theme.padding;
        let bx = rect.position.x;
        let by = rect.position.y + self.theme.padding;

        self.draw.set_color(style.bg);
        self.draw.rect(bx, by, box_size, box_size);

        if *value {
            let inset = box_size * 0.25;
            self.draw.set_color(style.fg);
            self.draw.rect(
                bx + inset,
                by + inset,
                box_size - 2.0 * inset,
                box_size - 2.0 * inset,
            );
        }

        self.draw.set_color(self.theme.text);
        self.draw.text(
            self.theme.font,
            label,
            bx + box_size + self.theme.padding,
            rect.position.y + self.theme.padding,
        );

        clicked
    }

    pub fn slider(&mut self, label: &str, value: &mut f32, range: RangeInclusive<f32>) -> bool {
        self.seq += 1;
        let id = UiId::make(label, self.seq);
        let rect = self.next_rect(self.theme.widget_height);

        let hovered = rect.size.contains_point(rect.position, self.mouse);

        if hovered {
            self.hot_this_frame = Some(id);
            if self.mouse_pressed {
                self.state.active = Some(id);
            }
        }

        let dragging = self.state.active == Some(id) && self.mouse_held;
        let mut changed = false;

        if dragging {
            let t = ((self.mouse.x - rect.position.x) / rect.size.width).clamp(0.0, 1.0);
            let (lo, hi) = (*range.start(), *range.end());
            let new = lo + t * (hi - lo);
            changed = new != *value;
            *value = new;
        }

        if self.state.active == Some(id) && !self.mouse_held {
            self.state.active = None;
        }

        let style = self.theme.slider.resolve(hovered, dragging);
        let (lo, hi) = (*range.start(), *range.end());
        let t = ((*value - lo) / (hi - lo)).clamp(0.0, 1.0);

        self.draw.set_color(style.bg);
        self.draw.rect_v(rect.position, rect.size);

        self.draw.set_color(style.fg);
        self.draw.rect_v(
            rect.position,
            math::Size::new(rect.size.width * t, rect.size.height),
        );

        self.draw.text(
            self.theme.font,
            label,
            rect.position.x + self.theme.padding,
            rect.position.y + self.theme.padding,
        );

        changed
    }

    pub fn draw(&mut self) -> &mut Draw<'d> {
        self.draw
    }

    pub fn at(&mut self, x: f32, y: f32, width: f32) {
        self.cursor = math::Vector2::new(x, y);
        self.col_width = width;
    }

    pub fn viewport(&self) -> math::Size<f32> {
        self.draw.viewport().as_f32()
    }

    pub fn button_styled(&mut self, label: &str, style: WidgetStyle) -> bool {
        self.seq += 1;
        let id = UiId::make(label, self.seq);
        let rect = self.next_rect(self.theme.widget_height);
        let (hovered, held, clicked) = self.interact(id, rect);

        let s = style.resolve(hovered, held);

        self.draw.set_color(s.bg);
        self.draw.rect_v(rect.position, rect.size);

        let ts = self.draw.measure_text(self.theme.font, label);

        self.draw.set_color(s.fg);
        self.draw.text(
            self.theme.font,
            label,
            rect.position.x + (rect.size.width - ts.width) * 0.5,
            rect.position.y + (rect.size.height - ts.height) * 0.5,
        );

        clicked
    }

    pub fn slider_styled(
        &mut self,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        style: WidgetStyle,
        label_color: math::Vector4<f32>,
    ) -> bool {
        self.seq += 1;
        let id = UiId::make(label, self.seq);
        let rect = self.next_rect(self.theme.widget_height);

        let label_w = rect.size.width * 0.4;
        let track = Rect {
            position: math::Vector2::new(rect.position.x + label_w, rect.position.y),
            size: math::Size::new(rect.size.width - label_w, rect.size.height),
        };

        let hovered = track.size.contains_point(track.position, self.mouse);

        if hovered {
            self.hot_this_frame = Some(id);
            if self.mouse_pressed {
                self.state.active = Some(id);
            }
        }
        let dragging = self.state.active == Some(id) && self.mouse_held;
        let mut changed = false;

        if dragging {
            let t = ((self.mouse.x - track.position.x) / track.size.width).clamp(0.0, 1.0);
            let (lo, hi) = (*range.start(), *range.end());
            let new = lo + t * (hi - lo);
            changed = new != *value;
            *value = new;
        }
        if self.state.active == Some(id) && !self.mouse_held {
            self.state.active = None;
        }

        let s = style.resolve(hovered, dragging);
        let (lo, hi) = (*range.start(), *range.end());
        let t = ((*value - lo) / (hi - lo)).clamp(0.0, 1.0);

        self.draw.set_color(label_color);
        self.draw.text(
            self.theme.font,
            label,
            rect.position.x,
            rect.position.y + self.theme.padding,
        );

        self.draw.set_color(s.bg);
        self.draw.rect_v(track.position, track.size);
        self.draw.set_color(s.fg);
        self.draw.rect_v(
            track.position,
            math::Size::new(track.size.width * t, track.size.height),
        );

        changed
    }
}
