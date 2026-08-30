//! A tour of karna's text stack.
//!
//! Pages are switched with `1`..`4` (or `Tab` / `Shift+Tab`):
//!
//!   1. typography  - sizes, families, synthetic styles, line height, wrapping, alignment
//!   2. rich text   - per-span colour / weight / family / metadata, fallback, shaping, emoji
//!   3. effects     - per-glyph iteration: wave, rainbow, typewriter, pop, shadow, rgb split
//!   4. editor      - a real single-line editor: caret, selection, hit testing, IME preedit
//!
//! The second window shows the glyph atlas the text system rasterises into, so you can
//! watch pages fill up as new sizes, faces and colour glyphs get cached.

use karna::prelude::*;

const SHOWCASE_SCENE: SceneId = SceneId::new_str("text_showcase");
const ATLAS_SCENE: SceneId = SceneId::new_str("atlas");

// ---- palette ---------------------------------------------------------------

const INK: Color = Color::hex(0xE6E6E6);
const DIM: Color = Color::hex(0x8A8F98);
const FAINT: Color = Color::hex(0x3C4049);
const BLUE: Color = Color::hex(0x7AA2F7);
const PURPLE: Color = Color::hex(0xBB9AF7);
const GREEN: Color = Color::hex(0x9ECE6A);
const YELLOW: Color = Color::hex(0xE0AF68);
const RED: Color = Color::hex(0xF7768E);
const PANEL: Color = Color::hex(0x1E2029);
const SELECTION: Color = Color::hex(0x33467C);

// ---- copy ------------------------------------------------------------------

const PARAGRAPH: &str = "Shaping turns a string into positioned glyphs: kerning pairs \
     collapse, ligatures fuse, and scripts that need it get reordered. Wrapping then breaks \
     the run into lines that fit the box you asked for.";

const ALIGNED: &str = "Alignment needs a box to align inside, so it only bites once a wrap \
     width is set. Justified spreads the spaces of every line but the last.";

const ALIGNMENTS: [(TextAlign, &str); 4] = [
    (TextAlign::Left, "Left"),
    (TextAlign::Center, "Center"),
    (TextAlign::Right, "Right"),
    (TextAlign::Justified, "Justified"),
];

// ---- editor geometry -------------------------------------------------------

const FIELD_X: f32 = 40.0;
const FIELD_Y: f32 = 232.0;
const FIELD_W: f32 = 1200.0;
const FIELD_H: f32 = 72.0;
const FIELD_PAD: f32 = 20.0;

/// The page the tour opens on.
const START_PAGE: Page = Page::Typography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Typography,
    Rich,
    Effects,
    Editor,
}

impl Page {
    const ALL: [Self; 4] = [Self::Typography, Self::Rich, Self::Effects, Self::Editor];

    fn label(self) -> &'static str {
        match self {
            Self::Typography => "1 typography",
            Self::Rich => "2 rich text",
            Self::Effects => "3 effects",
            Self::Editor => "4 editor",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|page| *page == self).unwrap_or(0)
    }
}

struct Fonts {
    sans: Handle<Font>,
    serif: Handle<Font>,
    mono: Handle<Font>,
    emoji: Handle<Font>,
}

/// Styles are plain data: build them once, clone and tweak per call site.
struct Styles {
    title: TextStyle,
    tab: TextStyle,
    label: TextStyle,
    body: TextStyle,
    mono: TextStyle,
    field: TextStyle,
}

struct Showcase {
    fonts: Fonts,
    styles: Styles,
    tabs: Vec<f32>,
    page: Page,
    clock: f32,
    blink: f32,
    wrap: f32,
    align: usize,
    link: Option<usize>,
    text: String,
    caret: usize,
    anchor: usize,
    dragging: bool,
}

impl Showcase {
    // ---- editing primitives (byte indices, kept on char boundaries) --------

    fn selection(&self) -> (usize, usize) {
        (self.caret.min(self.anchor), self.caret.max(self.anchor))
    }

    fn drop_selection(&mut self) -> bool {
        let (start, end) = self.selection();

        if start == end {
            return false;
        }

        self.text.replace_range(start..end, "");
        self.caret = start;
        self.anchor = start;

        true
    }

    fn insert(&mut self, s: &str) {
        self.drop_selection();
        self.text.insert_str(self.caret, s);
        self.caret += s.len();
        self.anchor = self.caret;
        self.blink = 0.0;
    }

    fn prev_boundary(&self) -> usize {
        self.text[..self.caret]
            .chars()
            .next_back()
            .map(|c| self.caret - c.len_utf8())
            .unwrap_or(0)
    }

    fn next_boundary(&self) -> usize {
        self.text[self.caret..]
            .chars()
            .next()
            .map(|c| self.caret + c.len_utf8())
            .unwrap_or(self.caret)
    }

    fn move_caret(&mut self, to: usize, select: bool) {
        self.caret = to;
        self.blink = 0.0;

        if !select {
            self.anchor = to;
        }
    }

    fn backspace(&mut self) {
        if self.drop_selection() {
            return;
        }

        let prev = self.prev_boundary();

        if prev != self.caret {
            self.text.replace_range(prev..self.caret, "");
            self.move_caret(prev, false);
        }
    }

    fn delete(&mut self) {
        if self.drop_selection() {
            return;
        }

        let next = self.next_boundary();

        if next != self.caret {
            self.text.replace_range(self.caret..next, "");
            self.blink = 0.0;
        }
    }

    // ---- spans -------------------------------------------------------------

    /// The rich paragraph is built in one place because both `update` (hit testing)
    /// and `draw` (rendering) need the exact same spans.
    fn rich_spans(&self) -> Vec<TextSpan<'static>> {
        vec![
            TextSpan::new("karna shapes a paragraph through "),
            TextSpan::new("cosmic-text")
                .with_font(self.fonts.mono)
                .with_color(GREEN)
                .with_metadata(1),
            TextSpan::new(", so one run can mix "),
            TextSpan::new("weights").bold().with_color(YELLOW),
            TextSpan::new(", "),
            TextSpan::new("italics").italic().with_color(PURPLE),
            TextSpan::new(", "),
            TextSpan::new("families")
                .with_font(self.fonts.serif)
                .with_color(BLUE),
            TextSpan::new(" and "),
            TextSpan::new("colours").with_color(RED),
            TextSpan::new(" without breaking kerning across the seams. Every span also carries a "),
            TextSpan::new("metadata")
                .with_font(self.fonts.mono)
                .with_color(GREEN)
                .with_metadata(2),
            TextSpan::new(" tag that survives shaping, which is all you need for "),
            TextSpan::new("hyperlinks")
                .with_color(BLUE)
                .with_metadata(3),
            TextSpan::new(", tooltips or syntax highlighting. Hover the tagged words."),
        ]
    }

    fn rich_style(&self) -> TextStyle {
        TextStyle::new(self.fonts.sans, 21.0)
            .with_line_height(34.0)
            .with_wrap(1180.0)
    }

    // ---- chrome ------------------------------------------------------------

    fn heading(&self, draw: &mut Draw, title: &str, x: f32, y: f32, rule: f32) -> f32 {
        draw.set_color(DIM);

        let size = draw.print(title, &self.styles.label, x, y);

        draw.set_color(FAINT);
        draw.rect(x, y + size.h() + 5.0, rule, 1.0);

        y + size.h() + 20.0
    }

    fn draw_chrome(&self, ctx: &DrawContext, draw: &mut Draw) {
        draw.set_color(INK);
        draw.print_rich(
            &[
                TextSpan::new("karna").bold().with_color(BLUE),
                TextSpan::new("  text system tour").with_color(DIM),
            ],
            &self.styles.title,
            40.0,
            26.0,
        );

        let mut x = 40.0;

        for (i, page) in Page::ALL.iter().enumerate() {
            let width = self.tabs[i];
            let active = *page == self.page;

            if active {
                draw.set_color(PANEL);
                draw.rect(x - 12.0, 68.0, width + 24.0, 30.0);
            }

            draw.set_color(if active { INK } else { DIM });
            draw.print(page.label(), &self.styles.tab, x, 74.0);

            x += width + 46.0;
        }

        let pages = draw.assets().atlas_page_count();
        let family = draw
            .assets()
            .font_family(self.fonts.mono)
            .unwrap_or("?")
            .to_owned();

        draw.set_color(FAINT);
        draw.rect(40.0, 668.0, 1200.0, 1.0);

        draw.set_color(DIM);
        draw.print(
            format!(
                "{:.0} fps   atlas pages: {}   mono family: {}   [1-4] / tab: page   esc: quit",
                ctx.time.fps(),
                pages,
                family,
            ),
            &self.styles.mono,
            40.0,
            684.0,
        );
    }

    // ---- pages -------------------------------------------------------------

    fn draw_typography(&self, draw: &mut Draw) {
        // --- size ramp: one style cloned per size, so metrics scale together.
        let mut y = self.heading(draw, "SIZE RAMP", 40.0, 120.0, 560.0);

        for size in [13.0, 18.0, 25.0, 35.0, 49.0] {
            let style = TextStyle::new(self.fonts.sans, size);

            draw.set_color(DIM);
            draw.print(
                format!("{size:>4.0}px"),
                &self.styles.mono,
                40.0,
                y + size * 0.35,
            );

            draw.set_color(INK);
            let measured = draw.print("Hamburgefonstiv", &style, 116.0, y);

            y += measured.h() + 4.0;
        }

        // --- families and synthetic styles.
        y = self.heading(draw, "FAMILIES & STYLES", 40.0, y + 12.0, 560.0);

        let rows: [(&str, Handle<Font>, bool, bool); 5] = [
            ("sans regular", self.fonts.sans, false, false),
            ("sans bold", self.fonts.sans, true, false),
            ("sans italic", self.fonts.sans, false, true),
            ("serif bold italic", self.fonts.serif, true, true),
            ("mono (loaded .ttf)", self.fonts.mono, false, false),
        ];

        for (label, font, bold, italic) in rows {
            let style = TextStyle::new(font, 19.0);

            draw.set_color(DIM);
            draw.print(label, &self.styles.mono, 40.0, y + 4.0);

            let mut span = TextSpan::new("Sphinx of black quartz").with_color(INK);

            if bold {
                span = span.bold();
            }

            if italic {
                span = span.italic();
            }

            draw.set_color(INK);
            y += draw.print_rich(&[span], &style, 216.0, y).h() + 6.0;
        }

        // --- shaping proof: ligatures and kerning pairs.
        y = self.heading(draw, "SHAPING", 40.0, y + 12.0, 560.0);

        draw.set_color(GREEN);
        y +=
            draw.print(
                "-> => != <= >= ::  |  fi fl ffi",
                &TextStyle::new(self.fonts.mono, 22.0),
                40.0,
                y,
            )
            .h() + 4.0;

        draw.set_color(DIM);
        draw.print(
            "AV  Wa  To  LT  P.  Yo  \"quoted\"",
            &TextStyle::new(self.fonts.serif, 22.0),
            40.0,
            y,
        );

        // --- line height, two columns of the same paragraph.
        let mut ry = self.heading(draw, "LINE HEIGHT", 660.0, 120.0, 580.0);

        for (factor, label) in [(1.0, "1.0x"), (1.7, "1.7x")] {
            let x = if factor > 1.0 { 960.0 } else { 660.0 };
            let style = TextStyle::new(self.fonts.sans, 15.0)
                .with_line_height(15.0 * factor)
                .with_wrap(270.0);

            draw.set_color(YELLOW);
            draw.print(label, &self.styles.mono, x, ry);

            draw.set_color(INK);
            draw.print(PARAGRAPH, &style, x, ry + 20.0);
        }

        ry += 190.0;

        // --- wrapping + alignment inside a live box.
        let (align, align_label) = ALIGNMENTS[self.align];

        ry = self.heading(draw, "WRAP & ALIGNMENT", 660.0, ry, 580.0);

        let style = TextStyle::new(self.fonts.sans, 17.0)
            .with_line_height(24.0)
            .with_wrap(self.wrap)
            .with_align(align);

        // Laying out first gives the measured block back, so the frame can be drawn
        // to the text instead of the text into a guessed frame.
        let laid = draw.layout(ALIGNED, &style);
        let size = laid.size();

        draw.set_color(PANEL);
        draw.rect(660.0, ry, self.wrap, size.h() + 20.0);

        draw.set_color(INK);
        draw.text(&laid, 660.0, ry + 10.0);

        frame(draw, FAINT, 660.0, ry, self.wrap, size.h() + 20.0, 1.0);

        draw.set_color(BLUE);
        draw.print(
            format!(
                "align: {align_label}   wrap: {:.0}px   measured: {:.0} x {:.0}px   [a] align   [left/right] wrap",
                self.wrap,
                size.w(),
                size.h(),
            ),
            &self.styles.mono,
            660.0,
            ry + size.h() + 32.0,
        );

        draw.set_color(DIM);
        draw.print(
            "`TextStyle::wrap` is the layout width: `None` keeps everything on one line \
             and disables alignment, `Some(w)` turns on word-or-glyph wrapping. \
             `Text::size` reports what the shaper actually produced, which is what the \
             frame above is drawn from.",
            &self
                .styles
                .mono
                .clone()
                .with_line_height(19.0)
                .with_wrap(580.0),
            660.0,
            ry + size.h() + 58.0,
        );
    }

    fn draw_rich(&self, draw: &mut Draw) {
        let y = self.heading(draw, "ONE SHAPED RUN, MANY SPANS", 40.0, 120.0, 1200.0);
        let style = self.rich_style();
        let spans = self.rich_spans();
        let laid = draw.layout_rich(&spans, &style);
        let origin = Vector2::new(40.0, y);

        // Metadata survives shaping, so the tagged runs can be underlined and
        // highlighted after layout without knowing anything about the string.
        for (id, baseline, x0, x1) in link_runs(&laid) {
            let hot = self.link == Some(id);

            draw.set_color(if hot { BLUE } else { FAINT });
            draw.rect(origin.x + x0, origin.y + baseline + 4.0, x1 - x0, 1.5);

            if hot {
                draw.set_color(Color::rgba(0.48, 0.64, 0.97, 0.15));
                draw.rect(
                    origin.x + x0 - 2.0,
                    origin.y + baseline - style.size * 0.85,
                    x1 - x0 + 4.0,
                    style.line_height * 0.85,
                );
            }
        }

        let hovered = self.link;

        draw_glyphs(draw, &laid, origin.x, origin.y, |glyph| {
            let base = glyph.color.unwrap_or(INK);

            match hovered {
                Some(id) if id == glyph.metadata => Tweak::new(Color::WHITE).offset(0.0, -1.0),
                _ => Tweak::new(base),
            }
        });

        let mut y = y + laid.size().h() + 34.0;

        // --- fallback: families the primary face has no glyphs for.
        y = self.heading(draw, "FALLBACK & SCRIPTS", 40.0, y, 580.0);

        let body = TextStyle::new(self.fonts.sans, 22.0).with_line_height(36.0);

        draw.set_color(INK);
        y += draw
            .print("日本語のテキスト - グリフのフォールバック", &body, 40.0, y)
            .h();
        y += draw.print("مرحبا بالعالم - تشكيل النص", &body, 40.0, y).h();
        draw.print("Ελληνικά  Кириллица  हिन्दी", &body, 40.0, y);

        // --- symbols and emoji, pulled in by the same fallback pass.
        let mut cy = self.heading(draw, "SYMBOLS & EMOJI", 660.0, y - 108.0, 580.0);

        draw.set_color(INK);
        cy +=
            draw.print_rich(
                &[
                    TextSpan::new("emoji: "),
                    TextSpan::new("🚀 🎉 ✨ 🐙 ").with_font(self.fonts.emoji),
                    TextSpan::new("math: ∀x ∈ ℝ  ∑ ∫ ≠ ∞"),
                ],
                &TextStyle::new(self.fonts.sans, 24.0),
                660.0,
                cy,
            )
            .h() + 10.0;

        // Mask glyphs take the draw colour; glyphs swash hands back as RGBA are
        // flagged `colored` and drawn untinted instead.
        draw.set_color(RED);
        cy +=
            draw.print_rich(
                &[
                    TextSpan::new("mask glyphs tint: "),
                    TextSpan::new("🚀 🎉").with_font(self.fonts.emoji),
                ],
                &TextStyle::new(self.fonts.sans, 24.0),
                660.0,
                cy,
            )
            .h() + 10.0;

        draw.set_color(DIM);
        draw.print(
            "swash rasterises CBDT and COLRv0 colour faces to RGBA, and the text \
             system flags those glyphs so they are drawn untinted. COLRv1 - what \
             ships as \"Noto Color Emoji\" on most distros today - is not supported \
             yet and rasterises to nothing, so this page asks for the monochrome \
             \"Noto Emoji\" face instead.",
            &self
                .styles
                .mono
                .clone()
                .with_line_height(19.0)
                .with_wrap(580.0),
            660.0,
            cy,
        );
    }

    fn draw_effects(&self, ctx: &DrawContext, draw: &mut Draw) {
        let y = self.heading(draw, "PER-GLYPH CONTROL", 40.0, 120.0, 1200.0);
        let style = TextStyle::new(self.fonts.sans, 34.0);
        let phrase = "the quick brown fox jumps over the lazy dog";
        let clock = self.clock;

        let row = |draw: &mut Draw, index: usize, label: &str| -> (Text, f32) {
            let y = y + index as f32 * 80.0;

            draw.set_color(DIM);
            draw.print(label, &self.styles.mono, 40.0, y + 14.0);

            (draw.layout(phrase, &style), y)
        };

        // 1. wave: phase taken from the shaped pen position, so wide glyphs and
        //    kerning pull the crest along with them.
        let (laid, ry) = row(draw, 0, "wave");

        draw_glyphs(draw, &laid, 260.0, ry, |glyph| {
            let phase = clock * 5.0 - glyph.pen.x * 0.035;

            Tweak::new(INK).offset(0.0, phase.sin() * 7.0)
        });

        // 2. rainbow: colour ramped along the run.
        let (laid, ry) = row(draw, 1, "gradient");

        draw_glyphs(draw, &laid, 260.0, ry, |glyph| {
            let t = glyph.index as f32 / glyph.count.max(1) as f32;

            Tweak::new(hue(t * 0.7 + clock * 0.15))
        });

        // 3. typewriter: reveal by index, with a caret riding the cut.
        let (laid, ry) = row(draw, 2, "typewriter");
        let count = laid.glyphs().len();
        let cut = (clock * 9.0) % (count as f32 + 14.0);

        draw_glyphs(draw, &laid, 260.0, ry, |glyph| {
            let d = cut - glyph.index as f32;
            let alpha = d.clamp(0.0, 1.0);

            Tweak::new(Color::rgba(INK.r, INK.g, INK.b, alpha))
                .offset(0.0, (1.0 - alpha) * 8.0)
                .scale(0.7 + alpha * 0.3)
        });

        if let Some(glyph) = laid
            .glyphs()
            .get((cut as usize).min(count.saturating_sub(1)))
        {
            draw.set_color(GREEN);
            draw.rect(260.0 + glyph.pen.x, ry + 4.0, 2.0, style.size);
        }

        // 4. cursor pull: the pen position of each glyph against the mouse, so the
        //    effect follows the shaped run rather than a guessed character grid.
        let (laid, ry) = row(draw, 3, "cursor");
        let mouse = ctx.window.mouse_position();

        draw_glyphs(draw, &laid, 260.0, ry, |glyph| {
            let gx = 260.0 + glyph.pen.x;
            let gy = ry + style.size * 0.5;
            let distance = ((mouse.x - gx).powi(2) + (mouse.y - gy).powi(2)).sqrt();
            let pull = (1.0 - distance / 130.0).clamp(0.0, 1.0).powi(2);

            Tweak::new(mix(INK, YELLOW, pull))
                .scale(1.0 + pull * 0.45)
                .offset(0.0, -pull * 12.0)
        });

        // 5. shadow + outline: the same layout drawn several times.
        let (laid, ry) = row(draw, 4, "shadow");

        for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
            draw_glyphs(draw, &laid, 260.0 + dx, ry + dy, |_| {
                Tweak::new(Color::rgba(0.0, 0.0, 0.0, 0.85))
            });
        }

        draw_glyphs(draw, &laid, 264.0, ry + 5.0, |_| {
            Tweak::new(Color::rgba(BLUE.r, BLUE.g, BLUE.b, 0.35))
        });

        draw_glyphs(draw, &laid, 260.0, ry, |_| Tweak::new(INK));

        // 6. rgb split, with a per-glyph hash for the jitter.
        let (laid, ry) = row(draw, 5, "rgb split");
        let jolt = ((clock * 3.0).sin() * 0.5 + 0.5).powf(8.0);

        for (channel, dir) in [(RED, -1.0), (GREEN, 0.0), (BLUE, 1.0)] {
            draw_glyphs(draw, &laid, 260.0, ry, |glyph| {
                let n = noise(glyph.index as f32 + (clock * 12.0).floor());
                let shake = (n - 0.5) * 7.0 * jolt;

                Tweak::new(Color::rgba(channel.r, channel.g, channel.b, 0.75))
                    .offset(dir * (1.5 + jolt * 4.0) + shake, 0.0)
            });
        }

        draw.set_color(DIM);
        draw.print(
            "every row above is the same laid-out `Text`, walked glyph by glyph at draw time \
             - move the mouse over the `cursor` row",
            &self.styles.mono,
            40.0,
            y + 6.0 * 80.0 - 6.0,
        );
    }

    fn draw_editor(&self, ctx: &DrawContext, draw: &mut Draw) {
        self.heading(draw, "CARET, SELECTION & HIT TESTING", 40.0, 120.0, 1200.0);

        draw.set_color(DIM);
        draw.print(
            "type to edit  -  click or drag to place the caret  -  shift+arrows select  -  \
             ctrl+a selects all  -  IME preedit is drawn dimmed",
            &self.styles.body,
            40.0,
            168.0,
        );

        let style = &self.styles.field;
        let text_x = FIELD_X + FIELD_PAD;
        let text_y = FIELD_Y + (FIELD_H - style.line_height) * 0.5;

        draw.set_color(PANEL);
        draw.rect(FIELD_X, FIELD_Y, FIELD_W, FIELD_H);
        frame(draw, BLUE, FIELD_X, FIELD_Y, FIELD_W, FIELD_H, 1.0);

        let laid = draw.layout(&self.text, style);
        let (start, end) = self.selection();

        // `caret_x` maps a byte offset back to a pixel offset, which is all a
        // selection highlight needs.
        if start != end {
            let x0 = laid.caret_x(start);
            let x1 = laid.caret_x(end);

            draw.set_color(SELECTION);
            draw.rect(text_x + x0, text_y, x1 - x0, style.line_height);
        }

        if self.text.is_empty() {
            draw.set_color(FAINT);
            draw.print("type something...", style, text_x, text_y);
        } else {
            draw.set_color(INK);
            draw.text(&laid, text_x, text_y);
        }

        let caret_x = text_x + laid.caret_x(self.caret);

        if self.blink % 1.0 < 0.55 {
            draw.set_color(BLUE);
            draw.rect(caret_x, text_y - 2.0, 2.0, style.line_height + 4.0);
        }

        // Platform IME composition: not committed yet, so it lives beside the buffer.
        let preedit = ctx.input.preedit();

        if !preedit.is_empty() {
            draw.set_color(YELLOW);

            let size = draw.print(preedit, style, caret_x, text_y);

            draw.rect(caret_x, text_y + size.h() - 2.0, size.w(), 1.0);
        }

        let mouse = ctx.window.mouse_position();
        let hit = laid.byte_at_x(mouse.x - text_x);

        draw.set_color(GREEN);
        draw.print(
            format!(
                "{} bytes   {} chars   caret {}   selection {}..{}   measured {:.1}px   \
                 byte under mouse: {}",
                self.text.len(),
                self.text.chars().count(),
                self.caret,
                start,
                end,
                laid.size().w(),
                hit,
            ),
            &self.styles.mono,
            FIELD_X,
            FIELD_Y + FIELD_H + 20.0,
        );

        // A width bar makes `Text::size()` legible: it tracks the string exactly.
        draw.set_color(FAINT);
        draw.rect(FIELD_X, FIELD_Y + FIELD_H + 56.0, FIELD_W, 6.0);
        draw.set_color(BLUE);
        draw.rect(
            FIELD_X,
            FIELD_Y + FIELD_H + 56.0,
            laid.size().w().min(FIELD_W),
            6.0,
        );

        draw.set_color(DIM);
        draw.print(
            "the same layout drives the caret, the selection, the mouse hit test and the \
             IME rectangle reported to the compositor",
            &self.styles.mono,
            FIELD_X,
            FIELD_Y + FIELD_H + 78.0,
        );
    }
}

impl Scene for Showcase {
    fn load(ctx: &mut LoadContext) -> Self {
        let fonts = Fonts {
            sans: ctx.assets.system_font("Inter"),
            serif: ctx.assets.system_font("Noto Serif"),
            mono: ctx.assets.load_font("assets/jbmono.ttf"),
            // Monochrome on purpose: swash rasterises CBDT / COLRv0 colour faces,
            // but not the COLRv1 "Noto Color Emoji" most distros now ship, which
            // comes back as an empty image. See the colour-glyph note on page 2.
            emoji: ctx.assets.system_font("Noto Emoji"),
        };

        let styles = Styles {
            title: TextStyle::new(fonts.sans, 26.0),
            tab: TextStyle::new(fonts.sans, 16.0),
            label: TextStyle::new(fonts.mono, 12.0),
            body: TextStyle::new(fonts.sans, 17.0),
            mono: TextStyle::new(fonts.mono, 13.0),
            field: TextStyle::new(fonts.mono, 24.0).with_line_height(32.0),
        };

        // Text can be laid out outside of `draw` too - here to size the tab bar once,
        // instead of guessing pixel widths.
        let tabs = Page::ALL
            .iter()
            .map(|page| ctx.text().layout(page.label(), &styles.tab).size().w())
            .collect::<Vec<_>>();

        // Text input is a platform mode, so it has to be asked for explicitly.
        if START_PAGE == Page::Editor {
            ctx.window.start_text_input();
        }

        let text = String::from("edit me - ligatures ->, accents éàü, symbols ∑ ≠ ∞, CJK 日本語");
        let caret = text.len();

        Self {
            fonts,
            styles,
            tabs,
            page: START_PAGE,
            clock: 0.0,
            blink: 0.0,
            wrap: 440.0,
            align: 0,
            link: None,
            text,
            caret,
            anchor: caret,
            dragging: false,
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let dt = ctx.time.delta();

        self.clock += dt;
        self.blink += dt;

        let keys = [Key::Num1, Key::Num2, Key::Num3, Key::Num4];
        let mut page = self.page;

        for (i, key) in keys.into_iter().enumerate() {
            if ctx.input.key_pressed(key) {
                page = Page::ALL[i];
            }
        }

        if ctx.input.key_pressed(Key::Tab) {
            let shift = ctx.input.key_down(Key::LShift) || ctx.input.key_down(Key::RShift);
            let step = if shift { Page::ALL.len() - 1 } else { 1 };

            page = Page::ALL[(self.page.index() + step) % Page::ALL.len()];
        }

        if page != self.page {
            self.page = page;
            self.link = None;
            ctx.window.set_cursor(Cursor::default());

            // Text input is a platform mode: only ask for it on the page that edits.
            if page == Page::Editor {
                ctx.window.start_text_input();
            } else {
                ctx.window.stop_text_input();
                ctx.window.clear_text_input_area();
            }
        }

        match self.page {
            Page::Typography => self.update_typography(ctx),
            Page::Rich => self.update_rich(ctx),
            Page::Effects => {}
            Page::Editor => self.update_editor(ctx),
        }
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        self.draw_chrome(&ctx, draw);

        match self.page {
            Page::Typography => self.draw_typography(draw),
            Page::Rich => self.draw_rich(draw),
            Page::Effects => self.draw_effects(&ctx, draw),
            Page::Editor => self.draw_editor(&ctx, draw),
        }
    }
}

impl Showcase {
    fn update_typography(&mut self, ctx: &mut UpdateContext) {
        let dt = ctx.time.delta();
        let fast = ctx.input.key_down(Key::LShift) || ctx.input.key_down(Key::RShift);
        let speed = if fast { 900.0 } else { 260.0 };

        if ctx.input.key_down(Key::Left) {
            self.wrap = (self.wrap - speed * dt).max(180.0);
        }

        if ctx.input.key_down(Key::Right) {
            self.wrap = (self.wrap + speed * dt).min(580.0);
        }

        if ctx.input.key_pressed(Key::A) {
            self.align = (self.align + 1) % ALIGNMENTS.len();
        }
    }

    fn update_rich(&mut self, ctx: &mut UpdateContext) {
        let mouse = ctx.window.mouse_position();
        let style = self.rich_style();
        let spans = self.rich_spans();

        // Laying out in `update` is fine: glyph rasterisation is cached by
        // (face, size, subpixel offset), so this costs a hash lookup per glyph.
        let laid = ctx.text().layout_rich(&spans, &style);

        self.link = link_at(&laid, Vector2::new(40.0, 140.0), style.line_height, mouse);

        ctx.window.set_cursor(match self.link {
            Some(_) => Cursor::System(SystemCursor::POINTER),
            None => Cursor::default(),
        });
    }

    fn update_editor(&mut self, ctx: &mut UpdateContext) {
        let select = ctx.input.key_down(Key::LShift) || ctx.input.key_down(Key::RShift);
        let ctrl = ctx.input.key_down(Key::LCtrl) || ctx.input.key_down(Key::RCtrl);
        let typed = ctx.input.text();

        if !typed.is_empty() {
            self.insert(typed);
        }

        if ctx.input.key_pressed(Key::Backspace) {
            self.backspace();
        }

        if ctx.input.key_pressed(Key::Delete) {
            self.delete();
        }

        if ctx.input.key_pressed(Key::Left) {
            let to = self.prev_boundary();
            self.move_caret(to, select);
        }

        if ctx.input.key_pressed(Key::Right) {
            let to = self.next_boundary();
            self.move_caret(to, select);
        }

        if ctx.input.key_pressed(Key::Home) {
            self.move_caret(0, select);
        }

        if ctx.input.key_pressed(Key::End) {
            self.move_caret(self.text.len(), select);
        }

        if ctrl && ctx.input.key_pressed(Key::A) {
            self.anchor = 0;
            self.caret = self.text.len();
        }

        let mouse = ctx.window.mouse_position();
        let inside = mouse.x >= FIELD_X
            && mouse.x < FIELD_X + FIELD_W
            && mouse.y >= FIELD_Y
            && mouse.y < FIELD_Y + FIELD_H;

        ctx.window.set_cursor(if inside {
            Cursor::System(SystemCursor::TEXT)
        } else {
            Cursor::default()
        });

        if inside && ctx.input.mouse_pressed(MouseButton::Left) {
            self.dragging = true;
        }

        if !ctx.input.mouse_down(MouseButton::Left) {
            self.dragging = false;
        }

        let style = self.styles.field.clone();
        let laid = ctx.text().layout(&self.text, &style);
        let text_x = FIELD_X + FIELD_PAD;

        if self.dragging {
            // `byte_at_x` is the inverse of `caret_x`: pixels back to a byte offset.
            let byte = laid.byte_at_x(mouse.x - text_x);
            let anchoring = ctx.input.mouse_pressed(MouseButton::Left) && !select;

            self.caret = byte;
            self.blink = 0.0;

            if anchoring {
                self.anchor = byte;
            }
        }

        // Tell the compositor where the edited text is, so IME candidate windows
        // and on-screen keyboards line up with the caret.
        let caret_x = text_x + laid.caret_x(self.caret);

        ctx.window.set_text_input_area(
            [FIELD_X as i32, FIELD_Y as i32],
            (FIELD_W as u32, FIELD_H as u32),
            (caret_x - FIELD_X) as i32,
        );
    }
}

/// The glyph atlas the text system rasterises into.
struct Atlas;

impl Scene for Atlas {
    fn load(_ctx: &mut LoadContext) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &mut UpdateContext) {}

    fn draw(&mut self, _ctx: DrawContext, draw: &mut Draw) {
        if let Some(page) = draw.assets().atlas_page_image(0) {
            draw.textured(page, 0.0, 0.0, 1024.0, 1024.0, Color::WHITE);
        }
    }
}

// ---- glyph helpers ---------------------------------------------------------

/// What a per-glyph callback gets to decide with.
#[derive(Debug, Clone, Copy)]
struct GlyphInfo {
    index: usize,
    count: usize,
    metadata: usize,
    color: Option<Color>,
    /// Baseline pen position, before the bitmap placement offset.
    pen: Vector2<f32>,
}

/// What a per-glyph callback gives back.
#[derive(Debug, Clone, Copy)]
struct Tweak {
    offset: Vector2<f32>,
    scale: f32,
    color: Color,
}

impl Tweak {
    fn new(color: Color) -> Self {
        Self {
            offset: Vector2::zero(),
            scale: 1.0,
            color,
        }
    }

    fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Vector2::new(x, y);
        self
    }

    fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

/// `Draw::text` with a hook per glyph - the whole point of laying out once and
/// drawing yourself. Scaling happens about the glyph centre so runs stay put.
fn draw_glyphs<F>(draw: &mut Draw, text: &Text, x: f32, y: f32, mut tweak: F)
where
    F: FnMut(GlyphInfo) -> Tweak,
{
    let count = text.glyphs().len();

    for (index, glyph) in text.glyphs().iter().enumerate() {
        let Some(image) = glyph.image else {
            continue;
        };

        let t = tweak(GlyphInfo {
            index,
            count,
            metadata: glyph.metadata,
            color: glyph.color,
            pen: glyph.pen,
        });

        let size = glyph.size.cast::<f32>();
        let w = size.w() * t.scale;
        let h = size.h() * t.scale;
        let cx = x + glyph.pos.x + size.w() * 0.5 + t.offset.x;
        let cy = y + glyph.pos.y + size.h() * 0.5 + t.offset.y;

        // Colour glyphs carry their own pixels; only the alpha is ours to touch.
        let color = if glyph.colored {
            Color::rgba(1.0, 1.0, 1.0, t.color.a)
        } else {
            t.color
        };

        draw.textured(image, cx - w * 0.5, cy - h * 0.5, w, h, color);
    }
}

/// Merge tagged glyphs into `(metadata, baseline, x0, x1)` runs, one per line.
fn link_runs(text: &Text) -> Vec<(usize, f32, f32, f32)> {
    let glyphs = text.glyphs();
    let mut runs: Vec<(usize, f32, f32, f32)> = Vec::new();

    for (i, glyph) in glyphs.iter().enumerate() {
        if glyph.metadata == 0 {
            continue;
        }

        // Right edge of the glyph: the next pen on the same line, or its bitmap.
        let right = match glyphs.get(i + 1) {
            Some(next) if next.line == glyph.line => next.pen.x,
            _ => glyph.pen.x + glyph.size.w() as f32,
        };

        match runs.last_mut() {
            Some(run) if run.0 == glyph.metadata && run.1 == glyph.pen.y => run.3 = right,
            _ => runs.push((glyph.metadata, glyph.pen.y, glyph.pen.x, right)),
        }
    }

    runs
}

fn link_at(
    text: &Text,
    origin: Vector2<f32>,
    line_height: f32,
    point: Vector2<f32>,
) -> Option<usize> {
    let glyphs = text.glyphs();

    for (i, glyph) in glyphs.iter().enumerate() {
        if glyph.metadata == 0 {
            continue;
        }

        let x0 = origin.x + glyph.pen.x;
        let x1 = origin.x
            + match glyphs.get(i + 1) {
                Some(next) if next.line == glyph.line => next.pen.x,
                _ => glyph.pen.x + glyph.size.w() as f32,
            };
        let top = origin.y + glyph.pen.y - line_height * 0.75;

        if point.x >= x0 && point.x < x1 && point.y >= top && point.y < top + line_height {
            return Some(glyph.metadata);
        }
    }

    None
}

// ---- misc ------------------------------------------------------------------

fn frame(draw: &mut Draw, color: Color, x: f32, y: f32, w: f32, h: f32, t: f32) {
    draw.set_color(color);
    draw.rect(x, y, w, t);
    draw.rect(x, y + h - t, w, t);
    draw.rect(x, y + t, t, h - t * 2.0);
    draw.rect(x + w - t, y + t, t, h - t * 2.0);
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn hue(h: f32) -> Color {
    let h = h.rem_euclid(1.0) * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();

    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };

    Color::rgb(r, g, b)
}

fn noise(i: f32) -> f32 {
    let v = (i * 12.9898).sin() * 43758.5453;

    v - v.floor()
}

fn main() {
    let _ = init_logging(LogConfig::default().with_min_level(LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("karna - text")
                .with_size((1280, 720))
                .with_scene::<Showcase>(SHOWCASE_SCENE)
                .with_active_scene(SHOWCASE_SCENE),
        )
        .with_window(
            WindowBuilder::new()
                .with_title("glyph atlas")
                .with_size((1024, 1024))
                .with_scene::<Atlas>(ATLAS_SCENE)
                .with_active_scene(ATLAS_SCENE),
        )
        .with_root("examples/")
        .build()
        .run();
}
