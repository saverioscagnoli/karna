use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::RangeInclusive;

use assets::Font;
use utils::Handle;
use winit::event::MouseButton;

use crate::Draw;
use crate::input::Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub border_radius: f32,
    pub border_thickness: f32,
    pub border_color: math::Vector4<f32>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            bg: math::Vector4::new(1.0, 1.0, 1.0, 1.0),
            fg: math::Vector4::new(0.0, 0.0, 0.0, 1.0),
            border_radius: 0.0,
            border_thickness: 0.0,
            border_color: math::Vector4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl Style {
    pub fn lerp(&self, to: &Style, t: f32) -> Style {
        use math::Lerp;

        Style {
            bg: self.bg.lerp(&to.bg, t),
            fg: self.fg.lerp(&to.fg, t),
            border_radius: self.border_radius.lerp(&to.border_radius, t),
            border_thickness: self.border_thickness.lerp(&to.border_thickness, t),
            border_color: self.border_color.lerp(&to.border_color, t),
        }
    }

    pub fn patched(mut self, p: &StylePatch) -> Self {
        if let Some(bg) = p.bg {
            self.bg = bg;
        }
        if let Some(fg) = p.fg {
            self.fg = fg;
        }
        if let Some(r) = p.border_radius {
            self.border_radius = r;
        }
        if let Some(t) = p.border_thickness {
            self.border_thickness = t;
        }
        if let Some(c) = p.border_color {
            self.border_color = c;
        }
        self
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct StylePatch {
    pub bg: Option<math::Vector4<f32>>,
    pub fg: Option<math::Vector4<f32>>,
    pub border_radius: Option<f32>,
    pub border_thickness: Option<f32>,
    pub border_color: Option<math::Vector4<f32>>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WidgetStyle {
    pub base: Style,
    pub hovered: Style,
    pub held: Style,
}

impl WidgetStyle {
    pub fn from_accent(accent: math::Vector4<f32>, fg: math::Vector4<f32>) -> Self {
        let shade = |c: math::Vector4<f32>, k: f32| {
            math::Vector4::new(
                (c.x * k).min(1.0),
                (c.y * k).min(1.0),
                (c.z * k).min(1.0),
                c.w,
            )
        };

        Self {
            base: Style {
                bg: accent,
                fg,
                ..Default::default()
            },
            hovered: Style {
                bg: shade(accent, 1.35),
                fg,
                ..Default::default()
            },
            held: Style {
                bg: shade(accent, 0.7),
                fg,
                ..Default::default()
            },
        }
    }

    pub fn resolve_animated(&self, t_hover: f32, t_held: f32) -> Style {
        let t_hover = math::Easing::CubicOut.apply(t_hover);
        let t_held = math::Easing::QuadOut.apply(t_held);

        self.base
            .lerp(&self.hovered, t_hover)
            .lerp(&self.held, t_held)
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
        let fg = v(0.9, 0.9, 0.9);

        let mut slider = WidgetStyle::from_accent(v(0.20, 0.22, 0.27), fg);
        slider.held.bg = v(0.35, 0.55, 0.95);

        Self {
            button: WidgetStyle::from_accent(v(0.20, 0.22, 0.27), fg),
            checkbox: WidgetStyle::from_accent(v(0.20, 0.22, 0.27), fg),
            slider,
            text: v(0.92, 0.92, 0.92),
            padding: 8.0,
            item_gap: 6.0,
            widget_height: 32.0,
            font,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct AnimState {
    hover: f32,
    held: f32,
    seen: bool,
}

#[derive(Default)]
pub struct UiState {
    hot: Option<UiId>,
    active: Option<UiId>,
    pub wants_mouse: bool,
    anims: HashMap<UiId, AnimState>,
}

impl UiState {
    pub fn is_animating(&self) -> bool {
        self.anims
            .values()
            .any(|a| (a.hover > 0.0 && a.hover < 1.0) || (a.held > 0.0 && a.held < 1.0))
    }
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
    dt: f32,

    style_stack: Vec<StylePatch>,
}

impl<'a, 'd> Ui<'a, 'd> {
    pub fn begin(
        draw: &'a mut Draw<'d>,
        input: &Input,
        state: &'a mut UiState,
        theme: Theme,
        dt: f32,
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
            dt,
            style_stack: Vec::new(),
        }
    }

    pub fn end(self) {
        self.state.hot = self.hot_this_frame;

        if !self.mouse_held {
            self.state.active = None;
        }

        self.state.wants_mouse = self.hot_this_frame.is_some() || self.state.active.is_some();
        self.state.anims.retain(|_, a| std::mem::take(&mut a.seen));
    }

    pub fn with_style(&mut self, patch: StylePatch, f: impl FnOnce(&mut Self)) {
        self.style_stack.push(patch);
        f(self);
        self.style_stack.pop();
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

    fn animate(&mut self, id: UiId, hovered: bool, held: bool) -> (f32, f32) {
        let a = self.state.anims.entry(id).or_default();
        a.seen = true;

        let step = |v: &mut f32, target: f32, secs: f32, dt: f32| {
            let d = dt / secs.max(1e-6);
            *v = if target > *v {
                (*v + d).min(1.0)
            } else {
                (*v - d).max(0.0)
            };
        };

        step(&mut a.hover, hovered as u8 as f32, 0.12, self.dt);
        step(&mut a.held, held as u8 as f32, 0.05, self.dt);

        (a.hover, a.held)
    }

    fn resolve(&mut self, id: UiId, widget: WidgetStyle, hovered: bool, held: bool) -> Style {
        let (t_hover, t_held) = self.animate(id, hovered, held);
        let mut style = widget.resolve_animated(t_hover, t_held);

        for patch in &self.style_stack {
            style = style.patched(patch);
        }

        style
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
        let widget = self.theme.button;
        self.button_styled(label, widget)
    }

    pub fn button_styled(&mut self, label: &str, widget: WidgetStyle) -> bool {
        self.seq += 1;
        let id = UiId::make(label, self.seq);
        let rect = self.next_rect(self.theme.widget_height);
        let (hovered, held, clicked) = self.interact(id, rect);

        let style = self.resolve(id, widget, hovered, held);

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

        let widget = self.theme.checkbox;
        let style = self.resolve(id, widget, hovered, held);
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

        let widget = self.theme.slider;
        let style = self.resolve(id, widget, hovered, dragging);
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

    pub fn slider_styled(
        &mut self,
        label: &str,
        value: &mut f32,
        range: RangeInclusive<f32>,
        widget: WidgetStyle,
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

        let style = self.resolve(id, widget, hovered, dragging);
        let (lo, hi) = (*range.start(), *range.end());
        let t = ((*value - lo) / (hi - lo)).clamp(0.0, 1.0);

        self.draw.set_color(label_color);
        self.draw.text(
            self.theme.font,
            label,
            rect.position.x,
            rect.position.y + self.theme.padding,
        );

        self.draw.set_color(style.bg);
        self.draw.rect_v(track.position, track.size);
        self.draw.set_color(style.fg);
        self.draw.rect_v(
            track.position,
            math::Size::new(track.size.width * t, track.size.height),
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
}
